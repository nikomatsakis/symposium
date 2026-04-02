# Creating a plugin

A plugin is a TOML manifest that bundles skills and hooks together. You need a plugin if you want to publish hooks, or if you want to host skills from your own repository rather than contributing them to the recommendations repo.

## Minimal example

```toml
name = "widgetlib"

[[skills]]
crates = ["widgetlib"]
source.path = "skills"
```

This tells Symposium: "when a project depends on `widgetlib`, load skills from the `skills/` directory next to this manifest."

## Adding hooks

```toml
name = "widgetlib"

[[skills]]
crates = ["widgetlib"]
source.path = "skills"

[[hooks]]
name = "check-widget-usage"
event = "PreToolUse"
matcher = "Bash"
command = "./scripts/check-widget.sh"
```

See [Publishing hooks](./publishing-hooks.md) for more on writing hooks.

## Where to put your plugin

You have two options:

1. **In your crate's repository** — add a `symposium.toml` at the root (or a subdirectory). Users or the Symposium recommendations repo can point to it via a git URL.
2. **In the recommendations repo** — submit a PR to [symposium-dev/recommendations](https://github.com/symposium-dev/recommendations) with your plugin manifest.

## Validation

```bash
symposium plugin validate path/to/symposium.toml
```

This parses the manifest and reports any errors. Use `--check-crates` to also verify that crate names exist on crates.io.

## Reference

See the [Plugin definition reference](../reference/plugin-definition.md) for the full manifest format.
