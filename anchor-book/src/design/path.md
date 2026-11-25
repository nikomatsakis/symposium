# Path registries

A path registry exposes a directory as a named collection. The built-in user plugin directory uses this same manager.

> **`A-path-entries`** (draft): **A path registry stops searching inside a directory once it finds a plugin there**
> Property: The path manager enumerates entries independently of workspace dependencies. Once a directory is recognized as a plugin, scanning stops at that directory. Entry paths identify packages within the registry. A bare `SKILL.md` can supply a single-skill plugin.
> Authority: `R-ecosystems`
> - Criterion `enumerate-and-load` [test]: List a local registry with a manifest plugin and a nested bare skill. Look up one plugin by entry path and search by a fragment of that path.
> - Criterion `nested` [test]: Scan a plugin with nested content and observe the enclosing entry only.
