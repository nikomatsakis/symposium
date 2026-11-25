# Cargo

The built-in Cargo manager lives in the `symposium-pm-cargo` crate. It can also run as a configured executable. Its workspace answers follow Cargo membership and dependency resolution.

> **`A-cargo-workspace`** (draft): **Cargo reports the workspace members and their direct dependencies**
> Property: Cargo reports the workspace root, member directories and direct dependencies across members. A dependency can supply skills in its crate directory without a `Symposium.toml`. A transitive dependency alone does not enter the direct dependency set.
> Authority: `R-ecosystems`
> - Criterion `member-and-offer` [test]: Query the Cargo PM process for the root, member, lockfile input and direct path dependency. List offers and locate the dependency’s skill.
> - Criterion `transitive` [test]: Sync a workspace whose dependency pulls in another crate. Observe that the latter does not satisfy a direct-dependency gate.

> **`A-cargo-embedded`** (draft): **Cargo offers a dependency when its available source contains plugin content**
> Property: For Cargo-owned dependency IDs, Cargo inspects source it can locate without fetching updates. A Symposium manifest, an Agent Plugins manifest, a `[package.metadata.symposium]` table, or a `SKILL.md` under `skills/` identifies an embedded plugin. Cargo omits a package whose source is unavailable or contains none of these. Its workspace dependency ID remains valid in either case.
> Authority: `R-ecosystems`
> - Criterion `skill-only` [test]: Give a direct path dependency a `skills/usage/SKILL.md` and no plugin manifest. Observe a plugin offer rooted at the crate with the skill accessible there. Remove the skill and query again. The dependency list stays the same and the offer disappears.
