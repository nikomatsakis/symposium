# Cargo

The built-in Cargo manager lives in the `symposium-pm-cargo` crate. It can also run as a configured executable. Its workspace answers follow Cargo membership and dependency resolution.

> **`A-cargo-workspace`** (draft): **Cargo separates members from dependencies**
> Property: Cargo reports the workspace root, member directories and direct dependencies across members. A dependency can supply skills in its crate directory without a `Symposium.toml`. A transitive dependency alone does not enter the direct dependency set.
> Authority: `R-ecosystems`
> - Criterion `member-and-offer` [test]: Query the Cargo PM process for the root, member, lockfile input and direct path dependency. List offers and locate the dependency’s skill.
> - Criterion `transitive` [test]: Sync a workspace whose dependency pulls in another crate. Observe that the latter does not satisfy a direct-dependency gate.
