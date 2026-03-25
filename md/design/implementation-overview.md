# Implementation overview

Symposium is a single Rust binary crate. The source is in `src/` with four files:

| File | Purpose |
|------|---------|
| `main.rs` | CLI entry point using clap. Defines three subcommands: `cargo`, `tutorial`, `mcp`. |
| `cargo_cmd.rs` | Runs cargo with token-optimized output via the `symposium-rtk` crate. |
| `tutorial.rs` | Renders the tutorial template (`md/tutorial.md`) with context-specific command examples. Has two render modes: CLI and MCP. |
| `mcp.rs` | MCP server over stdio using `sacp`. Exposes a single `rust` tool that dispatches to the tutorial or to the symposium binary. |

## Key dependencies

- **symposium-rtk** — Provides cargo output filtering for token efficiency
- **sacp / sacp-tokio** — MCP server implementation
- **clap** — CLI argument parsing

## Build and test

```bash
cargo check
cargo test
cargo run -- tutorial      # print the tutorial
cargo run -- cargo check   # run cargo check with filtered output
```

## Agent plugin generation

The Claude Code plugin skill is generated from a template:

```bash
just skill
```

This runs `cargo run -- tutorial`, appends the output to `agent-plugins/claude-code/skills/rust/SKILL.md.tmpl`, and writes the result to `SKILL.md`.
