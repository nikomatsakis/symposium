# Extension Packaging

This chapter documents the design decisions for building and distributing the VSCode extension.

## Architecture Overview

The extension consists of:

1. **TypeScript code** - The extension logic, bundled via webpack
2. **Native binary** - The `symposium-acp-agent` Rust binary for the target platform
3. **External dependency** - [Toad](https://github.com/anthropics/toad), which must be installed separately (via PATH or `symposium.toadCommand` setting)

## Platform-Specific Extensions

We publish **separate extensions for each platform** rather than a universal extension containing all binaries.

**Rationale:**
- A universal extension would be ~70MB+ (all platform binaries)
- Platform-specific extensions are ~7MB each
- VSCode Marketplace natively supports this - users automatically get the right variant
- Aligns with how other extensions with native dependencies work (rust-analyzer, etc.)

**Supported platforms:**

| Platform | Description |
|----------|-------------|
| darwin-arm64 | macOS Apple Silicon |
| darwin-x64 | macOS Intel |
| linux-x64 | Linux x86_64 |
| linux-arm64 | Linux ARM64 |
| win32-x64 | Windows x86_64 |

## Binary Resolution

The extension uses a fallback chain for finding the conductor binary:

1. **Bundled binary** in `bin/<platform>/` (production)
2. **PATH lookup** (development)
3. **User override** via the `symposium.acpAgentPath` setting — if set, this path is used verbatim and takes precedence

This enables local development without packaging - developers can `cargo install` the binary and the extension finds it in PATH.

## Release Flow

Releases are orchestrated through release-plz and GitHub Actions:

```
release-plz creates tag
        ↓
GitHub Release created
        ↓
Binary build workflow triggered
        ↓
┌───────────────────────────────────────┐
│  Build binaries (parallel)            │
│  - macOS arm64/x64                    │
│  - Linux x64/arm64/musl               │
│  - Windows x64                        │
└───────────────────────────────────────┘
        ↓
Upload archives to GitHub Release
        ↓
┌───────────────────────────────────────┐
│  Build VSCode extensions (parallel)   │
│  - One per platform                   │
│  - Each bundles its platform binary   │
└───────────────────────────────────────┘
        ↓
Upload .vsix files to GitHub Release
        ↓
Publish to marketplaces (TODO)
```

**Why GitHub Releases as the source:**
- Single source of truth for all binaries
- Enables Zed extension (points to release archives)
- Enables direct downloads for users not on VSCode
- Versioned and immutable

## Local Development

For development without building platform packages:

1. Install Toad: follow [Toad installation instructions](https://github.com/anthropics/toad)
2. Install the conductor: `cargo install --path src/symposium-acp-agent`
3. Build the extension: `cd vscode-extension && npm run compile`
4. Launch via F5 in VSCode

The extension finds both binaries via PATH when no bundled binary exists.
