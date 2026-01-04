use std::{collections::HashMap, net::SocketAddr, ops::Range, sync::Arc};

use anyhow::Context;
use axum::{Router, routing::get};
use tokio::net::TcpListener;
use tower_http::services::ServeDir;
use uniremote_core::{Remote, RemoteId};

mod handlers;

const LISTEN_PORT_RANGE: Range<u16> = 8000..8101;
const ASSETS_DIR: &str = "server/assets";

struct AppState {
    remotes: HashMap<RemoteId, Remote>,
}

pub async fn run(remotes: HashMap<RemoteId, Remote>) -> anyhow::Result<()> {
    let state = Arc::new(AppState { remotes });

    let app = Router::new()
        .route("/", get(handlers::list_remotes))
        .route("/r/{id}", get(handlers::get_remote))
        .nest_service("/assets", ServeDir::new(ASSETS_DIR))
        .with_state(state);

    let listener = bind_lan_port(LISTEN_PORT_RANGE)
        .await
        .context("failed to bind to lan port")?;

    let local_addr = listener.local_addr()?;
    tracing::info!("server listening on {local_addr}");
    print_qr_code(local_addr);

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

pub fn print_qr_code(addr: SocketAddr) {
    let url = format!("http://{}", addr);

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
