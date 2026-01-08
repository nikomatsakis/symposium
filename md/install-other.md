# Other Editors

## Zed

Clone the repository and run the setup tool:

```bash
git clone https://github.com/symposium-dev/symposium.git
cd symposium
cargo setup --zed
```

This configures Zed with multiple agent options:
- Symposium (Claude Code)
- Symposium (Codex)
- Symposium (Kiro CLI)
- Symposium (Gemini)

Restart Zed after setup to use the new configurations.

## Other ACP-compatible Editors

For other ACP-compatible editors, install the Symposium agent binary:

```bash
cargo binstall symposium-acp-agent
```

Then configure your editor to use `symposium-acp-agent act-as-agent` as the agent command, passing your preferred downstream agent. For example, with Claude Code:

```bash
symposium-acp-agent act-as-agent --proxy defaults -- npx -y @anthropic-ai/claude-code-acp
```

The `--proxy defaults` enables all Symposium extensions (Sparkle, Ferris, Cargo). The `--` separates Symposium's arguments from the downstream agent command.
