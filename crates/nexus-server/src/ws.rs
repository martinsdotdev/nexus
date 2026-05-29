//! The `/sync` WebSocket: snapshot-on-connect, then a bidirectional relay of
//! Loro update frames. Incoming `Update` frames are merged through the runtime
//! (which validates/repairs/persists/broadcasts); rebroadcast deltas are
//! forwarded back out to every connected socket.

use std::sync::Arc;

use axum::extract::State;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::Response;
use futures_util::{SinkExt, StreamExt};

use crate::protocol::Frame;
use crate::runtime::WorkspaceRuntime;

pub async fn sync_handler(
    ws: WebSocketUpgrade,
    State(runtime): State<Arc<WorkspaceRuntime>>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, runtime))
}

async fn handle_socket(socket: WebSocket, runtime: Arc<WorkspaceRuntime>) {
    let (mut sender, mut receiver) = socket.split();

    // Snapshot-on-connect: the new peer imports the canonical state, then both
    // sides exchange incremental updates.
    let snapshot = Frame::Snapshot(runtime.snapshot()).encode();
    if sender.send(Message::Binary(snapshot.into())).await.is_err() {
        return;
    }

    // Forward every rebroadcast delta out to this socket.
    let mut rx = runtime.subscribe();
    let mut send_task = tokio::spawn(async move {
        while let Ok(delta) = rx.recv().await {
            let frame = Frame::Update(delta).encode();
            if sender.send(Message::Binary(frame.into())).await.is_err() {
                break;
            }
        }
    });

    // Merge incoming peer updates into the canonical document.
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(message)) = receiver.next().await {
            if let Message::Binary(bytes) = message
                && let Some(Frame::Update(update)) = Frame::decode(bytes.as_ref())
            {
                let _ = runtime.apply_remote(&update).await;
            }
        }
    });

    // When either direction closes, tear down the other.
    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    }
}
