use clap::Parser;
use conductor::ConductorArgs;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing with env filter support (RUST_LOG=debug, etc.)
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("conductor=info")),
        )
        .with(tracing_subscriber::fmt::layer().with_target(true))
        .init();

    tracing::info!("Conductor starting");

    ConductorArgs::parse().run().await
}
