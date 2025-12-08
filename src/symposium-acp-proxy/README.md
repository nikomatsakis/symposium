# symposium-acp-proxy

Symposium ACP proxy that orchestrates component chains to enrich agent capabilities.

## Overview

This crate provides the core Symposium functionality as both a library and binary. It sits between an editor and an agent, using a conductor to orchestrate a dynamic chain of component proxies.

```
Editor --> symposium-acp-proxy --> Agent
```

## Library Usage

The `Symposium` type implements `sacp::Component`, making it composable with other ACP components:

```rust
use sacp::Component;
use symposium_acp_proxy::Symposium;

// Use with stdio (proxy mode)
Symposium::new()
    .serve(sacp_tokio::Stdio::new())
    .await?;

// Or compose with other components
Symposium::new()
    .sparkle(true)
    .crate_sources_proxy(true)
    .serve(downstream_component)
    .await?;
```

## Binary Usage

Run as a standalone proxy:

```bash
symposium-acp-proxy
```

## Components

The proxy chain includes:
- **Rust Crate Sources** - Research Rust crate source code via sub-agent pattern
- **Sparkle** - AI collaboration identity framework (optional)

## Documentation

For detailed architecture and implementation information, see the [Symposium documentation](https://symposium-dev.github.io/symposium/).

## License

Apache-2.0
