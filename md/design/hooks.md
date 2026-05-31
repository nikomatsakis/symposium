# Hooks

Symposium supports two modes for plugin hooks: **symposium-format** (portable) and **native** (agent-specific).

## Symposium-format hooks

A symposium-format hook receives events in the symposium canonical format, regardless of which agent is running. Symposium acts as the intermediary:

1. The agent fires an event in its own wire format.
2. Symposium's global hook handler receives it and converts to the symposium canonical format.
3. Symposium dispatches to the plugin hook (stdin JSON, stdout JSON).
4. Symposium converts the hook's output back to the agent's wire format and returns it.

These hooks are portable — a single implementation works across all agents. The tradeoff is that they only have access to the fields that symposium's canonical format exposes.

## Native hooks

A native hook is written for a specific agent's wire format (e.g., Claude Code's `PreToolUse` JSON schema). Instead of being dispatched at runtime by symposium, native hooks are **registered directly** into the agent's configuration during `cargo agents sync`.

When the agent fires an event, it invokes the native hook directly — symposium is not in the loop for that hook's execution. Symposium's global handler still fires for the same event, but it skips delivery to any plugin that has a native handler registered for the current agent.

Native hooks get full fidelity of the agent's event system (all fields, agent-specific features like `updatedInput` or `modifiedArgs`), but only work on the agent they target.

## Dispatch rule

When symposium's global handler receives an event from agent A:

1. Load all plugins and find hooks matching the event.
2. For each plugin:
   - If the plugin declares a **native hook for agent A** → skip (the agent already invoked it directly).
   - If the plugin declares a **symposium-format hook** → deliver in symposium canonical format.

A plugin may declare both: a native hook for specific agents and a symposium-format hook as a fallback. For example, a plugin with hooks for `claude`, `gemini`, and `symposium` would have:
- On Claude: the native claude hook runs directly; symposium skips delivery.
- On Gemini: the native gemini hook runs directly; symposium skips delivery.
- On Copilot: no native handler → the symposium hook is delivered.

## Declaring hook format

The `format` field on a `[[hooks]]` entry controls which mode is used:

- `format = "symposium"` (default) — symposium delivers this hook at runtime, converting from the agent's wire format.
- `format = "claude"` / `"copilot"` / `"gemini"` / `"codex"` / `"kiro"` — symposium registers this hook natively into that agent's configuration at sync time. At runtime, symposium skips delivery for this plugin when the matching agent is active.

See the [plugin definition reference](../reference/plugin-definition.md#hooks) for the full hook schema and examples.
