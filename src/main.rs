use clap::Parser;
use runtipi_fleet_dashboard::{cache::CacheStore, config::FleetConfig, runner::run_once};
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
struct Args {
    #[arg(long, env = "FLEET_CONFIG", default_value = "/config/fleet.yaml")]
    config: PathBuf,
    #[arg(long, env = "FLEET_STATE_DIR", default_value = "/state")]
    state_dir: PathBuf,
    #[arg(
        long,
        env = "HOMEPAGE_SERVICES",
        default_value = "/homepage/services.yaml"
    )]
    output: PathBuf,
    #[arg(long)]
    once: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();
    let args = Args::parse();
    let config = FleetConfig::load(&args.config).await?;
    let cache = CacheStore::new(args.state_dir.join("inventories"));
    loop {
        if let Err(error) = run_once(&config, &cache, &args.output).await {
            tracing::error!(%error, "poll cycle failed");
        }
        if args.once {
            break;
        }
        tokio::select! {
            _ = tokio::time::sleep(config.poll_interval()) => {},
            _ = tokio::signal::ctrl_c() => break,
        }
    }
    Ok(())
}
