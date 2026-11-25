# Tested behavior missing from the anchors

The audit compared 2,230 test entries with the 39 anchors present at review time. The table below preserves those findings. For recommendations against the 54-anchor book, see the [unlinked test review](unlinked-tests.md).

## Anchors added after the audit

| Question | Anchor |
| --- | --- |
| Which `widget` does `use widget` select, and where does the selection apply? | [Package lookup](../design/plugins.md#A-plugin-use-name) · [Workspace scope](../design/plugins.md#A-plugin-use-scope) |
| When does a plugin need explicit `use`? | [Registry dormancy](../design/plugins.md#A-plugin-dormancy) |
| Which decision wins when consent rules overlap? | [Offer precedence](../design/discovery-consent.md#A-offer-precedence) |
| When will the same pending dependency appear in another notice? | [Final reply](../design/discovery-consent.md#A-notice-delivery) · [Package release](../design/discovery-consent.md#A-notice-release) |
| What survives a manager failure, and when can a hook reuse workspace discovery? | [Sync cleanup](../design/installation.md#A-sync-manager-failure) · [Workspace freshness](../design/workspace-discovery.md#A-workspace-freshness) |
| Which events match a handler, and where does it run? | [Tool matcher](../design/hooks.md#A-hook-matcher) · [Antigravity prompt event](../design/agents.md#A-agent-antigravity-prompt) · [Event directory](../design/hooks.md#A-hook-directory) |
| How long can dispatch take, and what happens to a timed-out handler’s children? | [Dispatch deadline](../design/hooks.md#A-hook-budget) · [Child commands](../design/hooks.md#A-hook-timeout-children) |
| Which skills ship in a globally installed plugin? | [Global bundle contents](../design/agents.md#A-agent-global-skills) |
| What happens when two plugins define the same predicate? | [Custom predicate names](../design/predicates.md#A-predicate-name) |

## Findings at the time of the audit

| Flow | What the current anchor leaves open | Tested behavior to pin |
| --- | --- | --- |
| [Activate a plugin](../design/plugin-activation.md#A-flow-activation) | Which plugins require explicit `use` in the first place? | A registry manifest with no qualifying gate is dormant. A dependency gate on a skill group or package reference can change that. The same gateless manifest can apply as a workspace member. Start with `dormancy_honors_gates_at_every_level` and `workspace_manifest_may_omit_dependency_gate`. |
| [Choose a package](../design/plugins.md#A-plugin-sources) | Which `widget` does an unqualified `use widget` select? | A matching registry entry wins over a same-named dependency. The dependency can still be pending. A workspace-scoped selection counts only in that workspace. See `choosing_registry_plugin_does_not_enable_same_named_dependency`. |
| [Classify an offer](../design/discovery-consent.md#A-flow-consent) | What if several saved decisions match the same package? | Discovery checks disable first, then revocation, explicit use, and standing auto-enable. A positive audit alone leaves an undecided offer pending. See `revocation_outranks_grants_and_only_disable_outranks_it` and `an_audited_offer_is_still_a_candidate`. |
| [Reconcile installed contributions](../design/installation.md#A-installation-sync) | Is a missing package authoritative when a manager fails? | If a manager crashes or times out during loading, sync preserves installed skills and MCP registrations that might still be needed. An ordinary package error leaves cleanup available. Pair `a_crash_during_load_preserves_installed_skills_and_mcp` with `a_package_error_does_not_disable_cleanup`. |
| [Deliver a consent notice](../design/discovery-consent.md#A-flow-consent) | When has the agent actually received a notice? | Delivery counts when the final, nonblocking response carries the notice in a form the agent can read. A later workspace change can retry an undelivered notice. The same pending release is suppressed after delivery; a new version can be named again. See `a_notice_the_agent_cannot_read_is_given_on_the_next_change` and `a_new_version_of_a_noticed_dependency_is_named_again`. |
| [Discover the workspace](../design/workspace-discovery.md#A-flow-workspace) | When can a later hook reuse a previous answer? | Freshness includes paths previously recorded as absent. Creating a nested `Cargo.toml` can change the workspace selected for the same working directory. See `creating_a_nested_workspace_invalidates_the_hook_workspace_cache`. |
| [Enter and dispatch a hook](../design/hook-dispatch.md#A-flow-dispatch) | Which host events count, where does the command run, and what happens when it outlives its budget? | These merit separate anchors. For example, Antigravity's later invocations in the same turn do not produce another prompt event. The event directory scopes execution, while a plugin-relative command resolves against its manifest. Command preparation shares the dispatch budget, and a timed-out command's children are stopped. See `only_the_first_invocation_of_a_turn_is_a_prompt`, `hook_setup_respects_the_total_budget`, and `timed_out_hook_stops_its_child_commands`. |
| [Deliver a plugin to an agent](../design/agents.md#A-agent-delivery) | Which skills belong to a globally delivered bundle? | A globally delivered whole plugin can carry skills that this workspace's gates would filter out of project delivery. The distinction matters even when both operations select the same plugin. See `a_global_plugin_carries_all_its_skills_whatever_the_workspace`. |

**Native hook fallback is already pinned.** The [hook selection anchor](../design/hooks.md#A-hook-selection) specifies native priority per plugin, the Symposium-format fallback, and predicate-gated native handlers. The adjoining gap is **what counts as a match**. Tests exercise regular expressions against the tool name, wildcard handling and invalid patterns. That rule now has its [own anchor](../design/hooks.md#A-hook-matcher).

## Other chapters with tested decisions

- **Plugins and skills.** Which manifest supplies identity when several formats coexist? How are names disambiguated when two installed skills collide? Which directory owns a skill reached through two paths? The tests also cover package import, validation, plugin trials, and exposing plugin subcommands in help.
- **Installation and agent setup.** Which settings belong to the user? Which agents get registered on `init`, before the first sync? Tests cover idempotent registration, preserving user-disabled MCP entries, scope changes, user-owned directories, and rollback or cleanup of managed copies.
- **Predicates.** What happens when two registries provide the same predicate name? How are arguments passed? Which changes invalidate a saved result? The current anchors cover execution and time-based reuse. The tests exercise more of the author-facing interface.
- **Cargo and the SDK.** Cache-only Cargo lookup retains the requested registry identity and chooses a fully extracted release satisfying its version constraint. A path dependency and an explicit registry requirement can resolve differently. SDK tests also pin manifest overlay behavior, protocol framing, and the serialized update levels. Search the ledger for `cached_lookup_does_not_substitute_another_registry`, `merge_appends_lists_and_overrides_scalars`, or `update_serializes_to_the_documented_spellings`.
- **Telemetry.** This is the largest unanchored component by test count. A small set of design decisions could cover opt-in recording, what can identify a user or workspace, retention, bounds on public counts, and separation of inspectable data from private state. The newer storage and schema unit tests establish component behavior. They do not demonstrate which events a real session currently emits.

## Audit scope and evidence

The reviewers compared assertions with the **properties and criteria of the 39 design anchors in the baseline**, including tests already cited by an anchor. A rule described only in surrounding prose still qualifies as an anchor gap. The classifications are review judgments. Several tests can support one proposed anchor, and one test can exercise several decisions.

| Inventory | Entries |
| --- | ---: |
| Linux workspace tests listed by `cargo nextest list --workspace`, including 32 ignored tests | 2,218 |
| Compile-only SDK doctests listed by rustdoc | 4 |
| Additional platform-gated evaluator tests | 5 |
| Hidden evaluation task checks and one ignored source illustration | 3 |
| **Total ledger entries** | **2,230** |

Of those entries, **1,157** exercise a rule classified as missing, **228** have partial coverage, **152** exercise a covered rule, and **693** are supporting checks. For the **337 top-level integration tests alone**, the breakdown is **133 missing, 125 partial, 76 covered, and 3 supporting**. The audit inspected assertions and enumerated test binaries. It did not execute the full suite or the ignored scenarios.

[Download the per-test ledger](test-inventory.tsv). Filter `coverage` by `gap` or `partial`, or search for a test name above. Each row gives the source and line, the behavior or existing anchor, any explicitly linked criteria, and the Cargo test identifier where applicable.

The detailed [integration findings](integration-audit.md), [core findings](core-audit.md), and [evaluation findings](evaluation-audit.md) group related tests into candidate design decisions. An evaluator that grades an assistant's output supplies evidence about the grader. The evaluation findings identify the smaller set of real-binary tests that also exercise Symposium itself.
