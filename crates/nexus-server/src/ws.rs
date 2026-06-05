//! The `/sync` WebSocket: acquire the session's workspace from the registry,
//! snapshot-on-connect, then a bidirectional relay of Loro update frames plus opaque
//! presence frames. Incoming `Update` frames are merged through the runtime (which
//! validates/repairs/persists/broadcasts); rebroadcast deltas go back out to every
//! connected socket. Incoming `Presence` frames are forwarded opaquely to every OTHER
//! socket (never merged or persisted), so each peer sees the others' cursors/selections.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use axum::extract::State;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::Response;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::broadcast::error::RecvError;

use crate::http::AppState;
use crate::persistence::WorkspaceId;
use crate::protocol::Frame;
use crate::registry::WorkspaceRegistry;

/// Process-wide unique id per connection, so a session can skip echoing its own
/// presence back to itself.
static NEXT_CONN: AtomicU64 = AtomicU64::new(1);

pub async fn sync_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    // The workspace is resolved by the session/membership gate in a later increment;
    // for now every session opens the single local workspace.
    ws.on_upgrade(move |socket| handle_socket(socket, state.registry, WorkspaceId::LOCAL))
}

async fn handle_socket(socket: WebSocket, registry: Arc<WorkspaceRegistry>, id: WorkspaceId) {
    let runtime = match registry.acquire(id).await {
        Ok(runtime) => runtime,
        Err(error) => {
            tracing::warn!(%error, "failed to load workspace for /sync");
            return;
        }
    };
    let conn_id = NEXT_CONN.fetch_add(1, Ordering::Relaxed);
    let (mut sender, mut receiver) = socket.split();

    // Snapshot-on-connect: the new peer imports the canonical state, then both
    // sides exchange incremental updates.
    let snapshot = Frame::Snapshot(runtime.snapshot()).encode();
    if sender.send(Message::Binary(snapshot.into())).await.is_err() {
        registry.release(id).await;
        return;
    }

    // Forward rebroadcast deltas and other peers' presence out to this socket.
    let mut deltas = runtime.subscribe();
    let mut presence = runtime.subscribe_presence();
    let mut send_task = tokio::spawn(async move {
        loop {
            tokio::select! {
                delta = deltas.recv() => {
                    // A closed or lagged delta stream tears the socket down so the
                    // client reconnects and resyncs from a fresh snapshot; a missed
                    // incremental delta would otherwise leave the doc desynced.
                    let Ok(delta) = delta else { break };
                    let frame = Frame::Update(delta).encode();
                    if sender.send(Message::Binary(frame.into())).await.is_err() {
                        break;
                    }
                }
                msg = presence.recv() => match msg {
                    // Skip our own presence; forward everyone else's.
                    Ok((from, _)) if from == conn_id => {}
                    Ok((_, bytes)) => {
                        let frame = Frame::Presence(bytes).encode();
                        if sender.send(Message::Binary(frame.into())).await.is_err() {
                            break;
                        }
                    }
                    // Lagging presence is harmless (last-write-wins); only a closed
                    // channel ends the loop.
                    Err(RecvError::Lagged(_)) => {}
                    Err(RecvError::Closed) => break,
                },
            }
        }
    });

    // Route incoming frames: doc updates merge into the canonical document;
    // presence is forwarded opaquely to other peers (never merged or persisted).
    let recv_runtime = Arc::clone(&runtime);
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(message)) = receiver.next().await {
            if let Message::Binary(bytes) = message {
                match Frame::decode(bytes.as_ref()) {
                    Some(Frame::Update(update)) => {
                        let _ = recv_runtime.apply_remote(&update).await;
                    }
                    Some(Frame::Presence(presence)) => {
                        recv_runtime.forward_presence(conn_id, presence)
                    }
                    _ => {}
                }
            }
        }
    });

    // When either direction closes, tear down the other.
    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    }

    // Wait for both tasks to fully unwind (dropping their broadcast receivers) before
    // releasing, so the registry sees an accurate subscriber count and can evict.
    let _ = send_task.await;
    let _ = recv_task.await;
    registry.release(id).await;
}
