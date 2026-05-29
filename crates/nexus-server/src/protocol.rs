//! Wire framing for the `/sync` WebSocket: a 1-byte tag followed by the payload.
//!
//! - `0x01` SnapshotRequest (no payload)
//! - `0x02` Snapshot (full `ExportMode::Snapshot` bytes)
//! - `0x03` Update (incremental Loro update bytes, forwarded verbatim)
//!
//! Document bytes are opaque Loro binary; this layer only tags them so a peer
//! knows whether to import a snapshot or an update.

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
}

impl Frame {
    /// Serialize the frame to its tagged wire form.
    pub fn encode(&self) -> Vec<u8> {
        match self {
            Frame::SnapshotRequest => vec![0x01],
            Frame::Snapshot(payload) => tagged(0x02, payload),
            Frame::Update(payload) => tagged(0x03, payload),
        }
    }

    /// Parse a tagged wire message, `None` if empty or an unknown tag.
    pub fn decode(bytes: &[u8]) -> Option<Frame> {
        match bytes.split_first() {
            Some((0x01, [])) => Some(Frame::SnapshotRequest),
            Some((0x02, rest)) => Some(Frame::Snapshot(rest.to_vec())),
            Some((0x03, rest)) => Some(Frame::Update(rest.to_vec())),
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
        ] {
            assert_eq!(Frame::decode(&frame.encode()), Some(frame));
        }
    }

    #[test]
    fn rejects_empty_or_unknown_tag() {
        assert_eq!(Frame::decode(&[]), None);
        assert_eq!(Frame::decode(&[0xFF, 1, 2]), None);
        assert_eq!(
            Frame::decode(&[0x01, 9]),
            None,
            "SnapshotRequest takes no payload"
        );
    }
}
