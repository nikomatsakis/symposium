//! Integration tests for MCP tool routing through proxy components.
//!
//! These tests verify that:
//! 1. Proxy components can provide MCP tools
//! 2. Agent components can discover and invoke those tools
//! 3. Tool invocations route correctly through the proxy

mod mcp_integration;

use agent_client_protocol::{self as acp};
use conductor::component::ComponentProvider;
use conductor::conductor::Conductor;
use scp::JsonRpcConnection;

use tokio::io::duplex;
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

async fn run_test_with_components(
    components: Vec<Box<dyn ComponentProvider>>,
    editor_task: impl AsyncFnOnce(scp::JsonRpcConnectionCx) -> Result<(), acp::Error>,
) -> Result<(), acp::Error> {
    // Set up editor <-> conductor communication
    let (editor_out, conductor_in) = duplex(1024);
    let (conductor_out, editor_in) = duplex(1024);

    JsonRpcConnection::new(editor_out.compat_write(), editor_in.compat())
        .name("editor-to-connector")
        .with_spawned(async move {
            Conductor::run(
                conductor_out.compat_write(),
                conductor_in.compat(),
                components,
            )
            .await
        })
        .with_client(editor_task)
        .await
}

#[tokio::test]
async fn test_proxy_provides_mcp_tools() -> Result<(), acp::Error> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("conductor=debug".parse().unwrap()),
        )
        .with_test_writer()
        .try_init();

    run_test_with_components(
        vec![
            mcp_integration::proxy::create(),
            mcp_integration::agent::create(),
        ],
        async |_editor_cx| {
            // TODO: Send initialization, session/new, and verify agent can see MCP tools
            Ok(())
        },
    )
    .await?;

    Ok(())
}
