use clap::Parser;
use uniremote_server::args::Args;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let remotes = uniremote_loader::load_remotes(args.remotes)?;

    tracing::info!("loaded {} remotes", remotes.len());

    uniremote_server::run(remotes, args.bind).await?;

    Ok(())
}
