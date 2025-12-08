//! Symposium ACP Proxy - Main entry point

use sacp::Component;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    symposium_acp_proxy::Symposium::new()
        .serve(sacp_tokio::Stdio::new())
        .await?;
    Ok(())
}
