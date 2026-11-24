# Plugins

A plugin groups contributions under one identity. It can carry skills, hooks, MCP servers and CLI subcommands together. The distinction between a workspace plugin, a dependency offer and an explicitly selected plugin matters at activation.

> **`A-plugin-enablement`** (draft): **Discovery offers a dependency plugin for a user decision**
> Property: A newly discovered dependency plugin can be pending. Approval enables its contributions. Declining suppresses another offer for that dependency. A dormant registry plugin can be selected explicitly with `use`.
> Authority: `R-plugins`
> - Criterion `approve` [test]: Approve the fixture dependency, sync and observe its skill, MCP server, hook and subcommand.
> - Criterion `decline` [test]: Decline the dependency and sync with wildcard auto-enable. Observe no active contributions or repeat offer.
> - Criterion `dormant` [test]: Use a dormant registry plugin and then remove the use entry. Observe it activate and return to dormancy.

> **`A-plugin-chains`** (draft): **Plugin references expand to a fixed point**
> Property: An applicable plugin can name other Cargo packages through `[[plugins]]`. Symposium follows applicable references and expands each reached package once per manager and canonical name. A cycle terminates and two paths to a package share its contributions.
> Authority: `R-plugins`
> - Criterion `cycle` [test]: Sync A → B → A and observe B’s skill installed.
> - Criterion `diamond` [test]: Reach one package through two branches and count one installed copy of its skill.

> **`A-plugin-revocation`** (draft): **A configured audit revocation withholds the matching Cargo package**
> Property: Symposium checks reached Cargo packages against the configured audits list before evaluating their own predicates or expanding their references. A plugin already in the graph can therefore lose access to a chained package that was previously available.
> Authority: `R-plugins`
> - Criterion `chain` [test]: Reach a revoked crate through a plugin reference. Observe its skill withheld.
> - Criterion `predicate` [test]: Retire an installed revoked crate carrying a predicate with a visible side effect. Observe retirement without executing the predicate.
