# symposium-crate-sources-proxy

An ACP proxy component that provides agents with the ability to research Rust crate source code.

## Overview

This component uses a **sub-agent research pattern**: when an agent needs information about a Rust crate, the component spawns a dedicated research session with its own agent to investigate the crate sources and return findings.

## Usage

The component exposes a `rust_crate_query` MCP tool to agents:

```json
{
  "crate_name": "serde",
  "crate_version": "1.0",
  "prompt": "How do I use the derive macro for custom field names?"
}
```

The sub-agent then:
1. Downloads and extracts the crate source from crates.io
2. Reads and analyzes the source code
3. Returns synthesized findings (not raw pattern matches)

## Integration

This component is typically used as part of the Symposium proxy chain:

```rust
use symposium_crate_sources_proxy::CrateSourcesProxy;

components.push(sacp::DynComponent::new(CrateSourcesProxy {}));
```

## Documentation

For detailed architecture and implementation information, see the [Symposium documentation](https://symposium-dev.github.io/symposium/).

## License

Apache-2.0
