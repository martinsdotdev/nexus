//! The canonical-replica runtime: the relay's authoritative `LoroDoc` plus the
//! `merge -> validate/repair -> persist -> broadcast` pipeline.
//!
//! `LoroDoc` mutates through `&self` (interior mutability) and is `Send + Sync`,
//! so the runtime is shared across WebSocket sessions via `Arc`. The `apply_lock`
//! serializes the read-modify-validate-write critical section so two concurrent
//! peer frames cannot interleave a repair. Echo-suppression is intentionally
//! omitted: Loro `import` is idempotent, so rebroadcasting a peer's own ops back
//! to it is a no-op that still carries any repair ops (ADR-0005 / plan T7, T4).

use std::sync::Arc;

use loro::LoroDoc;
use nexus_core::{default_doc, model, validate};
use tokio::sync::{Mutex, broadcast};

use crate::persistence::FilePersistence;

/// Capacity of the rebroadcast channel; deltas are small and consumed promptly.
const BROADCAST_CAPACITY: usize = 256;

/// The relay's canonical replica. Shared across sessions via `Arc`.
pub struct WorkspaceRuntime {
    doc: LoroDoc,
    apply_lock: Mutex<()>,
    broadcast: broadcast::Sender<Vec<u8>>,
    persistence: FilePersistence,
}

impl WorkspaceRuntime {
    /// Load the persisted snapshot, or build and persist the curated default on
    /// first run. The curated default is built ONLY here (never on a client) so
    /// node `TreeID`s are minted once (ADR-0005 / plan T1).
    pub fn new(persistence: FilePersistence) -> anyhow::Result<Arc<Self>> {
        let doc = LoroDoc::new();
        doc.set_peer_id(1)?;
        match persistence.load()? {
            Some(snapshot) => {
                let _ = doc.import(&snapshot)?;
            }
            None => {
                default_doc::build_default(&doc)?;
                persistence.save(&doc.export(loro::ExportMode::Snapshot)?)?;
            }
        }
        let (broadcast, _) = broadcast::channel(BROADCAST_CAPACITY);
        Ok(Arc::new(Self {
            doc,
            apply_lock: Mutex::new(()),
            broadcast,
            persistence,
        }))
    }

    /// Subscribe to the stream of update deltas the relay rebroadcasts.
    pub fn subscribe(&self) -> broadcast::Receiver<Vec<u8>> {
        self.broadcast.subscribe()
    }

    /// A full snapshot of the canonical document (sent to a newly-connected peer).
    pub fn snapshot(&self) -> Vec<u8> {
        self.doc
            .export(loro::ExportMode::Snapshot)
            .expect("snapshot export of an in-memory doc cannot fail")
    }

    /// The current read-model projection. The overlay route will consume this;
    /// for now it is exercised by the runtime tests.
    #[allow(dead_code)]
    pub fn workspace(&self) -> model::Workspace {
        model::read_workspace(&self.doc)
    }

