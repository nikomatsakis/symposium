//! MCP Bridge: Bridges MCP JSON-RPC over stdio to TCP connection
//!
//! This module implements `conductor mcp $port` mode, which acts as an MCP server
//! over stdio but forwards all messages to/from a TCP connection on localhost:$port.
//!
//! The main conductor (in agent mode) listens on the TCP port and translates between
//! TCP (raw JSON-RPC) and ACP `_mcp/*` extension messages.

use anyhow::{Context, Result};
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

/// Run the MCP bridge: stdio ↔ TCP
///
/// Reads MCP JSON-RPC messages from stdin, forwards to TCP connection.
/// Reads responses from TCP, writes to stdout.
pub async fn run_mcp_bridge(port: u16) -> Result<()> {
    tracing::info!("MCP bridge starting, connecting to localhost:{}", port);

    // Connect to the main conductor via TCP
    let stream = connect_with_retry(port).await?;
    let (tcp_read, mut tcp_write) = stream.into_split();

    // Set up stdio
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let mut stdin_reader = BufReader::new(stdin);
    let mut stdout_writer = stdout;
    let mut tcp_reader = BufReader::new(tcp_read);

    // Prepare line buffers
    let mut stdin_line = String::new();
    let mut tcp_line = String::new();

    tracing::info!("MCP bridge connected, starting message loop");

    loop {
        tokio::select! {
            // Read from stdin → send to TCP
            result = stdin_reader.read_line(&mut stdin_line) => {
                let n = result.context("Failed to read from stdin")?;

                if n == 0 {
                    tracing::info!("Stdin closed, shutting down bridge");
                    break;
                }

                // Parse to validate JSON
                let _: Value = serde_json::from_str(stdin_line.trim())
                    .context("Invalid JSON from stdin")?;

                tracing::debug!("Bridge: stdin → TCP: {}", stdin_line.trim());

                // Forward to TCP
                tcp_write.write_all(stdin_line.as_bytes()).await
                    .context("Failed to write to TCP")?;
                tcp_write.flush().await
                    .context("Failed to flush TCP")?;

                stdin_line.clear();
            }

            // Read from TCP → send to stdout
            result = tcp_reader.read_line(&mut tcp_line) => {
                let n = result.context("Failed to read from TCP")?;

                if n == 0 {
                    tracing::info!("TCP connection closed, shutting down bridge");
                    break;
                }

                // Parse to validate JSON
                let _: Value = serde_json::from_str(tcp_line.trim())
                    .context("Invalid JSON from TCP")?;

                tracing::debug!("Bridge: TCP → stdout: {}", tcp_line.trim());

                // Forward to stdout
                stdout_writer.write_all(tcp_line.as_bytes()).await
                    .context("Failed to write to stdout")?;
                stdout_writer.flush().await
                    .context("Failed to flush stdout")?;

                tcp_line.clear();
            }
        }
    }

    tracing::info!("MCP bridge shutting down");
    Ok(())
}

/// Connect to TCP port with retry logic
async fn connect_with_retry(port: u16) -> Result<TcpStream> {
    let max_retries = 10;
    let mut retry_delay_ms = 50;

    for attempt in 1..=max_retries {
        match TcpStream::connect(format!("127.0.0.1:{}", port)).await {
            Ok(stream) => {
                tracing::info!("Connected to localhost:{} on attempt {}", port, attempt);
                return Ok(stream);
            }
            Err(e) if attempt < max_retries => {
                tracing::debug!(
                    "Connection attempt {} failed: {}, retrying in {}ms",
                    attempt,
                    e,
                    retry_delay_ms
                );
                tokio::time::sleep(tokio::time::Duration::from_millis(retry_delay_ms)).await;
                retry_delay_ms = (retry_delay_ms * 2).min(1000); // Exponential backoff, max 1s
            }
            Err(e) => {
                return Err(e).context(format!(
                    "Failed to connect to localhost:{} after {} attempts",
                    port, max_retries
                ));
            }
        }
    }

    unreachable!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::AsyncWriteExt;
    use tokio::net::TcpListener;

    #[tokio::test]
    async fn test_connect_with_retry_success() {
        // Set up a mock TCP server
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        // Spawn a task that accepts the connection
        let accept_task = tokio::spawn(async move {
            listener.accept().await.unwrap();
        });

        // Test connection with retry
        let result = connect_with_retry(port).await;
        assert!(result.is_ok());

        accept_task.await.unwrap();
    }

    #[tokio::test]
    async fn test_connect_with_retry_eventual_success() {
        // Start with no listener, then start one after a delay
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        // Drop the listener to make the port unavailable
        drop(listener);

        // Spawn a task that will start listening after a short delay
        let delayed_server = tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
            let listener = TcpListener::bind(format!("127.0.0.1:{}", port))
                .await
                .unwrap();
            listener.accept().await.unwrap();
        });

        // This should retry and eventually succeed
        let result = connect_with_retry(port).await;
        assert!(result.is_ok());

        delayed_server.await.unwrap();
    }

    #[tokio::test]
    async fn test_message_bridging() {
        // Set up a mock TCP server
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        // Spawn mock TCP server that echoes with modification
        let server_task = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut reader = BufReader::new(&mut socket);
            let mut line = String::new();

            // Read message from bridge
            reader.read_line(&mut line).await.unwrap();

            // Parse and modify
            let mut msg: Value = serde_json::from_str(line.trim()).unwrap();
            if let Some(obj) = msg.as_object_mut() {
                obj.remove("id");
                obj.insert("result".to_string(), Value::String("pong".to_string()));
            }

            // Send back
            let response = format!("{}\n", serde_json::to_string(&msg).unwrap());
            socket.write_all(response.as_bytes()).await.unwrap();
            socket.flush().await.unwrap();

            // Keep connection open briefly
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        });

        // Create a connected stream to verify the TCP side works
        let stream = TcpStream::connect(format!("127.0.0.1:{}", port))
            .await
            .unwrap();
        let (tcp_read, mut tcp_write) = stream.into_split();
        let mut tcp_reader = BufReader::new(tcp_read);

        // Send a test message
        let test_msg = r#"{"jsonrpc":"2.0","id":"test-1","method":"tools/call"}"#;
        tcp_write
            .write_all(format!("{}\n", test_msg).as_bytes())
            .await
            .unwrap();
        tcp_write.flush().await.unwrap();

        // Read response
        let mut response_line = String::new();
        tcp_reader.read_line(&mut response_line).await.unwrap();

        // Verify response
        let response: Value = serde_json::from_str(response_line.trim()).unwrap();
        assert_eq!(response["result"], "pong");

        server_task.await.unwrap();
    }
}
