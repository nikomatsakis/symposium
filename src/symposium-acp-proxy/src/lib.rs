//! Symposium ACP Proxy
//!
//! This crate provides the Symposium proxy functionality. It sits between an
//! editor and an agent, using sacp-conductor to orchestrate a dynamic chain
//! of component proxies that enrich the agent's capabilities.
//!
//! Two modes are supported:
//! - `Symposium`: Proxy mode - sits between editor and an existing agent
//! - `SymposiumAgent`: Agent mode - wraps a downstream agent
//!
//! Architecture:
//! 1. Receive Initialize request from editor
//! 2. Examine capabilities to determine what components are needed
//! 3. Build proxy chain dynamically using conductor's lazy initialization
//! 4. Forward Initialize through the chain
//! 5. Bidirectionally forward all subsequent messages

use anyhow::Result;
use sacp::link::{AgentToClient, ConductorToProxy, ProxyToConductor};
use sacp::{Component, DynComponent};
use sacp_conductor::{Conductor, McpBridgeMode};
use std::path::PathBuf;

/// Known proxy/extension names that can be configured.
pub const KNOWN_PROXIES: &[&str] = &["sparkle", "ferris", "cargo"];

/// Shared configuration for Symposium proxy chains.
struct SymposiumConfig {
    /// Ordered list of proxy names to include in the chain.
    proxy_names: Vec<String>,
    trace_dir: Option<PathBuf>,
}

impl SymposiumConfig {
    fn new() -> Self {
        SymposiumConfig {
            // Default: all proxies enabled
            proxy_names: KNOWN_PROXIES.iter().map(|s| s.to_string()).collect(),
            trace_dir: None,
        }
    }

    fn from_proxy_names(names: &[String]) -> Self {
        SymposiumConfig {
            proxy_names: names.to_vec(),
            trace_dir: None,
        }
    }

    /// Build proxy components from the configured names, preserving order.
    fn build_proxies(&self) -> Vec<DynComponent<ProxyToConductor>> {
        let mut proxies: Vec<DynComponent<ProxyToConductor>> = vec![];

        for name in &self.proxy_names {
            match name.as_str() {
                "sparkle" => {
                    proxies.push(DynComponent::new(sparkle::SparkleComponent::new()));
                }
                "ferris" => {
                    // Enable all Ferris tools by default
                    let ferris_config = symposium_ferris::Ferris::new()
                        .crate_sources(true)
                        .rust_researcher(true);
                    proxies.push(DynComponent::new(symposium_ferris::FerrisComponent::new(
                        ferris_config,
                    )));
                }
                "cargo" => {
                    proxies.push(DynComponent::new(symposium_cargo::CargoProxy));
                }
                other => {
                    tracing::warn!("Unknown proxy name: {}", other);
                }
            }
        }

        proxies
    }
}

/// Symposium in proxy mode - sits between an editor and an existing agent.
///
/// Use this when you want to add Symposium's capabilities to an existing
/// agent setup without Symposium managing the agent lifecycle.
pub struct Symposium {
    config: SymposiumConfig,
}

impl Symposium {
    /// Create a new Symposium with all default proxies enabled.
    pub fn new() -> Self {
        Symposium {
            config: SymposiumConfig::new(),
        }
    }

    /// Create a Symposium from a list of proxy names.
    ///
    /// Order matters - proxies are chained in the order specified.
    /// Known proxy names: "sparkle", "ferris", "cargo"
    ///
    /// Returns an error if any proxy name is unknown.
    pub fn from_proxy_names(names: &[String]) -> Result<Self, anyhow::Error> {
        // Validate all proxy names
        for name in names {
            if !KNOWN_PROXIES.contains(&name.as_str()) {
                anyhow::bail!(
                    "Unknown proxy name: '{}'. Known proxies: {}",
                    name,
                    KNOWN_PROXIES.join(", ")
                );
            }
        }

        Ok(Symposium {
            config: SymposiumConfig::from_proxy_names(names),
        })
    }

    /// Enable trace logging to a directory.
    /// Traces will be written as `<timestamp>.jsons` files.
    pub fn trace_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.config.trace_dir = Some(dir.into());
        self
    }

    /// Pair the symposium proxy with an agent, producing a new composite agent
    pub fn with_agent(self, agent: impl Component<AgentToClient>) -> SymposiumAgent {
        let Symposium { config } = self;
        SymposiumAgent::new(config, agent)
    }
}

impl Component<ProxyToConductor> for Symposium {
    async fn serve(self, client: impl Component<ConductorToProxy>) -> Result<(), sacp::Error> {
        tracing::debug!("Symposium::serve starting (proxy mode)");
        let Self { config } = self;

        let trace_dir = config.trace_dir.clone();

        tracing::debug!("Creating conductor (proxy mode)");
        let mut conductor = Conductor::new_proxy(
            "symposium",
            move |init_req| async move {
                tracing::info!(
                    "Building proxy chain with extensions: {:?}",
                    config.proxy_names
                );
                let proxies = config.build_proxies();
                Ok((init_req, proxies))
            },
            McpBridgeMode::default(),
        );

        // Enable tracing if a directory was specified
        if let Some(dir) = trace_dir {
            std::fs::create_dir_all(&dir).map_err(sacp::Error::into_internal_error)?;
            let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S");
            let trace_path = dir.join(format!("{}.jsons", timestamp));
            conductor = conductor
                .trace_to_path(&trace_path)
                .map_err(sacp::Error::into_internal_error)?;
            tracing::info!("Tracing to {}", trace_path.display());
        }

        tracing::debug!("Starting conductor.run()");
        conductor.run(client).await
    }
}

/// Symposium in agent mode - wraps a downstream agent.
///
/// Use this when Symposium should manage the agent lifecycle, e.g., when
/// building a standalone enriched agent binary.
pub struct SymposiumAgent {
    config: SymposiumConfig,
    agent: DynComponent<AgentToClient>,
}

impl SymposiumAgent {
    fn new<C: Component<AgentToClient>>(config: SymposiumConfig, agent: C) -> Self {
        SymposiumAgent {
            config,
            agent: DynComponent::new(agent),
        }
    }
}

impl Component<AgentToClient> for SymposiumAgent {
    async fn serve(
        self,
        client: impl Component<sacp::link::ClientToAgent>,
    ) -> Result<(), sacp::Error> {
        tracing::debug!("SymposiumAgent::serve starting (agent mode)");
        let Self { config, agent } = self;

        let trace_dir = config.trace_dir.clone();

        tracing::debug!("Creating conductor (agent mode)");
        let mut conductor = Conductor::new_agent(
            "symposium",
            move |init_req| async move {
                tracing::info!(
                    "Building proxy chain with extensions: {:?}",
                    config.proxy_names
                );
                let proxies = config.build_proxies();
                Ok((init_req, proxies, agent))
            },
            McpBridgeMode::default(),
        );

        // Enable tracing if a directory was specified
        if let Some(dir) = trace_dir {
            std::fs::create_dir_all(&dir).map_err(sacp::Error::into_internal_error)?;
            let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S");
            let trace_path = dir.join(format!("{}.jsons", timestamp));
            conductor = conductor
                .trace_to_path(&trace_path)
                .map_err(sacp::Error::into_internal_error)?;
            tracing::info!("Tracing to {}", trace_path.display());
        }

        tracing::debug!("Starting conductor.run()");
        conductor.run(client).await
    }
}
