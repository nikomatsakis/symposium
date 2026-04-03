# What is Symposium?

AI coding assistants are pretty good at Rust — until they're not. They hallucinate trait methods, suggest deprecated patterns, and give generic advice where your specific crates have strong opinions about how things should be done.

The people who know best are the crate authors. They know the idioms, the pitfalls, the patterns that actually work. Symposium gets that knowledge into your AI assistant's hands.

## How it works

Crate authors publish skills and other extensions to teach your AI assistant how to use their libraries well. Symposium scans your project's dependencies and automatically loads the right guidance for the crates you're actually using. No configuration needed.

When you're working with axum, your assistant knows axum's conventions. When you're using sqlx, it knows the query patterns. The advice comes from the people who built the libraries — not from stale training data.

## What you get

- **Crate-specific guidance** that activates automatically based on your `Cargo.toml`
- **Custom checks and lints** that catch mistakes before they become bugs
- **Up-to-date knowledge** that doesn't depend on when your model was trained
- **Streamlined workflows** that handle formatting, testing, and other routine steps so you spend fewer tokens on boilerplate

## For crate authors

If you maintain a Rust crate, you can publish skills for Symposium so that every AI-assisted user of your library gets your best practices built in. Think of it as documentation that the AI actually reads.

See [Supporting your crate](./crate-authors/supporting-your-crate.md) for how to get started.

## Works however you work

Symposium integrates with various AI agents and editors. It supports multiple capability types — hooks, skills, proxies — and adapts them to your setup, gracefully degrading where there isn't an exact match.
