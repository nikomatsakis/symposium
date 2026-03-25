# How to install

## Pre-built binary (recommended)

Download a pre-built binary using [cargo-binstall](https://github.com/cargo-bins/cargo-binstall):

```bash
cargo binstall symposium
```

This downloads the appropriate binary for your platform from GitHub releases.

## Build from source

```bash
cargo install symposium
```

## Python package

Symposium is also available as a Python package, which bundles the native binary:

```bash
uvx symposium
# or
pipx install symposium-rs
```

## Claude Code plugin

The repository includes a Claude Code plugin at `agent-plugins/claude-code/`. The plugin's bootstrap script finds or downloads the binary automatically.

To test locally:

```bash
claude --plugin-dir path/to/symposium/agent-plugins/claude-code
```

Then use `/symposium:rust` to activate the skill.

## MCP server

To use Symposium as an MCP server, run:

```bash
symposium mcp
```

This starts the server on stdio, exposing a `rust` tool. Configure your editor or agent to launch this command as an MCP server.
