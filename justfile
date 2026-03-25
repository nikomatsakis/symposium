# Generate the SKILL.md from the template + tutorial output
skill:
    ci/generate-skill.sh

# Build a self-contained Claude Code plugin directory ready for distribution
claude-code-plugin: skill
    ci/build-claude-code-plugin.sh
