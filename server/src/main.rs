use std::collections::HashMap;

use clap::Parser;
use tokio::sync::broadcast;
use uniremote_core::ServerMessage;
use uniremote_server::{RemoteWithChannel, args::Args};

const WORKER_CHANNEL_SIZE: usize = 100;
const BROADCAST_CHANNEL_SIZE: usize = 100;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let (remotes, lua_states) = uniremote_loader::load_remotes(args.remotes)?;

    tracing::info!("loaded {} remotes", remotes.len());

    // Create RemoteWithChannel for each remote with its own broadcast channel
    let mut remotes_with_channels = HashMap::new();
    for (remote_id, remote) in remotes {
        let (broadcast_tx, _) = broadcast::channel::<ServerMessage>(BROADCAST_CHANNEL_SIZE);

        // Add broadcast sender to the corresponding Lua state
        if let Some(lua_state) = lua_states.get(&remote_id) {
            lua_state.add_state(broadcast_tx.clone());
        }

        remotes_with_channels.insert(
            remote_id,
            RemoteWithChannel {
                remote,
                broadcast_tx,
            },
        );
    }

    let (tx, rx) = tokio::sync::mpsc::channel(WORKER_CHANNEL_SIZE);
    let worker = tokio::spawn(uniremote_lua::run(rx, lua_states));

    uniremote_server::run(tx, remotes_with_channels, args.bind).await?;
    worker.await?;

    Ok(())
}
