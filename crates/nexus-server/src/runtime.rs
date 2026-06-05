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

use crate::persistence::{WorkspaceId, WorkspacePersistence};

/// Capacity of the rebroadcast channel; deltas are small and consumed promptly.
const BROADCAST_CAPACITY: usize = 256;

/// Capacity of the presence relay channel. Presence is high-frequency (cursors)
/// but last-write-wins, so a slow consumer that lags simply skips stale frames.
const PRESENCE_CAPACITY: usize = 256;

/// The relay's canonical replica for one workspace. Shared across sessions via
/// `Arc`; in cloud mode the registry holds one per live workspace.
pub struct WorkspaceRuntime {
    id: WorkspaceId,
    doc: LoroDoc,
    apply_lock: Mutex<()>,
    broadcast: broadcast::Sender<Vec<u8>>,
    // Opaque presence frames keyed by their originating connection id. The relay
    // never imports, validates, or persists these (Loro's `EphemeralStore` is
    // JS-only); it only fans them out so each session can skip its own echo.
    presence: broadcast::Sender<(u64, Vec<u8>)>,
    persistence: Arc<dyn WorkspacePersistence>,
}

impl WorkspaceRuntime {
    /// Load workspace `id`'s persisted snapshot, or build and persist the curated
    /// default on first run. The curated default is built ONLY here (never on a
    /// client) so node `TreeID`s are minted once (ADR-0005 / plan T1).
    pub async fn load(
        id: WorkspaceId,
        persistence: Arc<dyn WorkspacePersistence>,
    ) -> anyhow::Result<Arc<Self>> {
        let doc = LoroDoc::new();
        doc.set_peer_id(1)?;
        match persistence.load(id).await? {
            Some(snapshot) => {
                let _ = doc.import(&snapshot)?;
            }
            None => {
                default_doc::build_default(&doc)?;
                persistence
                    .save(id, &doc.export(loro::ExportMode::Snapshot)?)
                    .await?;
            }
        }
        // Migrate snapshots that predate the seeded built-in themes (ADR-0007):
        // re-seed any missing built-in so scenes referencing them resolve. Idempotent
        // and a no-op for a freshly built default, which already seeded them.
        if default_doc::ensure_builtin_themes(&doc)? {
            doc.commit();
            persistence
                .save(id, &doc.export(loro::ExportMode::Snapshot)?)
                .await?;
        }
        let (broadcast, _) = broadcast::channel(BROADCAST_CAPACITY);
        let (presence, _) = broadcast::channel(PRESENCE_CAPACITY);
        Ok(Arc::new(Self {
            id,
            doc,
            apply_lock: Mutex::new(()),
            broadcast,
            presence,
            persistence,
        }))
    }

    /// Subscribe to the stream of update deltas the relay rebroadcasts.
    pub fn subscribe(&self) -> broadcast::Receiver<Vec<u8>> {
        self.broadcast.subscribe()
    }

    /// Subscribe to the presence relay. Each item is `(origin connection id,
    /// opaque presence bytes)`; a session forwards every item whose origin is not
    /// its own, so a peer never receives its own presence back.
    pub fn subscribe_presence(&self) -> broadcast::Receiver<(u64, Vec<u8>)> {
        self.presence.subscribe()
    }

    /// Forward an opaque presence frame from connection `from` to every other
    /// connection. Pure fan-out: the document is never touched and nothing is
    /// persisted (the relay holds no `EphemeralStore`).
    pub fn forward_presence(&self, from: u64, bytes: Vec<u8>) {
        let _ = self.presence.send((from, bytes));
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

        self.persistence.save(self.id, &self.snapshot()).await?;

        // Everything new since `before` = the peer's ops plus any repair ops.
        let delta = self.doc.export(loro::ExportMode::updates(&before))?;
        let _ = self.broadcast.send(delta);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence::FilePersistence;

    /// A local-mode runtime backed by a fresh file persistence under `dir`.
    async fn local_runtime(dir: &std::path::Path) -> Arc<WorkspaceRuntime> {
        WorkspaceRuntime::load(WorkspaceId::LOCAL, Arc::new(FilePersistence::new(dir)))
            .await
            .unwrap()
    }

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
        let runtime = local_runtime(dir.path()).await;
        let mut rx = runtime.subscribe();

        let target = peer_activates(&runtime, 1).await;

        assert_eq!(runtime.workspace().layouts[0].active_scene_id, target);
        let delta = rx.try_recv().expect("a delta was broadcast");
        assert!(!delta.is_empty());
    }

    #[tokio::test]
    async fn repairs_invalid_peer_activation() {
        let dir = tempfile::tempdir().unwrap();
        let runtime = local_runtime(dir.path()).await;

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
        let runtime = local_runtime(dir.path()).await;

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
    async fn reseeds_builtin_themes_for_legacy_snapshots() {
        let dir = tempfile::tempdir().unwrap();

        // Simulate a pre-ADR-0007 snapshot: the curated default with the theme
        // registry emptied (older relays never seeded the built-ins as data).
        let legacy = LoroDoc::new();
        legacy.set_peer_id(1).unwrap();
        default_doc::build_default(&legacy).unwrap();
        let registry = legacy.get_map(nexus_core::schema::THEMES);
        for id in ["cozy", "cyber", "editorial", "sticker"] {
            registry.delete(id).unwrap();
        }
        legacy.commit();
        FilePersistence::new(dir.path())
            .save(
                WorkspaceId::LOCAL,
                &legacy.export(loro::ExportMode::Snapshot).unwrap(),
            )
            .await
            .unwrap();

        // The relay loads the legacy snapshot and migrates the built-ins back in.
        let runtime = local_runtime(dir.path()).await;
        let ids: Vec<String> = runtime
            .workspace()
            .themes
            .iter()
            .map(|theme| theme.id.clone())
            .collect();
        for id in ["cozy", "cyber", "editorial", "sticker"] {
            assert!(ids.iter().any(|t| t == id), "{id} re-seeded on load");
        }

        // The migration persisted, so a restart needs no further re-seeding.
        let restarted = local_runtime(dir.path()).await;
        assert_eq!(restarted.workspace().themes.len(), 4, "migration persisted");
    }

    #[tokio::test]
    async fn reloads_persisted_state_on_restart() {
        let dir = tempfile::tempdir().unwrap();
        let target = {
            let runtime = local_runtime(dir.path()).await;
            peer_activates(&runtime, 1).await
        };

        let restarted = local_runtime(dir.path()).await;
        assert_eq!(
            restarted.workspace().layouts[0].active_scene_id,
            target,
            "state restored from the persisted snapshot"
        );
    }

    #[tokio::test]
    async fn forward_presence_reaches_subscribers_with_its_origin() {
        let dir = tempfile::tempdir().unwrap();
        let runtime = local_runtime(dir.path()).await;
        let mut rx = runtime.subscribe_presence();

        runtime.forward_presence(7, vec![1, 2, 3]);

        // The origin id rides along so each session can skip its own echo.
        assert_eq!(rx.try_recv().unwrap(), (7, vec![1, 2, 3]));
    }

    #[tokio::test]
    async fn presence_never_touches_the_document() {
        let dir = tempfile::tempdir().unwrap();
        let runtime = local_runtime(dir.path()).await;
        let before = runtime.snapshot();

        runtime.forward_presence(1, vec![9, 9, 9]);

        assert_eq!(
            runtime.snapshot(),
            before,
            "forwarding presence must not mutate (or persist) the document"
        );
    }
}
