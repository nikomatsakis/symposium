use agent_client_protocol::{
    CreateTerminalRequest, CreateTerminalResponse, KillTerminalCommandRequest,
    KillTerminalCommandResponse, ReadTextFileRequest, ReadTextFileResponse, ReleaseTerminalRequest,
    ReleaseTerminalResponse, RequestPermissionRequest, RequestPermissionResponse,
    SessionNotification, TerminalOutputRequest, TerminalOutputResponse, WaitForTerminalExitRequest,
    WaitForTerminalExitResponse, WriteTextFileRequest, WriteTextFileResponse,
};

use crate::jsonrpc::JsonRpcCx;

mod notifications;
mod requests;

/// ACP client for making requests to the editor.
///
/// This wraps a `JsonRpcCx` and provides strongly-typed methods for sending
/// ACP requests and notifications to the editor (client). This is the interface
/// an agent uses to interact with the editor environment.
///
/// # Example
///
/// ```rust,ignore
/// // Inside your agent implementation
/// let acp_client = AcpClient::new(cx.clone());
///
/// // Request permission from user
/// let response = acp_client.request_permission(RequestPermissionRequest {
///     tool_call_id: "call_123".into(),
///     options: vec![/* ... */],
/// }).recv().await?;
///
/// // Send a session notification
/// acp_client.session_notification(SessionNotification {
///     session_id: "session_456".into(),
///     content: /* ... */,
/// })?;
/// ```
#[derive(Clone)]
pub struct AcpClient {
    cx: JsonRpcCx,
}

impl AcpClient {
    /// Create a new ACP client from a JSON-RPC context.
    pub fn new(cx: JsonRpcCx) -> Self {
        Self { cx }
    }

    /// Request permission from the user for a tool call operation.
    ///
    /// Called by the agent when it needs user authorization before executing
    /// a potentially sensitive operation.
    pub fn request_permission(
        &self,
        request: RequestPermissionRequest,
    ) -> crate::jsonrpc::JsonRpcResponse<RequestPermissionResponse> {
        self.cx.send_request(request)
    }

    /// Write content to a text file in the client's file system.
    pub fn write_text_file(
        &self,
        request: WriteTextFileRequest,
    ) -> crate::jsonrpc::JsonRpcResponse<WriteTextFileResponse> {
        self.cx.send_request(request)
    }

    /// Read content from a text file in the client's file system.
    pub fn read_text_file(
        &self,
        request: ReadTextFileRequest,
    ) -> crate::jsonrpc::JsonRpcResponse<ReadTextFileResponse> {
        self.cx.send_request(request)
    }

    /// Execute a command in a new terminal.
    pub fn create_terminal(
        &self,
        request: CreateTerminalRequest,
    ) -> crate::jsonrpc::JsonRpcResponse<CreateTerminalResponse> {
        self.cx.send_request(request)
    }

    /// Get the terminal output and exit status.
    pub fn terminal_output(
        &self,
        request: TerminalOutputRequest,
    ) -> crate::jsonrpc::JsonRpcResponse<TerminalOutputResponse> {
        self.cx.send_request(request)
    }

    /// Release a terminal (kills command if still running).
    pub fn release_terminal(
        &self,
        request: ReleaseTerminalRequest,
    ) -> crate::jsonrpc::JsonRpcResponse<ReleaseTerminalResponse> {
        self.cx.send_request(request)
    }

    /// Wait for the terminal command to exit and return its exit status.
    pub fn wait_for_terminal_exit(
        &self,
        request: WaitForTerminalExitRequest,
    ) -> crate::jsonrpc::JsonRpcResponse<WaitForTerminalExitResponse> {
        self.cx.send_request(request)
    }

    /// Kill the terminal command without releasing the terminal.
    pub fn kill_terminal_command(
        &self,
        request: KillTerminalCommandRequest,
    ) -> crate::jsonrpc::JsonRpcResponse<KillTerminalCommandResponse> {
        self.cx.send_request(request)
    }

    /// Send a session notification to the client.
    ///
    /// This is a notification (no response expected) that sends real-time updates
    /// about session progress, including message chunks, tool calls, and execution plans.
    pub fn session_notification(
        &self,
        notification: SessionNotification,
    ) -> Result<(), jsonrpcmsg::Error> {
        self.cx
            .send_notification::<SessionNotification>(notification)
    }
}
