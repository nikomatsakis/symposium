# Plugin managers

A package manager (PM) answers questions about a project and the plugins it can supply. For example, Cargo can offer a skill embedded in a dependency, while a registry can offer a skill about the same dependency from a different author. Symposium decides which offers to activate.

> **`A-pm-bootstrap`** (draft): **Symposium selects the initial managers**
> Property: Symposium builds the manager set from its defaults, executable commands declared in `[[package-manager]]`, and named `[[registry]]` entries. A configured `cargo` command replaces the built-in Cargo instance. Plugin discovery uses that set.
> Authority: `R-ecosystems`
> - Criterion `replace-cargo` [test]: Configure `symposium-pm-cargo` as `cargo`. Observe one Cargo manager and the workspace dependency reported by that process.

> **`A-pm-workspace`** (draft): **Managers describe the workspace**
> Property: `project_info` reports the project root, local packages and inputs used to detect change. `list_deps` reports package IDs for direct workspace dependencies. Managers can report no project. Where managers report roots, Symposium requires agreement.
> Authority: `R-ecosystems`
> - Criterion `root-disagreement` [test]: Have two managers report different roots. Request the workspace and observe an error naming both managers.
> - Criterion `no-project` [test]: Have the manager report no project and observe that the workspace has no root.

> **`A-pm-offers`** (draft): **Listing and loading answer different questions**
> Property: `active_plugins(deps)` asks which plugins a manager offers for a dependency set. `load_plugin(id, update)` asks the manager named in the ID for the plugins supplied by one package. Each result carries a package ID, content root and raw manifest. A manager may synthesize the manifest. Symposium validates it.
> Authority: `R-ecosystems`
> - Criterion `synthesized-offer` [test]: List offers from an external manager that synthesizes a manifest. Observe its named plugin and dependency gate after validation.
> - Criterion `specific-package` [test]: Request an MCP-only dependency by ID. Observe its server in the loaded plugin even though the package ships no skills.

### Other questions a manager can answer

| Query | Meaning |
| --- | --- |
| `canonical_names(names)` | Give the comparison spelling for each name, in order. Configuration and identity checks use the owning manager’s rules. |
| `search(text)` | Offer name matches for a human lookup. Dependency discovery calls `active_plugins`. |
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
