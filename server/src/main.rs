#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let remotes = uniremote_loader::load_remotes()?;

    tracing::info!("loaded {} remotes", remotes.len());

    uniremote_server::run(remotes).await
}
