#!/usr/bin/env bash
# Generate SKILL.md from the template + tutorial output.
set -euo pipefail

echo "Generating SKILL.md..."
skill_dir="agent-plugins/claude-code/skills/rust"
cp "$skill_dir/SKILL.md.tmpl" "$skill_dir/SKILL.md"
cargo run -- tutorial 2>/dev/null >> "$skill_dir/SKILL.md"
echo "Generated $skill_dir/SKILL.md"
