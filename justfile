# Build the project
build:
    cargo build

# Assemble plugin artifacts into target/artifacts/
artifacts: build
    cargo artifacts

# Launch Claude Code with the Symposium plugin
claude: artifacts
    claude --plugin-dir target/artifacts/claude-code-plugin
