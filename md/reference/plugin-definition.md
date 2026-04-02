# Plugin definition reference

A plugin is a TOML manifest loaded from a configured plugin source. It can be a standalone `.toml` file or a `symposium.toml` inside a directory.

## Minimal manifest

```toml
name = "example"

[[skills]]
crates = ["serde"]
source.path = "skills"
```

## Top-level fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | yes | Plugin name. Used in logs and CLI output. |

## `[[skills]]` groups

Each `[[skills]]` entry declares a group of skills.

| Field | Type | Description |
|-------|------|-------------|
| `crates` | string or array | Which crates this group advises on. Accepts a single string (`"serde"`) or array (`["serde", "tokio>=1.0"]`). See [Skill matching](./skill-matching.md) for atom syntax. |
| `source.path` | string | Local directory containing skill subdirectories. Resolved relative to the manifest file. |
| `source.git` | string | GitHub URL pointing to a directory in a repository (e.g., `https://github.com/org/repo/tree/main/skills`). Symposium downloads the tarball, extracts the subdirectory, and caches it. |

A skill group must have exactly one of `source.path` or `source.git`.

## `[[hooks]]`

Each `[[hooks]]` entry declares a hook.

| Field | Type | Description |
|-------|------|-------------|
| `name` | string | Descriptive name for the hook (used in logs). |
| `event` | string | Event type to match (e.g., `PreToolUse`). |
| `matcher` | string | Which tool invocations to match (e.g., `Bash`). Omit to match all. |
| `command` | string | Command to run when the hook fires. Resolved relative to the plugin directory. |

## Example: full manifest

```toml
name = "widgetlib"

[[skills]]
crates = ["widgetlib=1.0"]
source.path = "skills/general"

[[skills]]
crates = ["widgetlib=1.0"]
source.git = "https://github.com/org/widgetlib/tree/main/symposium/serde-skills"

[[hooks]]
name = "check-widget-usage"
event = "PreToolUse"
matcher = "Bash"
command = "./scripts/check-widget.sh"
```

## Validation

```bash
symposium plugin validate path/to/symposium.toml
```

This parses the manifest and reports any errors. Use `--check-crates` to also verify that crate names exist on crates.io.
