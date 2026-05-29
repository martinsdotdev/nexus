//! `nexus-server`: the trusted Nexus sync relay (walking-skeleton stage).
//!
//! Holds the canonical Loro replica, merges peer updates, validates/repairs,
//! persists a snapshot, and rebroadcasts to peers. The relay runtime, the
//! WebSocket sync endpoint, and persistence land in the next steps. See
//! ADR-0005 and the implementation plan.

fn main() -> anyhow::Result<()> {
    Ok(())
}
