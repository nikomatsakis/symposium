//! Message types for proxy communication with successor components.
//!
//! These types wrap JSON-RPC messages for the `_proxy/successor/*` protocol.

use serde::{Deserialize, Serialize};

use crate::jsonrpc::{
    JsonRpcIncomingMessage, JsonRpcNotification, JsonRpcOutgoingMessage, JsonRpcRequest,
};
use crate::util::json_cast;

// ============================================================================
// Requests and notifications send TO successor (and the response we receieve)
// ============================================================================

/// A request being sent to the successor component.
///
/// Used in `_proxy/successor/send` when the proxy wants to forward a request downstream.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToSuccessorRequest<Req> {
    /// Name of the method to be invoked
    pub method: String,

    /// Parameters for the method invocation
    pub params: Req,
}

impl<Req: JsonRpcOutgoingMessage> JsonRpcOutgoingMessage for ToSuccessorRequest<Req> {
    fn method(&self) -> &str {
        "_proxy/successor/send/request"
    }

    fn params(&self) -> Result<Option<jsonrpcmsg::Params>, jsonrpcmsg::Error> {
        json_cast(self)
    }
}

impl<Req: JsonRpcRequest> JsonRpcRequest for ToSuccessorRequest<Req> {
    type Response = ToSuccessorResponse<Req::Response>;
}

/// A response received from a [`ToSuccessorRequest`].
///
/// Returned as the response to a `ToSuccessorRequest`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ToSuccessorResponse<Response> {
    /// Result of the method invocation (on success)
    Result(Response),

    /// Error object (on failure)
    Error(jsonrpcmsg::Error),
}

impl<Response: JsonRpcIncomingMessage> JsonRpcIncomingMessage for ToSuccessorResponse<Response> {
    fn from_value(_method: &str, value: serde_json::Value) -> Result<Self, jsonrpcmsg::Error> {
        json_cast(&value)
    }
}

impl<R> From<Result<R, jsonrpcmsg::Error>> for ToSuccessorResponse<R> {
    fn from(value: Result<R, jsonrpcmsg::Error>) -> Self {
        match value {
            Ok(v) => ToSuccessorResponse::Result(v),
            Err(e) => ToSuccessorResponse::Error(e),
        }
    }
}

/// A notification being sent to the successor component.
///
/// Used in `_proxy/successor/send` when the proxy wants to forward a notification downstream.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToSuccessorNotification<Req> {
    /// Name of the method to be invoked
    pub method: String,

    /// Parameters for the method invocation
    pub params: Req,
}

impl<Req: JsonRpcOutgoingMessage> JsonRpcOutgoingMessage for ToSuccessorNotification<Req> {
    fn method(&self) -> &str {
        "_proxy/successor/send/notification"
    }

    fn params(&self) -> Result<Option<jsonrpcmsg::Params>, jsonrpcmsg::Error> {
        json_cast(self)
    }
}

impl<Req: JsonRpcNotification> JsonRpcNotification for ToSuccessorNotification<Req> {}

// ============================================================================
// Messages FROM successor
// ============================================================================

/// A request received from the successor component.
///
/// Delivered via `_proxy/successor/receive` when the successor wants to call back upstream.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReceiveFromSuccessorRequest {
    /// Name of the method to be invoked
    pub method: String,

    /// Parameters for the method invocation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<jsonrpcmsg::Params>,
}

impl JsonRpcOutgoingMessage for ReceiveFromSuccessorRequest {
    fn method(&self) -> &str {
        "_proxy/successor/receive/request"
    }

    fn params(&self) -> Result<Option<jsonrpcmsg::Params>, jsonrpcmsg::Error> {
        json_cast(self)
    }
}

impl JsonRpcRequest for ReceiveFromSuccessorRequest {
    type Response = FromSuccessorResponse;
}

/// Response sent when we receive a [`FromSuccessorRequest`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FromSuccessorResponse {
    /// Result of the method invocation (on success)
    Result(serde_json::Value),

    /// Error object (on failure)
    Error(jsonrpcmsg::Error),
}

impl JsonRpcIncomingMessage for FromSuccessorResponse {
    fn from_value(_method: &str, value: serde_json::Value) -> Result<Self, jsonrpcmsg::Error> {
        json_cast(&value)
    }
}

/// A notification received from the successor component.
///
/// Delivered via `_proxy/successor/receive` when the successor sends a notification upstream.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FromSuccessorNotification {
    pub message: jsonrpcmsg::Request,
}

impl JsonRpcOutgoingMessage for FromSuccessorNotification {
    fn method(&self) -> &str {
        "_proxy/successor/receive/notification"
    }

    fn params(&self) -> Result<Option<jsonrpcmsg::Params>, jsonrpcmsg::Error> {
        json_cast(self)
    }
}

impl JsonRpcNotification for FromSuccessorNotification {}
