use agent_client_protocol::{
    AuthenticateRequest, AuthenticateResponse, InitializeRequest, InitializeResponse,
    LoadSessionRequest, LoadSessionResponse, NewSessionRequest, NewSessionResponse, PromptRequest,
    PromptResponse, SetSessionModeRequest, SetSessionModeResponse,
};

use crate::jsonrpc::{JsonRpcIncomingMessage, JsonRpcOutgoingMessage, JsonRpcRequest};
use crate::util::json_cast;

// ============================================================================
// InitializeRequest
// ============================================================================

impl JsonRpcOutgoingMessage for InitializeRequest {
    fn method(&self) -> &str {
        "initialize"
    }

    fn params(&self) -> Result<Option<jsonrpcmsg::Params>, jsonrpcmsg::Error> {
        json_cast(self)
    }
}

impl JsonRpcRequest for InitializeRequest {
    type Response = InitializeResponse;
}

impl JsonRpcIncomingMessage for InitializeResponse {
    fn from_value(_method: &str, value: serde_json::Value) -> Result<Self, jsonrpcmsg::Error> {
        json_cast(&value)
    }
}

// ============================================================================
// AuthenticateRequest
// ============================================================================

impl JsonRpcOutgoingMessage for AuthenticateRequest {
    fn method(&self) -> &str {
        "authenticate"
    }

    fn params(&self) -> Result<Option<jsonrpcmsg::Params>, jsonrpcmsg::Error> {
        json_cast(self)
    }
}

impl JsonRpcRequest for AuthenticateRequest {
    type Response = AuthenticateResponse;
}

impl JsonRpcIncomingMessage for AuthenticateResponse {
    fn from_value(_method: &str, value: serde_json::Value) -> Result<Self, jsonrpcmsg::Error> {
        json_cast(&value)
    }
}

// ============================================================================
// LoadSessionRequest
// ============================================================================

impl JsonRpcOutgoingMessage for LoadSessionRequest {
    fn method(&self) -> &str {
        "session/load"
    }

    fn params(&self) -> Result<Option<jsonrpcmsg::Params>, jsonrpcmsg::Error> {
        json_cast(self)
    }
}

impl JsonRpcRequest for LoadSessionRequest {
    type Response = LoadSessionResponse;
}

impl JsonRpcIncomingMessage for LoadSessionResponse {
    fn from_value(_method: &str, value: serde_json::Value) -> Result<Self, jsonrpcmsg::Error> {
        json_cast(&value)
    }
}

// ============================================================================
// NewSessionRequest
// ============================================================================

impl JsonRpcOutgoingMessage for NewSessionRequest {
    fn method(&self) -> &str {
        "session/new"
    }

    fn params(&self) -> Result<Option<jsonrpcmsg::Params>, jsonrpcmsg::Error> {
        json_cast(self)
    }
}

impl JsonRpcRequest for NewSessionRequest {
    type Response = NewSessionResponse;
}

impl JsonRpcIncomingMessage for NewSessionResponse {
    fn from_value(_method: &str, value: serde_json::Value) -> Result<Self, jsonrpcmsg::Error> {
        json_cast(&value)
    }
}

// ============================================================================
// PromptRequest
// ============================================================================

impl JsonRpcOutgoingMessage for PromptRequest {
    fn method(&self) -> &str {
        "session/prompt"
    }

    fn params(&self) -> Result<Option<jsonrpcmsg::Params>, jsonrpcmsg::Error> {
        json_cast(self)
    }
}

impl JsonRpcRequest for PromptRequest {
    type Response = PromptResponse;
}

impl JsonRpcIncomingMessage for PromptResponse {
    fn from_value(_method: &str, value: serde_json::Value) -> Result<Self, jsonrpcmsg::Error> {
        json_cast(&value)
    }
}

// ============================================================================
// SetSessionModeRequest
// ============================================================================

impl JsonRpcOutgoingMessage for SetSessionModeRequest {
    fn method(&self) -> &str {
        "session/set_mode"
    }

    fn params(&self) -> Result<Option<jsonrpcmsg::Params>, jsonrpcmsg::Error> {
        json_cast(self)
    }
}

impl JsonRpcRequest for SetSessionModeRequest {
    type Response = SetSessionModeResponse;
}

impl JsonRpcIncomingMessage for SetSessionModeResponse {
    fn from_value(_method: &str, value: serde_json::Value) -> Result<Self, jsonrpcmsg::Error> {
        json_cast(&value)
    }
}
