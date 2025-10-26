use std::sync::Arc;

use agent_client_protocol::ContentBlock;
use agent_client_protocol::{InitializeRequest, InitializeResponse};
use scp::{AcpAgentToClientCallbacks, JsonRpcCxExt};
use scp::{
    AcpClientToAgentCallbacks, AcpClientToAgentMessages, JsonRpcConnection, JsonRpcConnectionCx,
    JsonRpcNotificationCx,
};
use tokio::{io::duplex, sync::Mutex};
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};
use tracing::Instrument;

use crate::{
    component::{ComponentProvider, MockComponentImpl},
    conductor::Conductor,
};

// Tests have been moved to integration tests in conductor/tests/initialization_sequence.rs
