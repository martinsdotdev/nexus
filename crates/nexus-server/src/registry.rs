//! The workspace registry: the relay's map of live `WorkspaceRuntime`s keyed by
//! `WorkspaceId`. A workspace is loaded from persistence on its first join (race-safe:
//! concurrent first-joiners share one runtime), and evicted when its last session
//! disconnects (its final merge already persisted). Local mode holds exactly one
//! entry (the `LOCAL` sentinel); cloud mode holds one per live workspace, loaded on
//! demand (ADR-0009). Sticky routing across processes and an idle timer are deferred;
//! this is a single process, and the map is the seam a future router slots into.

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::Mutex;

use crate::persistence::{WorkspaceId, WorkspacePersistence};
use crate::runtime::WorkspaceRuntime;

pub struct WorkspaceRegistry {
    persistence: Arc<dyn WorkspacePersistence>,
    live: Mutex<HashMap<WorkspaceId, Arc<WorkspaceRuntime>>>,
}

impl WorkspaceRegistry {
    pub fn new(persistence: Arc<dyn WorkspacePersistence>) -> Arc<Self> {
        Arc::new(Self {
            persistence,
            live: Mutex::new(HashMap::new()),
        })
    }

    /// Get the live runtime for `id`, loading it from persistence on first join. The
    /// map lock is held across the load so two concurrent first-joiners share one
    /// runtime (the second finds it already inserted) rather than racing two loads.
    pub async fn acquire(&self, id: WorkspaceId) -> anyhow::Result<Arc<WorkspaceRuntime>> {
        let mut live = self.live.lock().await;
        if let Some(runtime) = live.get(&id) {
            return Ok(Arc::clone(runtime));
        }
        let runtime = WorkspaceRuntime::load(id, Arc::clone(&self.persistence)).await?;
        live.insert(id, Arc::clone(&runtime));
        Ok(runtime)
    }

    /// Release a session's interest in `id`. If no sessions remain connected, evict the
    /// runtime; a later join reloads it from persistence.
    pub async fn release(&self, id: WorkspaceId) {
        let mut live = self.live.lock().await;
        let idle = live.get(&id).is_some_and(|rt| rt.subscriber_count() == 0);
        if idle {
            live.remove(&id);
        }
    }

    /// Forcibly drop a workspace's live runtime, used when the workspace is deleted. Any
    /// session still holding the `Arc` keeps running until it disconnects; a fresh join is
    /// already barred by the deleted membership, so the gone workspace is never reloaded.
    pub async fn evict(&self, id: WorkspaceId) {
        self.live.lock().await.remove(&id);
    }

    #[cfg(test)]
    async fn live_count(&self) -> usize {
        self.live.lock().await.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence::{DatabasePersistence, FilePersistence};
    use uuid::Uuid;

    fn local_registry(dir: &std::path::Path) -> Arc<WorkspaceRegistry> {
        WorkspaceRegistry::new(Arc::new(FilePersistence::new(dir)))
    }

    #[tokio::test]
    async fn acquire_loads_once_for_concurrent_joiners() {
        let dir = tempfile::tempdir().unwrap();
        let registry = local_registry(dir.path());

        let a = registry.acquire(WorkspaceId::LOCAL).await.unwrap();
        let b = registry.acquire(WorkspaceId::LOCAL).await.unwrap();

        assert!(Arc::ptr_eq(&a, &b), "both joiners share one runtime");
        assert_eq!(registry.live_count().await, 1);
    }

    #[tokio::test]
    async fn evicts_when_the_last_session_releases() {
        let dir = tempfile::tempdir().unwrap();
        let registry = local_registry(dir.path());
        let runtime = registry.acquire(WorkspaceId::LOCAL).await.unwrap();

        // A live subscriber keeps the workspace resident.
        let session = runtime.subscribe();
        registry.release(WorkspaceId::LOCAL).await;
        assert_eq!(
            registry.live_count().await,
            1,
            "kept while a session is connected"
        );

        // The last subscriber leaving makes it evictable.
        drop(session);
        registry.release(WorkspaceId::LOCAL).await;
        assert_eq!(registry.live_count().await, 0, "evicted when idle");
        drop(runtime);
    }

    #[tokio::test]
    async fn distinct_workspaces_get_distinct_runtimes() {
        use testcontainers_modules::postgres::Postgres;
        use testcontainers_modules::testcontainers::runners::AsyncRunner;

        let container = Postgres::default().start().await.unwrap();
        let port = container.get_host_port_ipv4(5432).await.unwrap();
        let url = format!("postgres://postgres:postgres@127.0.0.1:{port}/postgres");
        let pool = sqlx::PgPool::connect(&url).await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();

        // Two workspace rows; the registry loads an independent doc for each.
        let (a, b) = (WorkspaceId(Uuid::new_v4()), WorkspaceId(Uuid::new_v4()));
        for id in [a, b] {
            sqlx::query("insert into workspace (id) values ($1)")
                .bind(id.0)
                .execute(&pool)
                .await
                .unwrap();
        }

        let registry = WorkspaceRegistry::new(Arc::new(DatabasePersistence::new(pool)));
        let ra = registry.acquire(a).await.unwrap();
        let rb = registry.acquire(b).await.unwrap();

        assert!(
            !Arc::ptr_eq(&ra, &rb),
            "different workspaces, different runtimes"
        );
        assert_eq!(registry.live_count().await, 2);
    }

    #[tokio::test]
    async fn evict_drops_the_runtime_even_with_a_live_subscriber() {
        let dir = tempfile::tempdir().unwrap();
        let registry = local_registry(dir.path());
        let runtime = registry.acquire(WorkspaceId::LOCAL).await.unwrap();
        let _session = runtime.subscribe();

        // release keeps it resident while a session is live; evict drops it outright.
        registry.release(WorkspaceId::LOCAL).await;
        assert_eq!(registry.live_count().await, 1);
        registry.evict(WorkspaceId::LOCAL).await;
        assert_eq!(
            registry.live_count().await,
            0,
            "evicted despite the live subscriber"
        );
        drop(runtime);
    }
}
