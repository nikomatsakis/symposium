use std::{collections::HashMap, net::SocketAddr};

use agent_client_protocol as acp;
use agent_client_protocol::McpServer;
use futures::{SinkExt, StreamExt as _, channel::mpsc};
use scp::{JsonRpcConnection, JsonRpcConnectionCx, JsonRpcRequestCx, UntypedMessage};
use tokio::net::TcpStream;
use tokio_util::compat::{TokioAsyncReadCompatExt as _, TokioAsyncWriteCompatExt as _};
use tracing::info;

use crate::conductor::ConductorMessage;

pub struct McpBridger {
    /// Mapping of acp:$UUID URLs to TCP bridge information for MCP message routing
    mcp_bridges: HashMap<String, McpBridgeInfo>,

    conductor_tx: mpsc::Sender<ConductorMessage>,
}

#[derive(Copy, Clone, Debug)]
pub struct McpPort {
    tcp_port: u16,
}

/// Information about an MCP bridge for routing messages.
///
/// When a component provides an MCP server with ACP transport (`acp:$UUID`),
/// and the agent lacks native `mcp_acp_transport` support, the conductor
/// spawns a TCP listener and transforms the server spec to use stdio transport.
#[derive(Clone, Debug)]
struct McpBridgeInfo {
    /// The original acp:$UUID URL from the MCP server specification
    acp_url: String,

    /// The TCP port we bound for this bridge
    tcp_port: McpPort,

    /// Send outgoing messages to the bridge
    bridge_tx: mpsc::Sender<McpBridgeMessage>,
}

enum McpBridgeMessage {
    Request(UntypedMessage, JsonRpcRequestCx<serde_json::Value>),
    Notification(UntypedMessage),
}

impl McpBridger {
    /// Transforms MCP servers with `acp:$UUID` URLs for agents that need bridging.
    ///
    /// For each MCP server with an `acp:` URL:
    /// 1. Spawns a TCP listener on an ephemeral port
    /// 2. Stores the mapping for message routing
    /// 3. Transforms the server to use stdio transport pointing to `conductor mcp $PORT`
    ///
    /// Returns the modified NewSessionRequest with transformed MCP servers.
    async fn transform_mcp_servers(
        &mut self,
        cx: &JsonRpcConnectionCx,
        mcp_server: &mut McpServer,
        conductor_tx: &mpsc::Sender<ConductorMessage>,
    ) -> Result<(), acp::Error> {
        use agent_client_protocol::McpServer;

        let McpServer::Http { name, url, headers } = mcp_server else {
            return Ok(());
        };

        if !url.starts_with("acp:") {
            return Ok(());
        }

        if !headers.is_empty() {
            return Err(acp::Error::internal_error());
        }

        info!(
            server_name = name,
            acp_url = url,
            "Detected MCP server with ACP transport, spawning TCP bridge"
        );

        // Spawn TCP listener on ephemeral port
        let tcp_port = self
            .spawn_tcp_listener(cx, &url, conductor_tx.clone())
            .await?;

        info!(
            server_name = name,
            acp_url = url,
            tcp_port.tcp_port,
            "Spawned TCP listener for MCP bridge"
        );

        // Transform to stdio transport pointing to conductor mcp process
        let transformed = McpServer::Stdio {
            name: name.clone(),
            command: std::path::PathBuf::from("conductor"),
            args: vec!["mcp".to_string(), tcp_port.tcp_port.to_string()],
            env: vec![],
        };
        *mcp_server = transformed;

        Ok(())
    }

    /// Spawns a TCP listener for an MCP bridge and stores the mapping.
    ///
    /// Binds to `localhost:0` to get an ephemeral port, then stores the
    /// `acp_url → tcp_port` mapping in `self.mcp_bridges`.
    ///
    /// Returns the bound port number.
    async fn spawn_tcp_listener(
        &mut self,
        cx: &JsonRpcConnectionCx,
        acp_url: &String,
        conductor_tx: mpsc::Sender<ConductorMessage>,
    ) -> anyhow::Result<McpPort> {
        use tokio::net::TcpListener;

        // Bind to ephemeral port
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let tcp_port = McpPort {
            tcp_port: listener.local_addr()?.port(),
        };

        info!(
            acp_url = acp_url,
            tcp_port.tcp_port, "Bound TCP listener for MCP bridge"
        );

        let (bridge_tx, bridge_rx) = mpsc::channel(128);

        // Store mapping for message routing (Phase 2b/3)
        self.mcp_bridges.insert(
            acp_url.clone(),
            McpBridgeInfo {
                acp_url: acp_url.clone(),
                tcp_port,
                bridge_tx,
            },
        );

        // Phase 2b: Accept connections from `conductor mcp $PORT`
        cx.spawn({
            let acp_url = acp_url.clone();
            async move {
                info!(
                    acp_url = acp_url,
                    tcp_port.tcp_port, "Waiting for bridge connection"
                );

                // Accept connections
                let (stream, addr) = listener
                    .accept()
                    .await
                    .map_err(acp::Error::into_internal_error)?;

                bridge_actor(acp_url, stream, addr, conductor_tx, bridge_rx).await
            }
        });

        Ok(tcp_port)
    }
}

async fn bridge_actor(
    acp_url: String,
    stream: TcpStream,
    addr: SocketAddr,
    conductor_tx: mpsc::Sender<ConductorMessage>,
    mut bridge_rx: mpsc::Receiver<McpBridgeMessage>,
) -> Result<(), acp::Error> {
    info!(
        acp_url,
        bridge_addr = ?addr,
        "Bridge connected"
    );

    let (read_half, write_half) = stream.into_split();

    // Establish bidirectional JSON-RPC connection
    // The bridge will send MCP requests (tools/call, etc.) to the conductor
    // The conductor can also send responses back
    JsonRpcConnection::new(write_half.compat_write(), read_half.compat())
        .on_receive(scp::GenericHandler::send_to({
            let mut conductor_tx = conductor_tx.clone();
            let acp_url = acp_url.clone();
            async move |method, params, response_cx| {
                info!(
                    method = method,
                    acp_url, "Received request from bridge, forwarding to proxy"
                );

                // Forward the MCP request to the proxy via conductor
                conductor_tx
                    .send(ConductorMessage::McpRequestReceived {
                        acp_url: acp_url.clone(),
                        method,
                        params,
                        response_cx,
                    })
                    .await
            }
        }))
        .with_client(async move |bridge_cx| {
            while let Some(message) = bridge_rx.next().await {
                match message {
                    McpBridgeMessage::Request(untyped_message, json_rpc_request_cx) => {
                        bridge_cx
                            .send_request(untyped_message)
                            .forward_to(conductor_tx.clone())
                            .await
                    }
                    McpBridgeMessage::Notification(untyped_message) => todo!(),
                }
            }
            Ok(())
        })
        .await
}
