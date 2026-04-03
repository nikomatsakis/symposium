# Getting started

## Install Symposium

### Pre-built binary (recommended)

Download a pre-built binary using [cargo-binstall](https://github.com/cargo-bins/cargo-binstall):

```bash
cargo binstall symposium
```

This downloads the appropriate binary for your platform from GitHub releases.

### Build from source

```bash
cargo install symposium
```

### Python package

Symposium is also available as a Python package, which bundles the native binary:

```bash
uvx symposium
# or
pipx install symposium-rs
```

## Connect to your agent

Once installed, connect Symposium to your AI coding assistant.

### Claude Code plugin

The repository includes a Claude Code plugin at `agent-plugins/claude-code/`. The plugin's bootstrap script finds or downloads the binary automatically.

```bash
claude --plugin-dir path/to/symposium/agent-plugins/claude-code
```

Then use `/symposium:rust` to activate the skill.

### MCP server

To use Symposium as an MCP server, run:

```bash
symposium mcp
```

This starts the server on stdio, exposing `rust` and `crate` tools. Configure your editor or agent to launch this command as an MCP server.

### Direct CLI

If Symposium is on your PATH, you can invoke it directly:

```bash
symposium crate --list    # list skills for crates in your project
symposium crate tokio     # get guidance for a specific crate
```

## What happens next

Once connected, Symposium scans your project's `Cargo.toml` dependencies and loads matching skills automatically. There's nothing else to configure — just start coding.

See [Usage patterns](./usage-patterns.md) for more on how to work with Symposium day-to-day.
