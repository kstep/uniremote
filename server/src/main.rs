use clap::Parser;
use std::collections::HashMap;
use tokio::sync::broadcast;
use uniremote_core::ServerMessage;
use uniremote_server::args::Args;

const WORKER_CHANNEL_SIZE: usize = 100;
const BROADCAST_CHANNEL_SIZE: usize = 100;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let (remotes, lua_states) = uniremote_loader::load_remotes(args.remotes)?;

    tracing::info!("loaded {} remotes", remotes.len());

    // Create separate broadcast channel for each remote
    let mut broadcast_channels = HashMap::new();
    for (remote_id, lua_state) in &lua_states {
        let (broadcast_tx, _) = broadcast::channel::<ServerMessage>(BROADCAST_CHANNEL_SIZE);
        lua_state.add_state(broadcast_tx.clone());
        broadcast_channels.insert(remote_id.clone(), broadcast_tx);
    }

    let (tx, rx) = tokio::sync::mpsc::channel(WORKER_CHANNEL_SIZE);
    let worker = tokio::spawn(uniremote_lua::run(rx, lua_states));

    uniremote_server::run(tx, remotes, args.bind, broadcast_channels).await?;
    worker.await?;

    Ok(())
}
