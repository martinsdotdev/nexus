//! Wire framing for the `/sync` WebSocket: a 1-byte tag followed by the payload.
//!
//! - `0x01` SnapshotRequest (no payload)
//! - `0x02` Snapshot (full `ExportMode::Snapshot` bytes)
//! - `0x03` Update (incremental Loro update bytes, forwarded verbatim)
//! - `0x04` Presence (opaque `EphemeralStore` bytes; the relay forwards these to
//!   other peers but never merges or persists them, ADR-0005/0009)
//!
//! Document and presence bytes are opaque (Loro doc binary / `EphemeralStore`
//! binary); this layer only tags them so a peer knows how to route the payload.
//! `decode` returns `None` for an unknown tag, so adding a tag is forward
//! compatible: an older peer silently drops frames it does not understand.

fn tagged(tag: u8, payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(payload.len() + 1);
    out.push(tag);
    out.extend_from_slice(payload);
    out
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Frame {
    SnapshotRequest,
    Snapshot(Vec<u8>),
    Update(Vec<u8>),
    /// Opaque presence (`EphemeralStore`) bytes the relay forwards to other peers
    /// without merging or persisting them.
    Presence(Vec<u8>),
}

impl Frame {
    /// Serialize the frame to its tagged wire form.
    pub fn encode(&self) -> Vec<u8> {
        match self {
            Frame::SnapshotRequest => vec![0x01],
            Frame::Snapshot(payload) => tagged(0x02, payload),
            Frame::Update(payload) => tagged(0x03, payload),
            Frame::Presence(payload) => tagged(0x04, payload),
        }
    }

    /// Parse a tagged wire message, `None` if empty or an unknown tag.
    pub fn decode(bytes: &[u8]) -> Option<Frame> {
        match bytes.split_first() {
            Some((0x01, [])) => Some(Frame::SnapshotRequest),
            Some((0x02, rest)) => Some(Frame::Snapshot(rest.to_vec())),
            Some((0x03, rest)) => Some(Frame::Update(rest.to_vec())),
            Some((0x04, rest)) => Some(Frame::Presence(rest.to_vec())),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_each_frame() {
        for frame in [
            Frame::SnapshotRequest,
            Frame::Snapshot(vec![1, 2, 3]),
            Frame::Update(vec![9, 8]),
            Frame::Presence(vec![5, 6, 7]),
        ] {
            assert_eq!(Frame::decode(&frame.encode()), Some(frame));
        }
    }

    #[test]
    fn rejects_empty_or_unknown_tag() {
        assert_eq!(Frame::decode(&[]), None);
        assert_eq!(Frame::decode(&[0xFF, 1, 2]), None);
        // The next unused tag stays unknown, so future tags are forward compatible.
        assert_eq!(Frame::decode(&[0x05, 1, 2]), None);
        assert_eq!(
            Frame::decode(&[0x01, 9]),
            None,
            "SnapshotRequest takes no payload"
        );
    }
}
