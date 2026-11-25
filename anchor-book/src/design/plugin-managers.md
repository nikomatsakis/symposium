# Plugin managers

A package manager (PM) answers questions about a project and the plugins it can supply.

> **`A-pm-offer`** (draft): **Each PM offers plugins under its own package IDs**
> Property: To offer a plugin, a PM supplies its manifest and content directory under a package ID naming that PM. Symposium rejects offers bearing another manager’s ID. Symposium validates the manifest and decides whether the plugin’s contributions become active.
> Authority: `R-ecosystems`
> - Criterion `awaiting-consent` [test]: With automatic enablement off, observe `crate-a` listed as awaiting consent. Run `symposium use crate-a` and observe that the same plugin is now active.

For example, the Cargo PM can offer a plugin shipped in the `widget` crate. A Git registry can offer a guide to `widget` written by someone else. The guide comes from the registry’s checkout and carries the registry’s package ID. Both plugins can apply to the same dependency.

> **`A-pm-bootstrap`** (draft): **Symposium selects the initial managers**
> Property: Symposium builds the manager set from its defaults, executable commands declared in `[[package-manager]]`, and named `[[registry]]` entries. A configured `cargo` command replaces the built-in Cargo instance. Plugin discovery uses that set.
> Authority: `R-ecosystems`
> - Criterion `replace-cargo` [test]: Configure `symposium-pm-cargo` as `cargo`. Observe one Cargo manager and the workspace dependency reported by that process.

> **`A-pm-workspace`** (draft): **Managers describe the workspace**
> Property: `project_info` reports the project root, local packages and inputs used to detect change. `list_deps` reports package IDs for direct workspace dependencies. Managers can report no project. Where managers report roots, Symposium requires agreement.
> Authority: `R-ecosystems`
> - Criterion `root-disagreement` [test]: Have two managers report different roots. Request the workspace and observe an error naming both managers.
> - Criterion `no-project` [test]: Have the manager report no project and observe that the workspace has no root.

> **`A-pm-offers`** (draft): **Each PM supports `offers(deps)`, which returns the plugins it offers given a set of dependencies**
> Property: The PM protocol calls this operation `active_plugins(deps)`. For ecosystem managers, Symposium passes the combined list of direct workspace dependencies reported by `list_deps`. Cargo returns plugins embedded in the listed crates. Symposium calls the same operation on a registry with an empty list, and the registry returns its own plugin entries.
> Authority: `R-ecosystems`
> - Criterion `synthesized-offer` [test]: List offers from an external manager that synthesizes a manifest. Observe its named plugin and dependency gate after validation.

For example, suppose a workspace member declares `widget = "1"` in `Cargo.toml` and Cargo resolves it to version `1.2.3`. The Cargo manager reads that resolution from `cargo metadata` and reports `cargo:widget:1.2.3` (manager, package name, resolved version). When Symposium asks for plugin offers, the Cargo manager checks whether that crate ships a plugin. The crate counts as a dependency whether or not it ships one.

> **`A-pm-load`** (draft): **Which plugins does this particular package supply?**
> Property: Symposium sends `load_plugin(id, update)` to the manager named in the package ID. One package can supply zero, one or several plugins. With updates disabled, the manager uses only content already on disk.
> Authority: `R-ecosystems`
> - Criterion `specific-package` [test]: Request an MCP-only dependency by ID. Observe its server in the loaded plugin even though the package ships no skills.

Both queries identify each plugin by its package ID, content root and raw manifest. A manager may synthesize the manifest. Symposium validates it.

### Other questions a manager can answer

| Query | Meaning |
| --- | --- |
| `canonical_names(names)` | Give the comparison spelling for each name, in order. Configuration and identity checks use the owning manager’s rules. |
| `search(text)` | Return matching package names for a human lookup. Dependency discovery calls `active_plugins`. |
| `fetch(id, update)` | Locate or acquire content and return its resolved ID and directory. |
| `refresh(update)` | Refresh the backing source and report whether it was refreshed. |

External PMs speak newline-delimited JSON-RPC over stdio. `initialize` establishes the protocol version, name, workspace and cache location.

> **`A-pm-protocol`** (draft): **A configured command must identify itself**
> Property: An external manager answers its initial handshake with the configured name and a supported protocol version. Either mismatch makes the instance unavailable.
> Authority: `R-ecosystems`
> - Criterion `name` [test]: Configure the name `demo` and have the command answer with another name. Observe the first request fail.
> - Criterion `version` [test]: Return an unsupported protocol version during initialization. Observe the request fail.

> **`A-pm-names`** (draft): **The owning manager decides which names compare equal**
> Property: When resolving user choices, Symposium compares names using the package’s manager. Two spellings can identify the same package in one ecosystem and different packages in another.
> Authority: `R-ecosystems`
> - Criterion `remote-rule` [test]: Configure a manager that folds case but preserves punctuation. Look up a mixed-case package and observe that a disable entry matches another case spelling while a change from hyphen to underscore remains significant.
