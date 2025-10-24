use agent_client_protocol::{
    CreateTerminalRequest, CreateTerminalResponse, KillTerminalCommandRequest,
    KillTerminalCommandResponse, ReadTextFileRequest, ReadTextFileResponse, ReleaseTerminalRequest,
    ReleaseTerminalResponse, RequestPermissionRequest, RequestPermissionResponse,
    TerminalOutputRequest, TerminalOutputResponse, WaitForTerminalExitRequest,
    WaitForTerminalExitResponse, WriteTextFileRequest, WriteTextFileResponse,
};

use crate::jsonrpc::{JsonRpcIncomingMessage, JsonRpcOutgoingMessage, JsonRpcRequest};
use crate::util::json_cast;

// Agent -> Client requests
// These are messages that agents send to clients/editors

// ============================================================================
// RequestPermissionRequest
// ============================================================================

impl JsonRpcOutgoingMessage for RequestPermissionRequest {
    fn method(&self) -> &str {
        "session/request_permission"
    }

    fn params(&self) -> Result<Option<jsonrpcmsg::Params>, jsonrpcmsg::Error> {
        json_cast(self)
    }
}

impl JsonRpcRequest for RequestPermissionRequest {
    type Response = RequestPermissionResponse;
}

impl JsonRpcIncomingMessage for RequestPermissionResponse {
    fn from_value(_method: &str, value: serde_json::Value) -> Result<Self, jsonrpcmsg::Error> {
        json_cast(&value)
    }
}

// ============================================================================
// WriteTextFileRequest
// ============================================================================

impl JsonRpcOutgoingMessage for WriteTextFileRequest {
    fn method(&self) -> &str {
        "fs/write_text_file"
    }

    fn params(&self) -> Result<Option<jsonrpcmsg::Params>, jsonrpcmsg::Error> {
        json_cast(self)
    }
}

impl JsonRpcRequest for WriteTextFileRequest {
    type Response = WriteTextFileResponse;
}

impl JsonRpcIncomingMessage for WriteTextFileResponse {
    fn from_value(_method: &str, value: serde_json::Value) -> Result<Self, jsonrpcmsg::Error> {
        json_cast(&value)
    }
}

// ============================================================================
// ReadTextFileRequest
// ============================================================================

impl JsonRpcOutgoingMessage for ReadTextFileRequest {
    fn method(&self) -> &str {
        "fs/read_text_file"
    }

    fn params(&self) -> Result<Option<jsonrpcmsg::Params>, jsonrpcmsg::Error> {
        json_cast(self)
    }
}

impl JsonRpcRequest for ReadTextFileRequest {
    type Response = ReadTextFileResponse;
}

impl JsonRpcIncomingMessage for ReadTextFileResponse {
    fn from_value(_method: &str, value: serde_json::Value) -> Result<Self, jsonrpcmsg::Error> {
        json_cast(&value)
    }
}

// ============================================================================
// CreateTerminalRequest
// ============================================================================

impl JsonRpcOutgoingMessage for CreateTerminalRequest {
    fn method(&self) -> &str {
        "terminal/create"
    }

    fn params(&self) -> Result<Option<jsonrpcmsg::Params>, jsonrpcmsg::Error> {
        json_cast(self)
    }
}

impl JsonRpcRequest for CreateTerminalRequest {
    type Response = CreateTerminalResponse;
}

impl JsonRpcIncomingMessage for CreateTerminalResponse {
    fn from_value(_method: &str, value: serde_json::Value) -> Result<Self, jsonrpcmsg::Error> {
        json_cast(&value)
    }
}

// ============================================================================
// TerminalOutputRequest
// ============================================================================

impl JsonRpcOutgoingMessage for TerminalOutputRequest {
    fn method(&self) -> &str {
        "terminal/output"
    }

    fn params(&self) -> Result<Option<jsonrpcmsg::Params>, jsonrpcmsg::Error> {
        json_cast(self)
    }
}

impl JsonRpcRequest for TerminalOutputRequest {
    type Response = TerminalOutputResponse;
}

impl JsonRpcIncomingMessage for TerminalOutputResponse {
    fn from_value(_method: &str, value: serde_json::Value) -> Result<Self, jsonrpcmsg::Error> {
        json_cast(&value)
    }
}

// ============================================================================
// ReleaseTerminalRequest
// ============================================================================

impl JsonRpcOutgoingMessage for ReleaseTerminalRequest {
    fn method(&self) -> &str {
        "terminal/release"
    }

    fn params(&self) -> Result<Option<jsonrpcmsg::Params>, jsonrpcmsg::Error> {
        json_cast(self)
    }
}

impl JsonRpcRequest for ReleaseTerminalRequest {
    type Response = ReleaseTerminalResponse;
}

impl JsonRpcIncomingMessage for ReleaseTerminalResponse {
    fn from_value(_method: &str, value: serde_json::Value) -> Result<Self, jsonrpcmsg::Error> {
        json_cast(&value)
    }
}

// ============================================================================
// WaitForTerminalExitRequest
// ============================================================================

impl JsonRpcOutgoingMessage for WaitForTerminalExitRequest {
    fn method(&self) -> &str {
        "terminal/wait_for_exit"
    }

    fn params(&self) -> Result<Option<jsonrpcmsg::Params>, jsonrpcmsg::Error> {
        json_cast(self)
    }
}

impl JsonRpcRequest for WaitForTerminalExitRequest {
    type Response = WaitForTerminalExitResponse;
}

impl JsonRpcIncomingMessage for WaitForTerminalExitResponse {
    fn from_value(_method: &str, value: serde_json::Value) -> Result<Self, jsonrpcmsg::Error> {
        json_cast(&value)
    }
}

// ============================================================================
// KillTerminalCommandRequest
// ============================================================================

impl JsonRpcOutgoingMessage for KillTerminalCommandRequest {
    fn method(&self) -> &str {
        "terminal/kill"
    }

    fn params(&self) -> Result<Option<jsonrpcmsg::Params>, jsonrpcmsg::Error> {
        json_cast(self)
    }
}

impl JsonRpcRequest for KillTerminalCommandRequest {
    type Response = KillTerminalCommandResponse;
}

impl JsonRpcIncomingMessage for KillTerminalCommandResponse {
    fn from_value(_method: &str, value: serde_json::Value) -> Result<Self, jsonrpcmsg::Error> {
        json_cast(&value)
    }
}
