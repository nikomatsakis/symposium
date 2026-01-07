# Common Issues

This section documents recurring bugs and pitfalls to check when implementing new features.

## VS Code Extension

### Configuration Not Affecting New Tabs

**Symptom:** User changes a setting, but new tabs still use the old value.

**Cause:** The setting affects how the agent process is spawned, but isn't included in `AgentConfiguration.key()`. Tabs with the same key share an agent process, so the new tab reuses the existing (stale) process.

**Fix:** Include the setting in `AgentConfiguration`:
1. Add the setting to the `AgentConfiguration` constructor
2. Include it in `key()` so different values produce different keys
3. Read it in `fromSettings()` when creating configurations

**Example:** The `symposium.extensions` setting was added but new tabs ignored it until extensions were added to `AgentConfiguration.key()`. See commit `fix: include extensions in AgentConfiguration key`.

**General principle:** If a setting affects process behavior (CLI args, environment, etc.), it must be part of the process identity key.
