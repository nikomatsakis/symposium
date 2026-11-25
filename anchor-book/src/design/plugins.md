# Plugins

A plugin groups contributions under one identity. It can carry skills, hooks, MCP servers and CLI subcommands together. The distinction between a workspace plugin, a dependency offer and an explicitly selected plugin matters at activation.

> **`A-plugin-sources`** (draft): **Symposium starts with registry and workspace plugins. User choices can add packages**
> Property: Symposium considers plugins from configured registries and the workspace root and members. For additional packages, `auto-enable` selects matching direct dependencies. An applicable `use` entry can select a package outside that dependency set. A disabled package is excluded from this selection. An active plugin can separately bring in packages through its applicable references.
> Authority: `R-plugins`
> - Criterion `selection` [test]: Configure `auto-enable`, `use`, and `disable` for different dependency names. Inspect the selected package IDs and observe the enabled dependencies, with the disabled and undecided ones omitted.

Follow [Activating a plugin](plugin-activation.md#A-flow-activation) to see when a selected package's contributions become eligible.

> **`A-plugin-enablement`** (draft): **Approving a pending dependency enables its plugin. Declining suppresses repeat offers**
> Property: A newly discovered dependency plugin can be pending. Approval enables its contributions. Declining suppresses another offer for that dependency. A dormant registry plugin can be selected explicitly with `use`.
> Authority: `R-plugins`
> - Criterion `approve` [test]: Approve the fixture dependency, sync and observe its skill, MCP server, hook and subcommand.
> - Criterion `decline` [test]: Decline the dependency and sync with wildcard auto-enable. Observe no active contributions or repeat offer.
> - Criterion `dormant` [test]: Use a dormant registry plugin and then remove the use entry. Observe it activate and return to dormancy.

> **`A-plugin-chains`** (draft): **Symposium follows applicable plugin references and expands each reachable package once**
> Property: An applicable plugin can name other Cargo packages through `[[plugins]]`. Symposium follows applicable references and expands each reached package once per manager and canonical name. A cycle terminates and two paths to a package share its contributions.
> Authority: `R-plugins`
> - Criterion `cycle` [test]: Sync A → B → A and observe B’s skill installed.
> - Criterion `diamond` [test]: Reach one package through two branches and count one installed copy of its skill.

> **`A-plugin-revocation`** (draft): **A configured audit revocation withholds the matching Cargo package**
> Property: Symposium checks reached Cargo packages against the configured audits list before evaluating their own predicates or expanding their references. A plugin already in the graph can therefore lose access to a chained package that was previously available.
> Authority: `R-plugins`
> - Criterion `chain` [test]: Reach a revoked crate through a plugin reference. Observe its skill withheld.
> - Criterion `predicate` [test]: Retire an installed revoked crate carrying a predicate with a visible side effect. Observe retirement without executing the predicate.

> **`A-plugin-dormancy`** (draft): **Registry plugins require explicit use unless they declare a dependency condition or directly list a custom predicate**
> Property: A registry plugin requires `use` unless a dependency condition appears on the plugin, a skill group, a hook, an MCP server, or a package reference, or the plugin directly lists a custom predicate. Workspace membership satisfies this explicit-use requirement.
> Authority: `R-plugins`
> - Criterion `gates` [test]: Compare an ungated registry manifest with manifests gated on the plugin, a skill group, or a package reference. Include a hook, an MCP server, and a direct custom predicate as alternative gates. Only the ungated manifest requires `use`.
> - Criterion `origin` [test]: Load the same ungated manifest from a registry and a workspace member. Only the registry copy requires `use`.
> - Criterion `selection` [test]: Select a different plugin, then this plugin. Only the latter selection wakes the dormant plugin.

> **`A-plugin-use-name`** (draft): **For use NAME, Symposium takes the first matching known plugin or falls back to the first manager that finds NAME**
> Property: For `use NAME`, Symposium takes the first matching known plugin, then the first package manager that finds the name if no plugin matched. If neither finds it, `use` fails. Only the selected package identity receives that grant.
> Authority: `R-plugins`
> - Criterion `collision` [test]: Give a registry plugin and a Cargo dependency the same name. Select the name and observe the registry plugin selected while the dependency remains pending.
> - Criterion `lookup` [test]: Look up names found only in Cargo, only in the registry, in both, and in neither.

> **`A-plugin-use-scope`** (draft): **A workspace `use` grants consent only in that workspace**
> Property: An explicit `use` scoped to a workspace counts for discovery only when the workspace matches.
> Authority: `R-plugins`
> - Criterion `workspace` [test]: Select a dependency in one workspace and observe its skill installed. Run discovery with a `use` scoped to a different workspace and observe the offer remains pending.
