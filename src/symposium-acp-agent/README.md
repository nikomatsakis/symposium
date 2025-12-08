# symposium-acp-agent

A Symposium-enriched ACP agent that wraps downstream agents with enhanced capabilities.

## Overview

This binary acts as a complete agent that editors can spawn directly. It combines the Symposium component chain with any downstream ACP agent.

```
Editor --> symposium-acp-agent --> downstream-agent (e.g., claude-code)
```

This is ideal for Zed extensions and similar scenarios where the editor expects to spawn a single agent binary.

## Usage

```bash
symposium-acp-agent -- <downstream-agent-command>
```

### Examples

Wrap Claude Code:
```bash
symposium-acp-agent -- npx -y @zed-industries/claude-code-acp
```

Disable optional components:
```bash
symposium-acp-agent --no-sparkle -- npx -y @zed-industries/claude-code-acp
```

## Options

- `--no-sparkle` - Disable Sparkle integration
- `--no-crate-researcher` - Disable Rust crate source research

## Components

The agent includes all Symposium components:
- **Rust Crate Sources** - Research Rust crate source code via sub-agent pattern
- **Sparkle** - AI collaboration identity framework

## Documentation

For detailed architecture and implementation information, see the [Symposium documentation](https://symposium-dev.github.io/symposium/).

## License

Apache-2.0
