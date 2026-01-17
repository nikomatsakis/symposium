# Run Mode

The `run` subcommand simplifies editor integration by reading agent configuration from a file rather than requiring command-line arguments.

## Configuration File

**Location:** `~/.symposium/config.jsonc`

The file uses JSONC (JSON with comments) format:

```jsonc
{
  // Downstream agent command (parsed as shell words)
  "agent": "npx -y @zed-industries/claude-code-acp",
  
  // Proxy extensions to enable
  "proxies": [
    { "name": "sparkle", "enabled": true },
    { "name": "ferris", "enabled": true },
    { "name": "cargo", "enabled": true }
  ]
}
```

### Fields

| Field | Type | Description |
|-------|------|-------------|
| `agent` | string | Shell command to spawn the downstream agent. Parsed using shell word splitting. |
| `proxies` | array | List of proxy extensions with `name` and `enabled` fields. |

## Architecture: SessionAgent

The `run` command spawns a **SessionAgent** that manages the lifecycle of sessions and conductors:

```
Client (editor)
    │
    ▼
SessionAgent
    │
    ├── handles InitializeRequest (returns default capabilities)
    ├── handles /symposium:config slash command (modal config mode)
    │
    │  on NewSessionRequest:
    │    1. Load current config
    │    2. Find or create Conductor for that config
    │    3. Forward session to Conductor
    │
    │  on PromptRequest:
    │    route to appropriate Conductor by session ID
    │
    ▼
Conductor A (config v1) ──→ proxies ──→ downstream agent
    └── handles sessions 1, 2, 3

Conductor B (config v2) ──→ proxies ──→ downstream agent
    └── handles sessions 4, 5
```

### Session-to-Conductor Mapping

The SessionAgent groups sessions by configuration:

- When a new session starts, the SessionAgent loads the current config from disk
- If a Conductor already exists for that config (compared by equality), the session is delegated to it
- If not, a new Conductor is spawned, initialized, and the session is delegated to it

This means:
- Multiple sessions with the same config share a Conductor
- Config changes take effect on the next session, not immediately
- Existing sessions continue with their original config

### Conductor Lifecycle

Each Conductor:
- Is keyed by config (agent command + proxy settings)
- Receives an `InitializeRequest` when first created
- Builds its proxy chain based on the config
- Handles multiple sessions over its lifetime
- Spawns and manages the downstream agent process

## Configuration Slash Command

Users can modify configuration at any time via the `/symposium:config` slash command. When invoked:

1. The SessionAgent intercepts the prompt (before routing to any Conductor)
2. Enters **modal config mode** for that session - all subsequent prompts go to the config handler
3. Displays current configuration:

```
# Current Setup

* Agent: Claude Code
* Extensions:
  - sparkle (enabled)
  - ferris (enabled)
  - cargo (enabled)

What would you like to change?

1) Change agent
2) Done, save changes and exit
3) Cancel all changes
```

4. On save: writes config to disk, tells user "Changes saved! New sessions will use the updated configuration."
5. On done/cancel: exits config mode, resumes normal routing to the Conductor

The modal takeover ensures the config interaction is isolated from the downstream agent.

## First-Time Setup

When no configuration file exists, the SessionAgent runs an interactive setup wizard instead of delegating to a Conductor:

1. Fetches available agents from the [ACP Agent Registry](https://github.com/agentclientprotocol/registry)
2. Presents a numbered list of agents
3. User types a number to select
4. Saves configuration with all proxies enabled by default
5. Continues with the selected agent

### Known Agents

The setup wizard offers agents from the registry, with fallback to:

| Name | Command |
|------|---------|
| Claude Code | `npx -y @zed-industries/claude-code-acp` |
| Gemini CLI | `npx -y -- @google/gemini-cli@latest --experimental-acp` |
| Codex | `npx -y @zed-industries/codex-acp` |
| Kiro CLI | `kiro-cli-chat acp` |

## Implementation

| Component | Location | Purpose |
|-----------|----------|---------|
| `SymposiumUserConfig` | `src/symposium-acp-agent/src/config.rs` | Config types with `Eq`/`Hash` for comparison |
| `SessionAgent` | `src/symposium-acp-agent/src/session_agent.rs` | Session routing and config mode |
| CLI integration | `src/symposium-acp-agent/src/main.rs` | `run` subcommand |

### Dependencies

| Crate | Purpose |
|-------|---------|
| `serde_jsonc` | Parse JSON with comments |
| `shell-words` | Parse agent command string into arguments |
| `dirs` | Cross-platform home directory resolution |
