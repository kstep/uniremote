use std::{collections::HashMap, sync::Arc};

use anyhow::Context;
use axum::{
    Router,
    http::{Method, header},
    routing::{get, post},
};
use tokio::sync::{broadcast, mpsc::Sender};
use tower_http::{
    cors::{AllowOrigin, CorsLayer},
    services::ServeDir,
    trace::TraceLayer,
};
use uniremote_core::{CallActionRequest, Remote, RemoteId, ServerMessage};

mod auth;
mod handlers;
mod qr;
mod websocket;

pub mod args;

pub use crate::args::BindAddress;
use crate::{auth::AuthToken, qr::print_qr_code};

const ASSETS_DIR: &str = "server/assets";

pub struct RemoteWithChannel {
    pub remote: Remote,
    pub broadcast_tx: broadcast::Sender<ServerMessage>,
}

pub struct AppState {
    worker_tx: Sender<(RemoteId, CallActionRequest)>,
    remotes: HashMap<RemoteId, RemoteWithChannel>,
    auth_token: AuthToken,
}

pub async fn run(
    worker_tx: Sender<(RemoteId, CallActionRequest)>,
    remotes: HashMap<RemoteId, RemoteWithChannel>,
    bind_addr: BindAddress,
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
    });

    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::exact(origin.parse().unwrap()))
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION, header::ACCEPT]);

    let app = Router::new()
        .route("/", get(handlers::list_remotes))
        .route("/r/{id}", get(handlers::get_remote))
        .route("/r/{id}/icon", get(handlers::get_remote_icon))
        .route("/api/r/{id}/call", post(handlers::call_remote_action))
        .route("/api/r/{id}/ws", get(websocket::websocket_handler))
        .nest_service("/assets", ServeDir::new(ASSETS_DIR))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state);

    tracing::info!("server listening on {origin}");
    axum::serve(listener, app).await?;

    Ok(())
}
