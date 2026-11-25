# `symposium telemetry`

Manage opt-in, per-user usage telemetry. See the
[telemetry design chapter](../design/telemetry.md) for the event format and the
[`[telemetry]` configuration](./configuration.md#telemetry) for the underlying
config key.

Telemetry is **off by default**, **local-first**, and **never uploaded
automatically** — you share it yourself. The preference is also offered during
`symposium init`.

## Usage

```bash
symposium telemetry [status]      # whether enabled, where data lives, how much is stored (default)
symposium telemetry enable        # save [telemetry] enabled = true
symposium telemetry disable       # turn it off
symposium telemetry show [--count N]   # print recent events (JSON lines) for inspection
```

## Where the data lives

`show` reads existing daily files under `~/.symposium/telemetry/`, such as
`events-2026-06-23.jsonl`. `enable` saves the preference. Current CLI commands
and hooks leave the event files untouched, including during session start and sync.
