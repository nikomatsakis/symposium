use std::{future::Future, pin::Pin};

use futures::{AsyncRead, AsyncWrite};
use scp::JsonRpcCx;
use tokio::process::Child;

/// A spawned component in the proxy chain.
///
/// This represents a component that has been launched and is connected
/// to the conductor via JSON-RPC.
pub struct Component {
    /// The child process, if this component was spawned via Command.
    /// None for mock components used in tests.
    #[expect(dead_code)]
    pub child: Option<Child>,
    pub jsonrpccx: JsonRpcCx,
}

/// Specifies how to create a component in the proxy chain.
pub enum ComponentProvider {
    /// Spawn a component by running a shell command.
    Command(String),

    /// Create a mock component for testing (provides byte streams directly).
    Mock(Box<dyn MockComponent>),
}

/// Trait for creating mock components in tests.
///
/// Mock components provide bidirectional byte streams that the conductor
/// can use to communicate via JSON-RPC, without spawning actual subprocesses.
pub trait MockComponent: Send {
    /// Create the byte streams for this mock component.
    ///
    /// Returns a pair of streams (outgoing, incoming) from the conductor's perspective:
    /// - outgoing: conductor writes to component
    /// - incoming: conductor reads from component
    fn create(
        &self,
    ) -> Pin<
        Box<
            dyn Future<
                    Output = anyhow::Result<(
                        Pin<Box<dyn AsyncWrite + Send>>,
                        Pin<Box<dyn AsyncRead + Send>>,
                    )>,
                > + Send,
        >,
    >;
}
