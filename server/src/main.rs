use clap::Parser;
use tokio::sync::broadcast;
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

    // Create broadcast channel for server-to-client messages
    let (broadcast_tx, _) = broadcast::channel(BROADCAST_CHANNEL_SIZE);

    // Add broadcast sender to each Lua state
    for lua_state in lua_states.values() {
        lua_state.add_state(broadcast_tx.clone());
    }

    let (tx, rx) = tokio::sync::mpsc::channel(WORKER_CHANNEL_SIZE);
    let worker = tokio::spawn(uniremote_lua::run(rx, lua_states));

    uniremote_server::run(tx, remotes, args.bind, broadcast_tx).await?;
    worker.await?;

    Ok(())
}