    /// Merge a peer update, repair any invariant violation, persist, and
    /// broadcast the resulting delta to all peers. The `apply_lock` keeps the
    /// import -> validate -> repair -> persist sequence atomic.
    pub async fn apply_remote(&self, update: &[u8]) -> anyhow::Result<()> {
        let _guard = self.apply_lock.lock().await;

        let before = self.doc.oplog_vv();
        let _ = self.doc.import(update)?;

        let repairs = validate::validate(&model::read_workspace(&self.doc));
        validate::apply_repairs(&self.doc, &repairs)?;

        self.persistence.save(&self.snapshot())?;

        // Everything new since `before` = the peer's ops plus any repair ops.
        let delta = self.doc.export(loro::ExportMode::updates(&before))?;
        let _ = self.broadcast.send(delta);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Seed a peer from the canonical snapshot, activate the scene at `index`,
    /// and feed the resulting update through the relay. Returns the target id.
    async fn peer_activates(runtime: &WorkspaceRuntime, index: usize) -> String {
        let peer = LoroDoc::new();
        peer.set_peer_id(99).unwrap();
        peer.import(&runtime.snapshot()).unwrap();
        let base = peer.oplog_vv();

        let tree = peer.get_tree(nexus_core::schema::TREE);
        let layout = tree.roots()[0];
        let scenes = tree.children(layout).unwrap();
        let target = scenes[index].to_string();
        tree.get_meta(layout)
            .unwrap()
            .insert("activeSceneId", target.clone())
            .unwrap();
        peer.commit();

        let update = peer.export(loro::ExportMode::updates(&base)).unwrap();
        runtime.apply_remote(&update).await.unwrap();
        target
    }

    #[tokio::test]
    async fn apply_remote_merges_activation_and_broadcasts() {
        let dir = tempfile::tempdir().unwrap();
        let runtime = WorkspaceRuntime::new(FilePersistence::new(dir.path())).unwrap();
        let mut rx = runtime.subscribe();

        let target = peer_activates(&runtime, 1).await;

        assert_eq!(runtime.workspace().layouts[0].active_scene_id, target);
        let delta = rx.try_recv().expect("a delta was broadcast");
        assert!(!delta.is_empty());
    }

    #[tokio::test]
    async fn repairs_invalid_peer_activation() {
        let dir = tempfile::tempdir().unwrap();
        let runtime = WorkspaceRuntime::new(FilePersistence::new(dir.path())).unwrap();

        // A peer activates a scene id the layout does not own.
        let peer = LoroDoc::new();
        peer.set_peer_id(2).unwrap();
        peer.import(&runtime.snapshot()).unwrap();
        let base = peer.oplog_vv();
        let tree = peer.get_tree(nexus_core::schema::TREE);
        let layout = tree.roots()[0];
        tree.get_meta(layout)
            .unwrap()
            .insert("activeSceneId", "ghost")
            .unwrap();
        peer.commit();
        let update = peer.export(loro::ExportMode::updates(&base)).unwrap();

        runtime.apply_remote(&update).await.unwrap();

        let ws = runtime.workspace();
        assert_eq!(
            ws.layouts[0].active_scene_id, ws.layouts[0].scenes[0].id,
            "invalid activation repaired to the first scene"
        );
    }

    #[tokio::test]
    async fn repairs_dangling_peer_theme() {
        let dir = tempfile::tempdir().unwrap();
        let runtime = WorkspaceRuntime::new(FilePersistence::new(dir.path())).unwrap();

        // A peer points the live scene at a theme that does not exist.
        let peer = LoroDoc::new();
        peer.set_peer_id(3).unwrap();
        peer.import(&runtime.snapshot()).unwrap();
        let base = peer.oplog_vv();
        let tree = peer.get_tree(nexus_core::schema::TREE);
        let live = tree.children(tree.roots()[0]).unwrap()[0];
        tree.get_meta(live)
            .unwrap()
            .insert("themeId", "deleted-theme")
            .unwrap();
        peer.commit();
        let update = peer.export(loro::ExportMode::updates(&base)).unwrap();

        runtime.apply_remote(&update).await.unwrap();

        let ws = runtime.workspace();
        assert_eq!(
            ws.layouts[0].scenes[0].theme_id, "cozy",
            "dangling theme repaired to the default"
        );
    }

    #[tokio::test]
    async fn reloads_persisted_state_on_restart() {
        let dir = tempfile::tempdir().unwrap();
        let target = {
            let runtime = WorkspaceRuntime::new(FilePersistence::new(dir.path())).unwrap();
            peer_activates(&runtime, 1).await
        };

        let restarted = WorkspaceRuntime::new(FilePersistence::new(dir.path())).unwrap();
        assert_eq!(
            restarted.workspace().layouts[0].active_scene_id,
            target,
            "state restored from the persisted snapshot"
        );
    }
}
