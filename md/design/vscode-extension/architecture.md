# VSCode Extension Architecture

The Symposium VSCode extension provides a chat interface for interacting with AI agents. The extension delegates UI and agent communication to [Toad](https://github.com/anthropics/toad), embedding its web frontend in a webview iframe.

## Architecture

```
┌─────────────────────────────────────────────────┐
│  VSCode Extension (Node.js)                     │
│  - Spawns Toad as a detached process            │
│  - Embeds Toad's web UI in an iframe            │
│  - Tracks editor state (active file, selection) │
│  - Manages Toad process lifecycle               │
└────────┬──────────────────────────┬─────────────┘
         │                          │
         │ iframe (localhost)       │ SYMPOSIUM_EDITOR_STATE_FILE
         ▼                          ▼
┌──────────────────┐    ┌─────────────────────────┐
│  Toad             │    │  State File (JSON)       │
│  - Chat UI        │    │  - Active file path      │
│  - Tool approval  │    │  - Language ID            │
│  - Tab management │    │  - Selected text          │
│  - Session state  │    │  - Workspace folders      │
└────────┬──────────┘    └─────────────┬───────────┘
         │ ACP (stdio)                 │ std::fs::read
         ▼                             ▼
┌─────────────────────────────────────────────────┐
│  Symposium Conductor                            │
│  ┌────────────────────────────────────────────┐ │
│  │ Editor Context Proxy (reads state file)    │ │
│  ├────────────────────────────────────────────┤ │
│  │ Sparkle / Ferris / Cargo / other proxies   │ │
│  └────────────────────────────────────────────┘ │
│                    ↓ ACP (stdio)                │
│              Downstream Agent                   │
└─────────────────────────────────────────────────┘
```

## Toad Integration

The extension spawns Toad as a **detached process** that runs an HTTP server on a local port. The webview displays an iframe pointing to `http://localhost:<port>/`.

Toad handles:
- Chat UI rendering and interaction
- Tool permission approval cards
- Tab and session management
- Streaming response display

The extension handles:
- Finding a free port and spawning Toad
- Persisting the Toad process across VSCode reloads (via `globalState`)
- Reconnecting to an existing Toad process if one is already running
- Tracking editor state for the editor context proxy

### Process Lifecycle

Toad is spawned with `detached: true` and `child.unref()`, so it survives extension host restarts. The extension stores `{pid, port}` in `globalState`. On activation, it checks whether the saved port is still responding before spawning a new process.

The Toad command is:
```bash
toad acp "<conductor-command> run" --serve --port <port>
```

Toad spawns the Symposium conductor internally and manages the ACP connection over stdio.

### Webview

The webview is minimal — an HTML page containing a single iframe:

```html
<iframe src="http://localhost:<port>/" sandbox="allow-scripts allow-same-origin allow-forms allow-popups" />
```

Content Security Policy restricts `frame-src` to the Toad port. The extension uses `retainContextWhenHidden: true` to keep the iframe alive when the panel is hidden.

## Editor Context Proxy

The editor context proxy injects the editor's current state (active file, selection) into agent prompts. It connects the TypeScript extension to the Rust proxy chain via file-based IPC.

### How It Works

1. **TypeScript writes**: `EditorStateTracker` subscribes to `onDidChangeActiveTextEditor` and `onDidChangeTextEditorSelection`. On changes (debounced 100ms), it writes a JSON file to the OS temp directory using atomic writes (write to `.tmp`, then `renameSync`).

2. **Env var bridges the gap**: The extension passes `SYMPOSIUM_EDITOR_STATE_FILE=<path>` to Toad at spawn time. Toad passes it through to the conductor, which passes it to the proxy.

3. **Rust reads**: `EditorContextComponent` reads `SYMPOSIUM_EDITOR_STATE_FILE` at startup. On each `PromptRequest`, it reads the JSON file, checks freshness (skips if >30 seconds old), and prepends an `<editor-context>` block to the prompt content.

### State File Format

```json
{
  "activeFile": "/project/src/main.rs",
  "languageId": "rust",
  "selection": {
    "text": "fn main() { ... }",
    "startLine": 10,
    "endLine": 12
  },
  "workspaceFolders": ["/project"]
}
```

### Why File-Based IPC

- Survives detached process restarts on both sides — no reconnection logic
- Zero coordination overhead — no ports, sockets, or protocol negotiation
- The env var doubles as a capability signal: the proxy is only inserted into the conductor chain when the variable is set, so CLI usage pays no cost

### Conditional Proxy Insertion

The conductor checks for `SYMPOSIUM_EDITOR_STATE_FILE` at chain assembly time. If present, `EditorContextComponent` is inserted as the first proxy in the chain. If absent (CLI usage, non-VSCode editors), the proxy is not included.

## Key Files

| File | Purpose |
|------|---------|
| `vscode-extension/src/extension.ts` | Extension activation, wires up EditorStateTracker |
| `vscode-extension/src/toadPanelProvider.ts` | Spawns Toad, embeds iframe, passes env var |
| `vscode-extension/src/editorState.ts` | Tracks editor state, writes JSON file |
| `src/symposium-editor-context/src/lib.rs` | ACP proxy that reads state file, injects into prompts |
| `src/symposium-acp-agent/src/config_agent/conductor_actor.rs` | Conditional proxy insertion |

## Configuration

| Setting | Description |
|---------|-------------|
| `symposium.toadCommand` | Path to the `toad` binary (default: `"toad"`, found via PATH) |
| `symposium.toadPort` | Fixed port for Toad (default: `0`, auto-selects a free port) |
| `symposium.acpAgentPath` | Override path to `symposium-acp-agent` binary |

## Language Model Provider

The extension also includes an experimental Language Model Provider that exposes ACP agents via VS Code's `LanguageModelChatProvider` API. This is independent of the Toad-based chat UI and is documented separately:

- [Language Model Provider](./lm-provider.md)
- [Language Model Tool Bridging](./lm-tool-bridging.md)

See also: [Common Issues](../common-issues.md) for recurring bug patterns.
