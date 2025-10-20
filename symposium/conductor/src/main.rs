use clap::Parser;
use conductor::args::ConductorArgs;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = ConductorArgs::parse();
    conductor::run(args.proxies).await
}
