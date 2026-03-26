# Configuration

Symposium reads its configuration from `~/.symposium/config.toml`. The file is optional — Symposium uses defaults when it is absent or when individual fields are omitted.

## File location

| Path | Purpose |
|------|---------|
| `~/.symposium/config.toml` | User configuration |
| `~/.symposium/logs/` | Log files (one per invocation, timestamped) |

The `~/.symposium/` directory is created automatically on first run.

## Reference

```toml
[logging]
level = "info"  # trace, debug, info, warn, error
```

### `[logging]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `level` | string | `"info"` | Minimum log level. One of: `trace`, `debug`, `info`, `warn`, `error`. |

## Logging

Each invocation of `symposium` writes a log file to `~/.symposium/logs/` with a timestamped filename (e.g., `symposium-20260325-154226.log`).

To see hook payloads and other verbose output, set the log level to `debug`:

```toml
[logging]
level = "debug"
```
