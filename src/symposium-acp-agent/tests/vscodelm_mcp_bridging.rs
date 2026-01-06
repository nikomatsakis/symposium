//! Integration test for VS Code LM → MCP tool bridging.
//!
//! This test simulates the VS Code side of the vscodelm protocol to verify that:
//! 1. Tools provided in ProvideResponseRequest are passed to the session
//! 2. The vscode-tools MCP server is set up and accessible
//!
//! Note: This test uses elizacp which is a simple chatbot that doesn't actually
//! query MCP servers. The test verifies the plumbing is correct by checking
//! that the session is created successfully and responds.

use std::process::Stdio;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::time::timeout;

/// A simple JSON-RPC client for communicating with the vscodelm backend.
struct VscodeLmClient {
    child: Child,
    stdin: tokio::process::ChildStdin,
    reader: BufReader<tokio::process::ChildStdout>,
    next_id: u64,
}

impl VscodeLmClient {
    /// Spawn the symposium-acp-agent in vscodelm mode.
    fn spawn() -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_symposium-acp-agent"))
            .arg("--log=debug")
            .arg("vscodelm")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit()) // Show agent stderr for debugging
            .spawn()
            .expect("failed to spawn symposium-acp-agent");

        let stdin = child.stdin.take().expect("stdin not available");
        let stdout = child.stdout.take().expect("stdout not available");
        let reader = BufReader::new(stdout);

        Self {
            child,
            stdin,
            reader,
            next_id: 1,
        }
    }

    /// Send a JSON-RPC request and return the response.
    async fn send_request(&mut self, method: &str, params: Value) -> Value {
        let id = self.next_id;
        self.next_id += 1;

        let request = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params
        });

        let request_str = serde_json::to_string(&request).unwrap();
        self.stdin
            .write_all(request_str.as_bytes())
            .await
            .expect("failed to write request");
        self.stdin
            .write_all(b"\n")
            .await
            .expect("failed to write newline");
        self.stdin.flush().await.expect("failed to flush");

        // Read response
        self.read_response(id).await
    }

    /// Read a response with the given ID, collecting any notifications along the way.
    async fn read_response(&mut self, expected_id: u64) -> Value {
        loop {
            let mut line = String::new();
            self.reader
                .read_line(&mut line)
                .await
                .expect("failed to read line");

            if line.trim().is_empty() {
                continue;
            }

            let msg: Value = serde_json::from_str(&line).expect("failed to parse JSON");

            // Check if this is our response
            if let Some(id) = msg.get("id") {
                if id.as_u64() == Some(expected_id) {
                    return msg;
                }
            }

            // Otherwise it's a notification, continue reading
            eprintln!(
                "[test] notification while waiting for response: {}",
                line.trim()
            );
        }
    }

    /// Read messages until we see a notification with the given method, or timeout.
    async fn read_until_notification(
        &mut self,
        method: &str,
        duration: Duration,
    ) -> Result<Vec<Value>, &'static str> {
        let mut messages = Vec::new();

        let result = timeout(duration, async {
            loop {
                let mut line = String::new();
                self.reader
                    .read_line(&mut line)
                    .await
                    .expect("failed to read line");

                if line.trim().is_empty() {
                    continue;
                }

                eprintln!("[test] received: {}", line.trim());
                let msg: Value = serde_json::from_str(&line).expect("failed to parse JSON");

                // Check if this is the notification we're looking for
                let is_target = msg.get("method").and_then(|m| m.as_str()) == Some(method);
                messages.push(msg);

                if is_target {
                    return messages;
                }
            }
        })
        .await;

        match result {
            Ok(msgs) => Ok(msgs),
            Err(_) => Err("timeout waiting for notification"),
        }
    }

    /// Kill the child process.
    async fn kill(&mut self) {
        let _ = self.child.kill().await;
    }
}

/// Tool definition as sent from VS Code.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ToolDefinition {
    name: String,
    description: String,
    input_schema: Value,
}

/// Agent definition for the request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum AgentDefinition {
    Eliza { deterministic: bool },
}

/// Chat message.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Message {
    role: String,
    content: Vec<ContentPart>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ContentPart {
    Text { value: String },
}

