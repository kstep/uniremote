use std::{collections::HashMap, net::SocketAddr, ops::Range, sync::Arc};

use anyhow::Context;
use axum::{
    Router,
    routing::{get, post},
};
use tokio::{net::TcpListener, sync::mpsc::Sender};
use tower_http::services::ServeDir;
use uniremote_core::{CallActionRequest, Remote, RemoteId};

mod auth;
mod handlers;

use auth::AuthToken;

const LISTEN_PORT_RANGE: Range<u16> = 8000..8101;
const ASSETS_DIR: &str = "server/assets";

struct AppState {
    worker_tx: Sender<(RemoteId, CallActionRequest)>,
    remotes: HashMap<RemoteId, Remote>,
    auth_token: AuthToken,
}

pub async fn run(
    worker_tx: Sender<(RemoteId, CallActionRequest)>,
    remotes: HashMap<RemoteId, Remote>,
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

    let listener = bind_lan_port(LISTEN_PORT_RANGE)
        .await
        .context("failed to bind to lan port")?;

    let local_addr = listener.local_addr()?;
    tracing::info!("server listening on {local_addr}");
    print_qr_code(local_addr, &auth_token);

    axum::serve(listener, app).await?;

    Ok(())
}

async fn bind_lan_port(port_range: Range<u16>) -> Option<TcpListener> {
    let ip = local_ip_address::local_ip().ok()?;

    if ip.is_loopback() {
        return None;
    }

    for port in port_range {
        let addr = SocketAddr::new(ip, port);
        let Ok(listener) = TcpListener::bind(addr).await else {
            continue;
        };
        return Some(listener);
    }

    None
}

pub fn print_qr_code(addr: SocketAddr, auth_token: &AuthToken) {
    let url = format!("http://{}?token={}", addr, auth_token.as_str());

    match qrcode::QrCode::new(&url) {
        Ok(code) => {
            let string = code
                .render::<char>()
                .quiet_zone(false)
                .module_dimensions(2, 1)
                .build();
            println!("\n{}\n", string);
            println!("Scan QR code or visit: {}", url);
        }
        Err(error) => {
            tracing::warn!("failed to generate qr code: {error}");
            println!("Visit: {}", url);
        }
    }
}
