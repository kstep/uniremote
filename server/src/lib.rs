use std::{collections::HashMap, net::{IpAddr, Ipv4Addr, SocketAddr}, ops::Range, sync::Arc};

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

    let listener = bind_address(&bind_addr)
        .await
        .context("failed to bind to address")?;

    let local_addr = listener.local_addr()?;
    tracing::info!("server listening on {local_addr}");
    
    // Only print QR code in LAN mode
    if bind_addr.is_lan() {
        print_qr_code(local_addr, &auth_token);
    }

    axum::serve(listener, app).await?;

    Ok(())
}

async fn bind_address(bind_addr: &BindAddress) -> Option<TcpListener> {
    match bind_addr {
        BindAddress::Ip { ip, port_range } => {
            bind_to_ip_port(*ip, port_range.clone()).await
        }
        BindAddress::Localhost { port_range } => {
            let localhost = IpAddr::V4(Ipv4Addr::LOCALHOST);
            bind_to_ip_port(localhost, port_range.clone()).await
        }
        BindAddress::Lan { port_range } => {
            bind_lan_port(port_range.clone()).await
        }
    }
}

async fn bind_to_ip_port(ip: IpAddr, port_range: Range<u16>) -> Option<TcpListener> {
    for port in port_range {
        let addr = SocketAddr::new(ip, port);
        let Ok(listener) = TcpListener::bind(addr).await else {
            continue;
        };
        return Some(listener);
    }
    None
}

async fn bind_lan_port(port_range: Range<u16>) -> Option<TcpListener> {
    let ip = local_ip_address::local_ip().ok()?;

    if ip.is_loopback() {
        return None;
    }

    bind_to_ip_port(ip, port_range).await
}
