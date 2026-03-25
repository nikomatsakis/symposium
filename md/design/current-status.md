# Current status

Symposium is in early development. This page describes what works today. For the full vision, see the [design overview](./overview.md).

## What works

### Token-optimized cargo

`symposium cargo` wraps cargo with filtered output that reduces token usage. It supports `check`, `build`, `test`, `clippy`, `install`, and `nextest`, with other subcommands passed through directly.

### MCP server

`symposium mcp` runs an MCP server over stdio, exposing a `rust` tool. The tool accepts command strings (e.g., `cargo check`, `help`) and returns the output. The tutorial is installed as the server's instructions.

### Tutorial

`symposium tutorial` prints a guide for agents (and humans) on how to use Symposium's cargo integration.

## How to use it

There are three ways to use Symposium today:

### Claude Code plugin

Install the plugin to get a `/symposium:rust` skill. The plugin includes a bootstrap script that finds or downloads the binary automatically.

```bash
claude --plugin-dir path/to/agent-plugins/claude-code
```

See [How to install](../install.md) for details.

### MCP server

Configure your editor or agent to run `symposium mcp` as an MCP server over stdio.

### Direct CLI

If Symposium is on your PATH:

```bash
symposium cargo check
symposium cargo test --all
symposium tutorial
```

## What's not yet implemented

The [design overview](./overview.md) describes the full architecture. The following are planned but not yet built:

- **Plugin system** — `symposium.toml`-based plugins providing skills, hooks, and other capabilities
- **Per-crate skills** — Guidance documents tailored to specific dependencies
- **Hooks** — Event interception for Claude Code and other hook-based systems
- **ACP agent** — Full interception via the Agent Client Protocol
- **Editor extensions** — Native integrations for VSCode, Zed, and IntelliJ
- **`symposium skill`** — CLI for listing and retrieving skills
- **`symposium update`** — Self-update mechanism
- **Plugin repository** — Central repository of community plugins
