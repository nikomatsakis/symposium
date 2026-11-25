# Telemetry

The CLI stores a per-user telemetry preference and can inspect existing local event files. No shipped Symposium command or hook currently calls the event writer. Enabling telemetry, starting a session, syncing a workspace, and executing a plugin hook all leave the event log untouched.

The [telemetry recording RFD](../rfds/telemetry-recording/README.md#event-producers) defines where the replacement recorder will obtain observations. Its implementation plan keeps collection inactive until the producers, local controls and consent flow are ready.

## Design principles for recording

- **Opt-in, per-user.** Nothing is recorded unless the user enables it; the
  preference lives in the user-wide `config.toml`.
- **Local-first.** Events are written to a local log under `~/.symposium/`.
  Uploading is a separate, deliberate step.
- **Anonymous by construction.** Events record counts and coarse metadata only —
  no prompt text, command lines, or file paths.
- **Extensible.** Events are JSON lines, so new event kinds and fields can be
  added without breaking older readers.
- **Never breaks a hook.** Every recording path is best-effort — failures are
  logged and swallowed.

## Event log format

The experimental writer appends **JSON lines** to per-day files under
`~/.symposium/telemetry/`:

```
~/.symposium/telemetry/events-2026-06-23.jsonl
```

Each line is one [`TelemetryEvent`](./module-structure.md): an `at` timestamp
plus a kind-tagged payload (`EventKind`), e.g.

```json
{"at":"2026-06-23T17:58:13Z","kind":"session_start","session_id":"P1","agent":"claude","plugins":["tokio-plugin"]}
{"at":"2026-06-23T17:58:14Z","kind":"user_prompt","session_id":"P1"}
{"at":"2026-06-23T17:58:15Z","kind":"tool_use","session_id":"P1","tool":"Bash"}
```

The experimental retention routine deletes files older than 30 days when called. The current `SessionStart` path leaves these files in place.

## Configuration

```toml
[telemetry]
enabled = true
```

The preference is collected during `symposium init` ("Enable anonymous usage
telemetry?") and can be toggled later with `symposium telemetry enable` /
`disable`.

## The `telemetry` subcommand

- `symposium telemetry status`: whether enabled, where data lives, and how
  much is stored.
- `symposium telemetry enable` / `disable`: toggle the opt-in.
- `symposium telemetry show [--count N]`: print recent events for
  inspection (the data the user would share).

## What we deliberately do *not* record

No prompt text, no shell command lines, no file paths. "How many tries until the
agent got it right" cannot be measured reliably from hook events; the trustworthy
version is an explicit user rating, which a later workstream will add.