/// Test that the vscodelm backend:
/// 1. Accepts ProvideResponseRequest with tools
/// 2. Creates a session with the MCP server set up
/// 3. Returns a response (even if elizacp doesn't use the tools)
///
/// The logs show that the MCP bridge is set up correctly:
/// - "setting initial VS Code tools tool_count=2"
/// - "Detected MCP server with ACP transport, spawning TCP bridge"
/// - "MCP bridge connected"
#[tokio::test]
async fn test_vscodelm_accepts_tools_and_responds() {
    let mut client = VscodeLmClient::spawn();

    // First, get model info to verify connection works
    let info_response = client
        .send_request("lm/provideLanguageModelChatInformation", json!({}))
        .await;
    assert!(
        info_response.get("result").is_some(),
        "Expected result in info response: {:?}",
        info_response
    );
    eprintln!("[test] got model info response");

    // Send a chat request with synthetic tools
    let tools = vec![
        ToolDefinition {
            name: "synthetic_read_file".to_string(),
            description: "Read a file from the filesystem".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" }
                },
                "required": ["path"]
            }),
        },
        ToolDefinition {
            name: "synthetic_write_file".to_string(),
            description: "Write content to a file".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "content": { "type": "string" }
                },
                "required": ["path", "content"]
            }),
        },
    ];

    let messages = vec![Message {
        role: "user".to_string(),
        content: vec![ContentPart::Text {
            value: "Hello, how are you?".to_string(),
        }],
    }];

    eprintln!("[test] sending chat request with {} tools", tools.len());

    let chat_response = client
        .send_request(
            "lm/provideLanguageModelChatResponse",
            json!({
                "modelId": "symposium-eliza",
                "messages": messages,
                "agent": AgentDefinition::Eliza { deterministic: true },
                "options": {
                    "tools": tools,
                    "toolMode": "auto"
                }
            }),
        )
        .await;

    // The response itself should succeed
    assert!(
        chat_response.get("result").is_some() || chat_response.get("error").is_none(),
        "Chat request failed: {:?}",
        chat_response
    );
    eprintln!("[test] got chat response, waiting for notifications");

    // Collect response parts and completion notification
    let notifications = client
        .read_until_notification("lm/responseComplete", Duration::from_secs(10))
        .await
        .expect("timeout waiting for response");

    eprintln!("[test] received {} messages total", notifications.len());

    // Check that we got some response parts
    let response_parts: Vec<_> = notifications
        .iter()
        .filter(|n| n.get("method").and_then(|m| m.as_str()) == Some("lm/responsePart"))
        .collect();

    eprintln!("[test] got {} response parts", response_parts.len());
    for part in &response_parts {
        if let Some(params) = part.get("params") {
            eprintln!(
                "[test] part: {}",
                serde_json::to_string_pretty(params).unwrap()
            );
        }
    }

    assert!(
        !response_parts.is_empty(),
        "Expected at least one response part"
    );

    // Check that we got a completion notification
    let complete = notifications
        .iter()
        .any(|n| n.get("method").and_then(|m| m.as_str()) == Some("lm/responseComplete"));

    assert!(complete, "Expected lm/responseComplete notification");

    // Verify the response contains text from Eliza
    let has_text = response_parts.iter().any(|part| {
        part.get("params")
            .and_then(|p| p.get("part"))
            .and_then(|p| p.get("value"))
            .and_then(|v| v.as_str())
            .map(|s| !s.is_empty())
            .unwrap_or(false)
    });
    assert!(has_text, "Expected response to contain text");

    client.kill().await;
}

/// Test that a second message in the same session works correctly.
/// This verifies that the session state is maintained and tools are
/// still available after the first turn.
#[tokio::test]
async fn test_vscodelm_multi_turn_conversation() {
    let mut client = VscodeLmClient::spawn();

    // Get model info first
    let info_response = client
        .send_request("lm/provideLanguageModelChatInformation", json!({}))
        .await;
    assert!(info_response.get("result").is_some());

    let tools = vec![ToolDefinition {
        name: "test_tool".to_string(),
        description: "A test tool".to_string(),
        input_schema: json!({"type": "object"}),
    }];

    // First turn
    let messages1 = vec![Message {
        role: "user".to_string(),
        content: vec![ContentPart::Text {
            value: "Hello".to_string(),
        }],
    }];

    let _response1 = client
        .send_request(
            "lm/provideLanguageModelChatResponse",
            json!({
                "modelId": "symposium-eliza",
                "messages": messages1,
                "agent": AgentDefinition::Eliza { deterministic: true },
                "options": { "tools": tools }
            }),
        )
        .await;

    let notifications1 = client
        .read_until_notification("lm/responseComplete", Duration::from_secs(10))
        .await
        .expect("timeout on first turn");

    assert!(
        notifications1
            .iter()
            .any(|n| n.get("method").and_then(|m| m.as_str()) == Some("lm/responseComplete")),
        "First turn should complete"
    );

    // Second turn - include previous messages
    let messages2 = vec![
        Message {
            role: "user".to_string(),
            content: vec![ContentPart::Text {
                value: "Hello".to_string(),
            }],
        },
        Message {
            role: "assistant".to_string(),
            content: vec![ContentPart::Text {
                value: "How are you today?".to_string(),
            }],
        },
        Message {
            role: "user".to_string(),
            content: vec![ContentPart::Text {
                value: "I am fine, thanks!".to_string(),
            }],
        },
    ];

    let _response2 = client
        .send_request(
            "lm/provideLanguageModelChatResponse",
            json!({
                "modelId": "symposium-eliza",
                "messages": messages2,
                "agent": AgentDefinition::Eliza { deterministic: true },
                "options": { "tools": tools }
            }),
        )
        .await;

    let notifications2 = client
        .read_until_notification("lm/responseComplete", Duration::from_secs(10))
        .await
        .expect("timeout on second turn");

    assert!(
        notifications2
            .iter()
            .any(|n| n.get("method").and_then(|m| m.as_str()) == Some("lm/responseComplete")),
        "Second turn should complete"
    );

    client.kill().await;
}
