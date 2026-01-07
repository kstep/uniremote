use std::sync::Arc;

use axum::{
    extract::{
        Path, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    http::StatusCode,
    response::Response,
};
use axum_extra::TypedHeader;
use flume::Receiver;
use futures_util::{
    sink::SinkExt,
    stream::{SplitSink, SplitStream, StreamExt},
};
use headers::{Header, HeaderName, HeaderValue};
use uniremote_core::{ClientMessage, RemoteId, ServerMessage};
use uniremote_lua::LuaWorker;

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
    TypedHeader(protocol): TypedHeader<SecWebSocketProtocol>,
    ws: WebSocketUpgrade,
) -> Result<Response, StatusCode> {
    // Extract token from Sec-WebSocket-Protocol header (format: "bearer.{token}")
    let token = protocol.bearer_token().ok_or(StatusCode::UNAUTHORIZED)?;

    state.auth_token.validate(token)?;

    let remote = state.remotes.get(&remote_id).ok_or(StatusCode::NOT_FOUND)?;

    let worker = remote.worker.clone();
    Ok(ws
        .protocols([format!("bearer.{token}")])
        .on_upgrade(move |socket| handle_websocket(socket, worker)))
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
    receiver: Receiver<ServerMessage>,
) {
    while let Ok(msg) = receiver.recv_async().await {
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
}

async fn handle_incoming_messages(worker: LuaWorker, mut receiver: SplitStream<WebSocket>) {
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
                        if let Err(error) = worker.send(request).await {
                            tracing::error!("failed to send action to worker: {error}");
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
}
