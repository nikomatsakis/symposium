# Symposium — AI the Rust Way

Agents writing Rust code should use the symposium tools.

## Building and running cargo

Instead of running `cargo` directly, use {{CARGO}}. It accepts the same arguments as cargo but produces more token-efficient output.

### Example: running tests

**Do not:** Run `cargo test --all --workspace`.

**Instead:** Run {{CARGO_EXAMPLE:test --all --workspace}}.
