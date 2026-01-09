use clap::Parser;
use uniremote_loader::LuaLimits;
use uniremote_server::args::Args;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let lua_limits = LuaLimits {
        memory_mb: args.lua_max_mem,
        max_instructions: args.lua_max_instructions,
    };

    let remotes = uniremote_loader::load_remotes(args.remotes, lua_limits)?;

    tracing::info!("loaded {} remotes", remotes.len());

    uniremote_server::run(remotes, args.bind).await?;

    Ok(())
}
