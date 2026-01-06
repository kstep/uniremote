use std::sync::Arc;

use axum::{
    extract::{
        Path, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::Response,
};
use axum_extra::TypedHeader;
use futures_util::{sink::SinkExt, stream::StreamExt};
use headers::{Header, HeaderName, HeaderValue};
use uniremote_core::{ClientMessage, RemoteId};

use crate::AppState;

/// Typed header for Sec-WebSocket-Protocol
#[derive(Debug, Clone)]
pub struct SecWebSocketProtocol(String);

impl Header for SecWebSocketProtocol {
    fn name() -> &'static HeaderName {
        static NAME: HeaderName = HeaderName::from_static("sec-websocket-protocol");
        &NAME
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
    where
        I: Iterator<Item = &'i HeaderValue>,
    {
        let value = values.next().ok_or_else(headers::Error::invalid)?;
        let s = value.to_str().map_err(|_| headers::Error::invalid())?;
        Ok(SecWebSocketProtocol(s.to_string()))
    }

    fn encode<E: Extend<HeaderValue>>(&self, values: &mut E) {
        if let Ok(value) = HeaderValue::from_str(&self.0) {
            values.extend(std::iter::once(value));
        }
    }
}

impl SecWebSocketProtocol {
    /// Extract the bearer token from the protocol string (format:
    /// "bearer.{token}")
    pub fn bearer_token(&self) -> Option<&str> {
        // Split by comma in case multiple protocols are specified
        self.0.split(',').find_map(|protocol| {
            let protocol = protocol.trim();
            protocol.strip_prefix("bearer.")
        })
    }
}

pub async fn websocket_handler(
    Path(remote_id): Path<RemoteId>,
    State(state): State<Arc<AppState>>,
    protocol: Option<TypedHeader<SecWebSocketProtocol>>,
    ws: WebSocketUpgrade,
) -> Result<Response, axum::http::StatusCode> {
    // Extract token from Sec-WebSocket-Protocol header (format: "bearer.{token}")
    let token = protocol
        .as_ref()
        .and_then(|TypedHeader(p)| p.bearer_token())
        .ok_or(axum::http::StatusCode::UNAUTHORIZED)?;

    // Validate token
    if !state.auth_token.validate(token) {
        return Err(axum::http::StatusCode::UNAUTHORIZED);
    }

    let _remote = state
        .remotes
        .get(&remote_id)
        .ok_or(axum::http::StatusCode::NOT_FOUND)?;

    // Accept the WebSocket with the same protocol to complete the handshake
    Ok(ws
        .protocols([format!("bearer.{token}")])
        .on_upgrade(move |socket| handle_websocket(socket, remote_id, state)))
}

async fn handle_websocket(socket: WebSocket, remote_id: RemoteId, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();

    // Get the broadcast channel for this specific remote from RemoteWithChannel
    let broadcast_tx = match state.remotes.get(&remote_id) {
        Some(remote_with_channel) => &remote_with_channel.broadcast_tx,
        None => {
            tracing::error!("no remote found for: {remote_id}");
            return;
        }
    };
    let mut broadcast_rx = broadcast_tx.subscribe();

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
                        ClientMessage::CallAction(request) => {
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
