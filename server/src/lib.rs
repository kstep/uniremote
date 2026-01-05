use std::{collections::HashMap, sync::Arc};

use anyhow::Context;
use axum::{
    Router,
    routing::{get, post},
};
use tokio::sync::mpsc::Sender;
use tower_http::services::ServeDir;
use uniremote_core::{CallActionRequest, Remote, RemoteId};

mod auth;
mod handlers;
mod qr;

pub mod args;

use crate::{auth::AuthToken, qr::print_qr_code};
pub use crate::args::BindAddress;

const ASSETS_DIR: &str = "server/assets";

struct AppState {
    worker_tx: Sender<(RemoteId, CallActionRequest)>,
    remotes: HashMap<RemoteId, Remote>,
    auth_token: AuthToken,
}

pub async fn run(
    worker_tx: Sender<(RemoteId, CallActionRequest)>,
    remotes: HashMap<RemoteId, Remote>,
    bind_addr: BindAddress,
) -> anyhow::Result<()> {
    let auth_token = AuthToken::generate();
    let state = Arc::new(AppState {
        worker_tx,
        remotes,
        auth_token: auth_token.clone(),
    });

    let app = Router::new()
        .route("/", get(handlers::list_remotes))
        .route("/r/{id}", get(handlers::get_remote))
        .route("/api/r/{id}/call", post(handlers::call_remote_action))
        .nest_service("/assets", ServeDir::new(ASSETS_DIR))
        .with_state(state);

    let listener = bind_addr.bind()
        .await
        .context("failed to bind to address")?;

    let local_addr = listener.local_addr()?;
    tracing::info!("server listening on {local_addr}");
    
    // Only print QR code in LAN mode
    if matches!(bind_addr, BindAddress::Lan { .. }) {
        print_qr_code(local_addr, &auth_token);
    }

    axum::serve(listener, app).await?;

    Ok(())
}
