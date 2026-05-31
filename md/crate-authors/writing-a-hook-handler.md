# Writing a hook handler

This guide walks through writing a symposium hook handler in Rust using the `symposium-hook` crate.

## Step 1. Create a new binary crate

```bash
cargo new my-hook-handler
cd my-hook-handler
cargo add symposium-hook
```

## Step 2. Write the handler

A hook handler is a program that reads a JSON event on stdin and writes a JSON response to stdout. The `symposium-hook` crate provides a `HookHandler` trait and a `run()` harness that handles the plumbing.

Implement `HookHandler` and override the methods for the events you care about:

```rust
// src/main.rs
use std::process::ExitCode;
use symposium_hook::{HookHandler, PreToolUseInput, Response, run};

struct MyHook;

impl HookHandler for MyHook {
    fn pre_tool_use(&self, event: &PreToolUseInput) -> anyhow::Result<Response> {
        if event.tool_name == "Bash" {
            Ok(Response::context("Remember: prefer non-destructive commands"))
        } else {
            Ok(Response::empty())
        }
    }
}

fn main() -> ExitCode {
    run(MyHook)
}
```

The `run()` function:

1. Reads symposium canonical JSON from stdin.
2. Deserializes it into an `Input` event.
3. Calls `handler.handle_event()`, which dispatches to the appropriate method.
4. Serializes the response to stdout (or writes to stderr and exits with code 2 for blocks).

You only need to override the methods you care about — unimplemented methods default to `Ok(Response::empty())`.

## Step 3. Register it in your plugin manifest

In your `SYMPOSIUM.toml`, reference the built binary as a hook command:

```toml
name = "my-crate"
crates = ["my-crate"]

[[hooks]]
name = "check-usage"
event = "PreToolUse"
command = { source = "cargo", crate = "my-hook-handler", executable = "my-hook-handler" }
```

## Response types

Your handler methods return `anyhow::Result<Response>`:

| Return value | Effect |
|-------------|--------|
| `Ok(Response::empty())` | No-op. Action proceeds, no output. |
| `Ok(Response::context("..."))` | Inject text into the agent's context for this event. |
| `Ok(Response::update_input(value))` | Replace the tool input (only for `PreToolUse`). |
| `Ok(Response::context_and_input("...", value))` | Inject context and replace tool input. |
| `Ok(Response::block("reason"))` | Block the action. Exits with code 2, reason on stderr. |
| `Err(...)` | Error. Exits with code 1, error message on stderr. |

## The `HookHandler` trait

```rust
pub trait HookHandler {
    fn handle_event(&self, input: &Input) -> anyhow::Result<Response> { /* dispatches */ }
    fn pre_tool_use(&self, event: &PreToolUseInput) -> anyhow::Result<Response> { /* empty */ }
    fn post_tool_use(&self, event: &PostToolUseInput) -> anyhow::Result<Response> { /* empty */ }
    fn user_prompt_submit(&self, event: &UserPromptSubmitInput) -> anyhow::Result<Response> { /* empty */ }
    fn session_start(&self, event: &SessionStartInput) -> anyhow::Result<Response> { /* empty */ }
}
```

Override `handle_event` only if you need custom dispatch logic (e.g., shared state across events). Otherwise, just override the per-event methods.

## Testing locally

You can test your handler by piping JSON directly:

```bash
cargo build
echo '{"PreToolUse":{"tool_name":"Bash","tool_input":{"command":"rm -rf /"},"session_id":null,"cwd":"/tmp"}}' \
  | ./target/debug/my-hook-handler
```

Or via the symposium CLI:

```bash
echo '{"PreToolUse":{"tool_name":"Bash","tool_input":{"command":"rm -rf /"},"session_id":null,"cwd":"/tmp"}}' \
  | cargo agents hook symposium pre-tool-use
```

## Example: blocking destructive commands

```rust
use std::process::ExitCode;
use symposium_hook::{HookHandler, PreToolUseInput, Response, run};

struct BlockDestructive;

impl HookHandler for BlockDestructive {
    fn pre_tool_use(&self, event: &PreToolUseInput) -> anyhow::Result<Response> {
        if event.tool_name == "Bash" {
            if let Some(cmd) = event.tool_input.get("command").and_then(|v| v.as_str()) {
                if cmd.contains("rm -rf") {
                    return Ok(Response::block("Destructive rm -rf commands are not allowed"));
                }
            }
        }
        Ok(Response::empty())
    }
}

fn main() -> ExitCode {
    run(BlockDestructive)
}
```

## Example: injecting context on session start

```rust
use std::process::ExitCode;
use symposium_hook::{HookHandler, SessionStartInput, Response, run};

struct InjectContext;

impl HookHandler for InjectContext {
    fn session_start(&self, _event: &SessionStartInput) -> anyhow::Result<Response> {
        Ok(Response::context(
            "This project uses tokio 1.x for async. Prefer spawn over block_on."
        ))
    }
}

fn main() -> ExitCode {
    run(InjectContext)
}
```
