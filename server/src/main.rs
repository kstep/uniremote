use clap::Parser;
use uniremote_core::SseBroadcaster;
use uniremote_server::args::Args;

const WORKER_CHANNEL_SIZE: usize = 100;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let (remotes, lua_states) = uniremote_loader::load_remotes(args.remotes)?;

    tracing::info!("loaded {} remotes", remotes.len());

    let (tx, rx) = tokio::sync::mpsc::channel(WORKER_CHANNEL_SIZE);
    let sse_tx: SseBroadcaster = uniremote_server::create_sse_broadcaster();
    
    let worker = tokio::spawn(uniremote_lua::run(rx, lua_states, sse_tx.clone()));

    uniremote_server::run(tx, remotes, args.bind, sse_tx).await?;
    worker.await?;

    Ok(())
}
