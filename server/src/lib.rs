use std::{collections::HashMap, sync::Arc};

use anyhow::Context;
use axum::{
    Router,
    http::{Method, header},
    routing::{get, post},
};
use tokio::sync::mpsc::Sender;
use tower_http::{
    cors::{AllowOrigin, CorsLayer},
    services::ServeDir,
    trace::TraceLayer,
};
use uniremote_core::{CallActionRequest, Remote, RemoteId, SseBroadcaster, SseMessage};

mod auth;
mod handlers;
mod qr;

pub mod args;

pub use crate::args::BindAddress;
use crate::{auth::AuthToken, qr::print_qr_code};

const ASSETS_DIR: &str = "server/assets";
const SSE_CHANNEL_SIZE: usize = 100;

struct AppState {
    worker_tx: Sender<(RemoteId, CallActionRequest)>,
    remotes: HashMap<RemoteId, Remote>,
    auth_token: AuthToken,
    sse_tx: SseBroadcaster,
}

pub fn create_sse_broadcaster() -> SseBroadcaster {
    let (tx, _) = tokio::sync::broadcast::channel(SSE_CHANNEL_SIZE);
    tx
}

pub async fn run(
    worker_tx: Sender<(RemoteId, CallActionRequest)>,
    remotes: HashMap<RemoteId, Remote>,
    bind_addr: BindAddress,
    sse_tx: SseBroadcaster,
) -> anyhow::Result<()> {
    let auth_token = AuthToken::generate();

    let listener = bind_addr
        .bind()
        .await
        .context("failed to bind to address")?;

    let local_addr = listener.local_addr()?;
    let origin = format!("http://{local_addr}");

    print_qr_code(local_addr, &auth_token);

    let state = Arc::new(AppState {
        worker_tx,
        remotes,
        auth_token,
        sse_tx,
    });

    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::exact(origin.parse().unwrap()))
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION, header::ACCEPT]);

    let app = Router::new()
        .route("/", get(handlers::list_remotes))
        .route("/r/{id}", get(handlers::get_remote))
        .route("/api/r/{id}/call", post(handlers::call_remote_action))
        .route("/api/r/{id}/events", get(handlers::sse_handler))
        .nest_service("/assets", ServeDir::new(ASSETS_DIR))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state);

    tracing::info!("server listening on {origin}");
    axum::serve(listener, app).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sse_message_serialization() {
        let message = SseMessage {
            action: "update".to_string(),
            args: serde_json::json!({
                "id": "widget-id",
                "text": "new text"
            }),
        };

        let json = serde_json::to_string(&message).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["action"], "update");
        assert_eq!(parsed["args"]["id"], "widget-id");
        assert_eq!(parsed["args"]["text"], "new text");
    }

    #[test]
    fn test_sse_message_with_multiple_properties() {
        let message = SseMessage {
            action: "update".to_string(),
            args: serde_json::json!({
                "id": "slider-1",
                "progress": 75,
                "visibility": "visible"
            }),
        };

        let json = serde_json::to_string(&message).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["action"], "update");
        assert_eq!(parsed["args"]["id"], "slider-1");
        assert_eq!(parsed["args"]["progress"], 75);
        assert_eq!(parsed["args"]["visibility"], "visible");
    }
}
