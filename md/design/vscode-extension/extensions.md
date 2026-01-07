# Agent Extensions

Agent extensions are proxy components that enrich the agent's capabilities. The VS Code extension allows users to configure which extensions are active, their order, and to add or remove extensions.

## Built-in Extensions

| ID | Name | Description |
|----|------|-------------|
| `sparkle` | Sparkle | AI collaboration identity and embodiment |
| `ferris` | Ferris | Rust development tools (crate sources, rust researcher) |
| `cargo` | Cargo | Cargo build and run tools |

## Configuration

Extensions are configured via the `symposium.extensions` VS Code setting:

```json
"symposium.extensions": [
  { "id": "sparkle", "enabled": true },
  { "id": "ferris", "enabled": true },
  { "id": "cargo", "enabled": true }
]
```

**Order matters** - extensions are applied in the order listed. The first extension in the list is closest to the editor, and the last is closest to the agent.

## Settings UI

The Settings panel includes an Extensions section where users can:

- **Enable/disable** extensions via checkbox (disabled extensions remain in the list but are grayed out)
- **Reorder** extensions by dragging the handle
- **Delete** extensions from the list
- **Add** extensions back via the "+ Add extension" link (shows deleted built-ins)

## CLI Interface

The VS Code extension passes extension configuration to `symposium-acp-agent` via `--proxy` arguments:

```bash
symposium-acp-agent --proxy sparkle --proxy ferris --proxy cargo -- npx @zed-industries/claude-code-acp
```

Only enabled extensions are passed, in their configured order.

## Architecture

```
┌─────────────────────────────────────────────────┐
│  VS Code Extension                              │
│  - Reads symposium.extensions setting           │
│  - Renders UI in Settings panel                 │
│  - Builds --proxy args for agent spawn          │
└─────────────────┬───────────────────────────────┘
                  │
┌─────────────────▼───────────────────────────────┐
│  symposium-acp-agent                            │
│  - Parses --proxy arguments                     │
│  - Validates proxy names                        │
│  - Builds proxy chain in specified order        │
└─────────────────┬───────────────────────────────┘
                  │
┌─────────────────▼───────────────────────────────┐
│  symposium-acp-proxy (Symposium struct)         │
│  - from_proxy_names() creates config            │
│  - build_proxies() instantiates components      │
│  - Conductor orchestrates the chain             │
└─────────────────────────────────────────────────┘
```

## Future Work

- **Registry extensions**: Allow installing extensions from the agent registry with `type = 'extension'`
- **Custom extensions**: Support arbitrary extension strings passed to `AcpAgent::from_str()`
- **Per-extension configuration**: Add sub-options for extensions (e.g., which Ferris tools to enable)
