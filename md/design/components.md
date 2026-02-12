# Components

Symposium's functionality is delivered through component proxies that are orchestrated by the internal conductor. Some features use a component/adapter pattern while others are standalone components.

## Component Types

### Standalone Components

Some components provide functionality that doesn't depend on upstream capabilities. These components work with any editor and add features purely through the proxy layer.

**Example:** A component that provides git history analysis through MCP tools doesn't need special editor support - it can work with the filesystem directly.

### Component/Adapter Pairs

Other components rely on primitive capabilities from the upstream editor. For these, Symposium uses a two-layer approach:

#### Adapter Layer

The adapter sits upstream in the proxy chain and provides primitive capabilities that the component needs.

**Responsibilities:**
- Check for required capabilities during initialization
- Pass requests through if the editor provides the capability
- Provide fallback implementation if the capability is missing
- Abstract away editor differences from the component

**Example:** The IDE Operations adapter checks if the editor supports `ide_operations`. If not, it can spawn a language server (like rust-analyzer) to provide that capability.

#### Component Layer

The component sits downstream from its adapter and enriches primitive capabilities into higher-level MCP tools.

**Responsibilities:**
- Expose MCP tools to the agent
- Process tool invocations
- Send requests upstream through the adapter
- Return results to the agent

**Example:** The IDE Operations component exposes an `ide_operation` MCP tool that accepts Dialect programs and translates them into IDE operation requests sent upstream.

## Component Lifecycle

For component/adapter pairs:

1. **Initialization** - Adapter receives initialize request from upstream (editor)
2. **Capability Check** - Adapter examines editor capabilities
3. **Conditional Spawning** - Adapter spawns fallback if capability is missing
4. **Chain Assembly** - Conductor wires adapter → component → downstream
5. **Request Flow** - Agent calls MCP tool → component → adapter → editor
6. **Response Flow** - Results flow back: editor → adapter → component → agent

## Proxy Chain Direction

The proxy chain flows from editor to agent:

```
Editor → [Adapter] → [Component] → Agent
```

- **Upstream** = toward the editor
- **Downstream** = toward the agent

Adapters sit closer to the editor, components sit closer to the agent.

## Current Components

### Rust Crate Sources

Provides access to published Rust crate source code through an MCP server.

- **Type:** Standalone component
- **Implementation:** Injects an MCP server that exposes the `rust-crate-sources` tool
- **Function:** Allows agents to fetch and examine source code from crates.io

### Sparkle

Provides AI collaboration framework through prompt injection and MCP tooling.

- **Type:** Standalone component
- **Implementation:** Injects Sparkle MCP server with collaboration tools
- **Function:** Enables partnership dynamics, pattern anchors, and meta-collaboration capabilities
- **Documentation:** [Sparkle docs](https://symposium-dev.github.io/sparkle/)

### Editor Context

Injects the editor's current state (active file, selection) into agent prompts.

- **Type:** Standalone component (no adapter)
- **Implementation:** Reads a JSON state file written by the editor extension, prepends an `<editor-context>` block to each `PromptRequest`
- **Activation:** Conditional — only inserted into the proxy chain when the `SYMPOSIUM_EDITOR_STATE_FILE` environment variable is set
- **Staleness:** Skips injection if the state file is older than 30 seconds
- **Crate:** `symposium-editor-context`
- **Documentation:** [VSCode Extension Architecture](./vscode-extension/architecture.md#editor-context-proxy)

The editor extension writes the state file; the proxy reads it. They communicate only through the filesystem, bridged by the environment variable. This means the proxy works with any editor that writes the expected JSON format — it is not VSCode-specific.

## Future Components

Additional components can be added following these patterns:

- **IDE Operations** - Code navigation and search (likely component/adapter pair)
- **Walkthroughs** - Interactive code explanations
- **Git Operations** - Repository analysis
- **Build Integration** - Compilation and testing workflows
