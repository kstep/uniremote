use clap::Parser;
use uniremote_server::args::Args;

const WORKER_CHANNEL_SIZE: usize = 100;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let (remotes, lua_states) = uniremote_loader::load_remotes()?;

    tracing::info!("loaded {} remotes", remotes.len());

    let (tx, rx) = tokio::sync::mpsc::channel(WORKER_CHANNEL_SIZE);
    let worker = tokio::spawn(uniremote_lua::run(rx, lua_states));

    uniremote_server::run(tx, remotes, args.bind).await?;
    worker.await?;

    Ok(())
}
