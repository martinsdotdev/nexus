//! Snapshot persistence behind a `WorkspacePersistence` port: load and save a
//! workspace's Loro snapshot keyed by `WorkspaceId`. Local file mode uses
//! `FilePersistence` (one `workspace.loro`, the id ignored); cloud mode adds a
//! Postgres-backed impl later. The port is async so the database impl fits; the
//! file impl does brief synchronous I/O (a small local snapshot).
//!
//! File writes are atomic (temp file in the same directory + rename) so a crash
//! mid write never corrupts the snapshot; `tempfile::persist` handles the Windows
//! replace-existing case the plain rename would fail on.

use std::io::Write;
use std::path::{Path, PathBuf};

use uuid::Uuid;

/// Identifies a workspace's document. Local file mode uses the single `LOCAL`
/// sentinel; cloud mode uses each workspace row's UUID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WorkspaceId(pub Uuid);

impl WorkspaceId {
    /// The one workspace served in local file mode.
    pub const LOCAL: WorkspaceId = WorkspaceId(Uuid::nil());
}

/// Loads and saves a workspace's Loro snapshot. Implementations: `FilePersistence`
/// (local) and, in cloud mode, a Postgres-backed store.
#[async_trait::async_trait]
pub trait WorkspacePersistence: Send + Sync {
    /// The snapshot bytes for `id`, or `None` if it has never been persisted.
    async fn load(&self, id: WorkspaceId) -> anyhow::Result<Option<Vec<u8>>>;
    /// Atomically replace the snapshot for `id`.
    async fn save(&self, id: WorkspaceId, snapshot: &[u8]) -> anyhow::Result<()>;
}

/// Single-file local persistence: one `workspace.loro` under a data directory.
/// The `WorkspaceId` is ignored (local mode serves exactly one workspace).
pub struct FilePersistence {
    path: PathBuf,
}

impl FilePersistence {
    pub fn new(data_dir: impl AsRef<Path>) -> Self {
        Self {
            path: data_dir.as_ref().join("workspace.loro"),
        }
    }
}

#[async_trait::async_trait]
impl WorkspacePersistence for FilePersistence {
    async fn load(&self, _id: WorkspaceId) -> anyhow::Result<Option<Vec<u8>>> {
        match std::fs::read(&self.path) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    async fn save(&self, _id: WorkspaceId, snapshot: &[u8]) -> anyhow::Result<()> {
        let dir = self
            .path
            .parent()
            .expect("snapshot path always has a parent directory");
        let mut tmp = tempfile::NamedTempFile::new_in(dir)?;
        tmp.write_all(snapshot)?;
        tmp.flush()?;
        tmp.persist(&self.path).map_err(|err| err.error)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn first_load_is_none_then_round_trips_and_overwrites() {
        let dir = tempfile::tempdir().unwrap();
        let store = FilePersistence::new(dir.path());

        assert_eq!(
            store.load(WorkspaceId::LOCAL).await.unwrap(),
            None,
            "first run: no snapshot"
        );

        store.save(WorkspaceId::LOCAL, &[1, 2, 3, 4]).await.unwrap();
        assert_eq!(
            store.load(WorkspaceId::LOCAL).await.unwrap(),
            Some(vec![1, 2, 3, 4])
        );

        store.save(WorkspaceId::LOCAL, &[9]).await.unwrap();
        assert_eq!(
            store.load(WorkspaceId::LOCAL).await.unwrap(),
            Some(vec![9]),
            "overwrite is atomic"
        );
    }

    #[tokio::test]
    async fn file_persistence_ignores_the_workspace_id() {
        let dir = tempfile::tempdir().unwrap();
        let store = FilePersistence::new(dir.path());

        // Two different ids hit the same single file (local mode has one workspace).
        store.save(WorkspaceId::LOCAL, &[1]).await.unwrap();
        let other = WorkspaceId(Uuid::from_u128(42));
        assert_eq!(store.load(other).await.unwrap(), Some(vec![1]));
    }
}
