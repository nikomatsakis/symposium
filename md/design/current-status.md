# Current status

Symposium is in early development. This page describes what works today. For the full vision, see the [design overview](./overview.md).

## What works

### Hooks

`symposium hook <event>` handles hook events from editor plugins. The Claude Code plugin registers a `PreToolUse` hook that invokes this subcommand, passing event data via stdin. Currently logs hook events to `~/.symposium/logs/`.

### Configuration

`~/.symposium/config.toml` provides user configuration:

```toml
[logging]
level = "info"  # trace, debug, info, warn, error

[defaults]
symposium-recommendations = true  # built-in plugin source (default: true)
user-plugins = true               # ~/.symposium/plugins/ (default: true)

[[plugin-source]]
name = "my-org"
git = "https://github.com/my-org/symposium-plugins"
auto-update = false  # default: true

[[plugin-source]]
name = "local-dev"
path = "my-plugins"  # relative to ~/.symposium/
```

### Logging

All symposium invocations emit structured logs to `~/.symposium/logs/symposium.log`. The log level is configured via `config.toml`.

### MCP server

`symposium mcp` runs an MCP server over stdio, exposing `rust` and `crate` tools. The tutorial is installed as the server's instructions. The `crate` tool supports `List` (crates with available guidance) and `Info` (fetch source and guidance for a specific crate).

### Tutorial

`symposium tutorial` prints a guide for agents (and humans) on how to use Symposium.

### Plugin sources

Plugins are discovered from configured **plugin sources**. Two built-in sources are enabled by default:

1. **`symposium-recommendations`** — the [symposium-dev/recommendations](https://github.com/symposium-dev/recommendations) repository, fetched as a tarball and cached under `~/.symposium/cache/plugin-sources/`.
2. **`user-plugins`** — the `~/.symposium/plugins/` directory for user-defined plugins.

Additional sources can be added via `[[plugin-source]]` in `config.toml`. Sources can point at a GitHub URL (`git`) or a local path (`path`, relative to `~/.symposium/` or absolute). Git sources are checked for freshness on startup and auto-updated; `auto-update = false` disables this (use `symposium update` to refresh manually).

Either built-in source can be disabled via `[defaults]` in `config.toml`.

### Plugins

A plugin is a TOML file. It can be a standalone `.toml` file or a `symposium.toml` inside a directory. Either way, the TOML is the plugin.

A plugin declares one or more `[[skills]]` groups. Each group specifies which crates it advises on and where the skill files come from:

```toml
name = "widgetlib-serde"

# group of skills for serialization in widgetlib 1.0
[[skills]]
crates = ["widgetlib=1.0", "serde"]
source.git = "https://github.com/org/repo/tree/main/widgetlib-serde"
```

When `source.git` points to a GitHub URL, symposium downloads the repository tarball, extracts the referenced subdirectory, and caches it under `~/.symposium/cache/plugins/`. The cached commit SHA is checked on each load; stale caches are refreshed automatically, and network failures fall back to the cached version.

### Skills

A skill group points at a directory following this layout:

```
dir/
    skills/
        skill-name/
            SKILL.md
            scripts/         # optional
            resources/       # optional
        another-skill/
            SKILL.md
```

Each `SKILL.md` follows the [agentskills.io](https://agentskills.io/specification.md) format: YAML-style frontmatter (name, description, license, compatibility, allowed-tools) and a markdown body.

Skills are matched to crate queries using two mechanisms:

- **`crates`** — declares which crates this skill advises on, as simple crate atoms (crate name with optional version constraint): `serde`, `tokio>=1.0`, `serde==1.0.193`. In TOML manifests, accepts a string or array: `crates = "serde"` or `crates = ["serde", "tokio"]`. In SKILL.md frontmatter, uses comma-separated values: `crates: serde, tokio>=1.0`.

This can be declared at the `[[skills]]` group level (in the plugin manifest) and at the individual skill level (SKILL.md frontmatter). They compose as AND (specialization): both layers must match for a skill to be selected. A skill with its own `crates` narrows the group's scope; it cannot widen it.

When `symposium crate <name>` or the MCP `crate` tool is invoked, matching skills are included in the output. Skills with `activation: default` have their body inlined; skills with `activation: optional` are listed with their frontmatter metadata and path so the agent can load them on demand.

## How to use it

There are three ways to use Symposium today:

### Claude Code plugin

Install the plugin to get a `/symposium:rust` skill and automatic `PreToolUse` hook integration. The plugin includes a bootstrap script that finds or downloads the binary automatically.

```bash
claude --plugin-dir path/to/agent-plugins/claude-code
```

See [How to install](../install.md) for details.

### MCP server

Configure your editor or agent to run `symposium mcp` as an MCP server over stdio.

### Direct CLI

If Symposium is on your PATH:

```bash
symposium tutorial
symposium hook pre-tool-use  # reads event JSON from stdin
```

## What's not yet implemented

The [design overview](./overview.md) describes the full architecture. The following are planned but not yet built:

- **Token-optimized cargo** — Cargo output filtering for token efficiency (temporarily removed, returning in a future release)
- **ACP agent** — Full interception via the Agent Client Protocol
- **Editor extensions** — Native integrations for VSCode, Zed, and IntelliJ
- **`symposium update`** — Self-update of the symposium binary (plugin source updates are implemented)
