# Packages, plugins, and their names

Suppose `my-app` declares a direct dependency on `widget` in its `Cargo.toml`. Cargo resolves `widget` to version `1.2.3`. That already gives Symposium a package to ask about, even if `widget` ships no plugin.

| Term | What it means in this example |
| --- | --- |
| Workspace | The project Symposium is acting on. Its root is a directory. `my-app` is a **member**, a local package that belongs to that workspace. |
| Package | Something a manager can identify and locate. Both `my-app` and `widget` are Cargo packages. |
| Dependency | A relationship between packages. Here `my-app` declares `widget` in its `Cargo.toml`, so `widget` is a direct dependency of `my-app`. |
| Manager | The instance Symposium asks about packages. The `cargo` manager reads Cargo’s workspace and reports `widget` as a direct dependency. |
| Registry | A named collection of plugin entries that Symposium can query through a manager. |
| Plugin | A manifest and the content it describes. A package can supply zero, one, or several plugins. The `widget` author could ship a plugin in the crate, and a registry author could separately publish a plugin about `widget`. |
| Offer | A plugin a manager reports to Symposium during discovery. If `widget` ships a plugin, Cargo can report its package ID, manifest, and directory. Symposium can show that plugin as awaiting approval while its skills are still inactive. |
| Active plugin | A plugin whose contributions Symposium can consider in this workspace. An active `widget` plugin can supply a skill even if it has no handler for the current event. See [activation](plugin-activation.md#A-flow-activation). |
| Contribution | Something a plugin supplies, such as a skill, a hook handler, an MCP server, or a CLI subcommand. |
| Predicate | A condition that decides when a plugin or one of its contributions applies. |
| Agent | The coding assistant that receives applicable contributions, such as Claude Code. |

## Which name do I use?

| Name | Example | What it identifies |
| --- | --- | --- |
| Manager name | `cargo` | The instance that owns the package name and knows how to resolve it. A configured registry has its own instance name. |
| Package name | `widget` | A package *within* a manager. Cargo uses crate names. A path registry searches and loads entries by their path relative to the registry directory. |
| Package ID | `cargo:widget:1.2.3` | A manager, package name, and version. Cargo reports this resolved ID for the workspace dependency. A lookup can also carry a version requirement, with `*` meaning any version. |
| Plugin manifest name | `widget-guide` | The name the manifest gives the plugin. A path registry can load the entry `guides/widgets` and return a plugin whose manifest name is `widget-guide`. Its returned package ID also uses `widget-guide` in the name field. |
| Skill name and origin | `review` and its `SKILL.md` path | The skill name is human-facing. During sync, Symposium uses the file’s canonical path to recognize the same skill reached through two plugins. Two files can both define a skill named `review`. |

`symposium use widget` takes a name. Symposium resolves it before recording the choice. If a configured registry and a Cargo dependency both match `widget`, the registry wins. A lookup by package ID already specifies the manager to ask.

> **`A-package-owner`** (draft): **A package ID names the manager that answers for that package**
> Property: Symposium sends a package lookup to the manager named in its ID. When a manager reports dependencies or supplies plugins, Symposium accepts only IDs belonging to that manager.
> Authority: `R-ecosystems`
> - Criterion `lookup` [test]: Give two managers a package called `foo`. Load `cargo:foo:*` and observe only Cargo’s result.
> - Criterion `ownership` [test]: Have a second manager report a Cargo dependency and return a plugin with a Cargo ID. Observe that Symposium drops both answers.
