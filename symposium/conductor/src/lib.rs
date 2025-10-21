use crate::conductor::Conductor;

mod component;
mod conductor;

#[cfg(test)]
mod conductor_tests;

use clap::Parser;
use component::ComponentProvider;
use tokio::io::{stdin, stdout};
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct ConductorArgs {
    /// List of proxy commands to chain together
    pub proxies: Vec<String>,
}

impl ConductorArgs {
    pub async fn run(self) -> anyhow::Result<()> {
        let providers = self
            .proxies
            .into_iter()
            .map(ComponentProvider::Command)
            .collect();

        Conductor::run(stdout().compat_write(), stdin().compat(), providers).await
    }
}
