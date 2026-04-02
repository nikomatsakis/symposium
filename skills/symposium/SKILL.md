---
name: symposium
description: Critical information that will help you with Rust development or information about Rust crates.
---

# Symposium — Rust Development

Use these commands for Rust development guidance. The script finds or installs the Symposium binary automatically.

* `scripts/symposium.sh rust` — get Rust coding guidance
* `scripts/symposium.sh crate <name> [--version <version>]` — get source code and guidance for a crate
* `scripts/symposium.sh crate --list` — list skills available for crates in our dependencies

**Note:** The `scripts/` path above is relative to this skill's directory. Due to a Claude Code limitation, skill scripts may not resolve correctly from the skill directory. If the relative path fails, use the full path with `${CLAUDE_PLUGIN_ROOT}/scripts/symposium.sh` instead.
