use std::sync::Arc;

use axum::{
    extract::{
        Path, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    http::StatusCode,
    response::Response,
};
use axum_extra::extract::cookie::CookieJar;
use futures_util::{
    sink::SinkExt,
    stream::{SplitSink, SplitStream, StreamExt},
};
use uniremote_core::{ClientMessage, RemoteId};
use uniremote_worker::{LuaWorker, Subscription};

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

    Ok(ws.on_upgrade(move |socket| handle_websocket(socket, worker)))
}

async fn handle_websocket(socket: WebSocket, worker: LuaWorker) {
    let (tx, rx) = socket.split();

    let mut send_task = tokio::spawn(handle_outgoing_messages(tx, worker.subscribe()));
    let mut recv_task = tokio::spawn(handle_incoming_messages(worker, rx));

    // Wait for either task to finish
    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    }
}

async fn handle_outgoing_messages(
    mut sender: SplitSink<WebSocket, Message>,
    subscription: Subscription,
) {
    while let Ok(msg) = subscription.recv().await {
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
