#!/usr/bin/env bash
# Build a self-contained Claude Code plugin directory ready for distribution.
# Expects SKILL.md to already be generated (run ci/generate-skill.sh first).
set -euo pipefail

out="${1:-dist/claude-code-plugin}"
rm -rf "$out"
mkdir -p "$out"

# Copy the plugin structure (exclude the template and old skill-local scripts)
cp -r agent-plugins/claude-code/.claude-plugin "$out/"
cp -r agent-plugins/claude-code/skills "$out/"
rm -f "$out/skills/rust/SKILL.md.tmpl"
rm -rf "$out/skills/rust/scripts"

# Copy shared scripts
cp -r agent-plugins/claude-code/scripts "$out/"
chmod +x "$out/scripts/symposium.sh"

# Copy hooks
cp -r agent-plugins/claude-code/hooks "$out/"

# Add the MCP server config
cat > "$out/.mcp.json" << 'EOF'
{
  "mcpServers": {
    "symposium": {
      "command": "${CLAUDE_PLUGIN_ROOT}/scripts/symposium.sh",
      "args": ["mcp"]
    }
  }
}
EOF

echo "Built Claude Code plugin at $out/"
echo "Test with: claude --plugin-dir $out"
