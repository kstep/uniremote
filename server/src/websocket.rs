use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use axum::{
    extract::{
        Path, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    http::StatusCode,
    response::Response,
};
use axum_extra::extract::cookie::CookieJar;
use flume::Receiver;
use futures_util::{
    sink::SinkExt,
    stream::{SplitSink, SplitStream, StreamExt},
};
use uniremote_core::{ClientMessage, RemoteId, ServerMessage};
use uniremote_worker::LuaWorker;

use crate::{AppState, auth::AUTH_COOKIE_NAME};

pub async fn websocket_handler(
    Path(remote_id): Path<RemoteId>,
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    ws: WebSocketUpgrade,
) -> Result<Response, StatusCode> {
    // Extract token from cookie
    let token = jar
        .get(AUTH_COOKIE_NAME)
        .map(|cookie| cookie.value())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    state.auth_token.validate(token)?;

    let remote = state.remotes.get(&remote_id).ok_or(StatusCode::NOT_FOUND)?;

    let worker = remote.worker.clone();
    let connection_count = remote.connection_count.clone();

    Ok(ws.on_upgrade(move |socket| handle_websocket(socket, worker, connection_count)))
}

async fn handle_websocket(
    socket: WebSocket,
    worker: LuaWorker,
    connection_count: Arc<AtomicUsize>,
) {
    // Increment connection count and trigger focus if this is the first connection
    let prev_count = connection_count.fetch_add(1, Ordering::SeqCst);
    if prev_count == 0 {
        if let Err(error) = worker.trigger_event("focus").await {
            tracing::warn!("failed to trigger focus event: {error}");
        }
    }

    let (tx, rx) = socket.split();

    let mut send_task = tokio::spawn(handle_outgoing_messages(tx, worker.subscribe()));
    let mut recv_task = tokio::spawn(handle_incoming_messages(worker.clone(), rx));

    // Wait for either task to finish
    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    }

    // Decrement connection count and trigger blur if this was the last connection
    let new_count = connection_count.fetch_sub(1, Ordering::SeqCst) - 1;
    if new_count == 0 {
        if let Err(error) = worker.trigger_event("blur").await {
            tracing::warn!("failed to trigger blur event: {error}");
        }
    }
}

async fn handle_outgoing_messages(
    mut sender: SplitSink<WebSocket, Message>,
    receiver: Receiver<ServerMessage>,
) {
    while let Ok(msg) = receiver.recv_async().await {
        let json = match serde_json::to_string(&msg) {
            Ok(json) => json,
            Err(error) => {
                tracing::error!("failed to serialize server message: {error}");
                continue;
            }
        };

        if sender.send(Message::Text(json.into())).await.is_err() {
            break;
        }
    }
}

async fn handle_incoming_messages(worker: LuaWorker, mut receiver: SplitStream<WebSocket>) {
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                let client_msg: ClientMessage = match serde_json::from_str(&text) {
                    Ok(msg) => msg,
                    Err(error) => {
                        tracing::error!("failed to parse client message: {error}");
                        continue;
                    }
                };

                match client_msg {
                    ClientMessage::CallAction(request) => {
                        if let Err(error) = worker.send(request).await {
                            tracing::error!("failed to send action to worker: {error}");
                        }
                    }
                }
            }
            Ok(Message::Close(_)) => break,
            Ok(_) => {}
            Err(error) => {
                tracing::error!("websocket error: {error}");
                break;
            }
        }
    }
}
