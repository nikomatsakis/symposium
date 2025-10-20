use crate::conductor::Conductor;

mod args;
mod component;
mod conductor;

pub async fn run(proxies: Vec<String>) -> anyhow::Result<()> {
    Conductor::run(proxies).await
}
