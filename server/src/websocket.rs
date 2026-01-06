use std::sync::Arc;

use axum::{
    extract::{
        Path, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::Response,
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use uniremote_core::{ActionId, CallActionRequest, RemoteId};

use crate::{AppState, auth::validate_token};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    #[serde(rename = "update")]
    Update {
        action: ActionId,
        args: serde_json::Value,
    },
    #[serde(rename = "error")]
    Error { message: String },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    #[serde(rename = "call")]
    CallAction {
        action: ActionId,
        #[serde(default)]
        args: Option<Vec<serde_json::Value>>,
    },
}

pub async fn websocket_handler(
    Path(remote_id): Path<RemoteId>,
    State(state): State<Arc<AppState>>,
    auth_header: Option<TypedHeader<Authorization<Bearer>>>,
    ws: WebSocketUpgrade,
) -> Result<Response, axum::http::StatusCode> {
    validate_token(auth_header, &state)?;

    let _remote = state
        .remotes
        .get(&remote_id)
        .ok_or(axum::http::StatusCode::NOT_FOUND)?;

    Ok(ws.on_upgrade(move |socket| handle_websocket(socket, remote_id, state)))
}

async fn handle_websocket(socket: WebSocket, remote_id: RemoteId, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();

    // Subscribe to broadcast channel for server-to-client messages
    let mut broadcast_rx = state.broadcast_tx.subscribe();

    // Spawn a task to forward broadcast messages to this WebSocket
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = broadcast_rx.recv().await {
            let json = match serde_json::to_string(&msg) {
                Ok(json) => json,
                Err(e) => {
                    tracing::error!("failed to serialize server message: {e}");
                    continue;
                }
            };

            if sender.send(Message::Text(json.into())).await.is_err() {
                break;
            }
        }
    });

    // Handle incoming messages from client
    let worker_tx = state.worker_tx.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    let client_msg: ClientMessage = match serde_json::from_str(&text) {
                        Ok(msg) => msg,
                        Err(e) => {
                            tracing::error!("failed to parse client message: {e}");
                            continue;
                        }
                    };

                    match client_msg {
                        ClientMessage::CallAction { action, args } => {
                            let request = CallActionRequest { action, args };

                            tracing::info!(
                                "websocket call action '{}' on remote '{remote_id}'",
                                request.action
                            );

                            if let Err(e) = worker_tx.send((remote_id.clone(), request)).await {
                                tracing::error!("failed to send action to worker: {e}");
                            }
                        }
                    }
                }
                Ok(Message::Close(_)) => break,
                Ok(_) => {}
                Err(e) => {
                    tracing::error!("websocket error: {e}");
                    break;
                }
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    }
}
