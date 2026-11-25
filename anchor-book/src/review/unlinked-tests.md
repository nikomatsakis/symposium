# Unlinked tests

[Download the recommendations for every unlinked test](unlinked-tests.tsv). Search by test name or filter the `review` column. Each row gives the assertion a reviewer relied on, any existing anchors it supports, and the tested behavior those anchors leave open. Source locations and Cargo test names let you find or run the example.

| Recommendation | Meaning | Tests |
| --- | --- | ---: |
| Link existing | The assertions support an existing property. The reviewer found no additional design rule in this test. | 109 |
| Link and specify | The test supports an existing property and also checks an outcome the book leaves open. | 166 |
| Specify behavior | The test checks a design choice for which the reviewer found no existing anchor to link. | 939 |
| Supporting check | The assertion concerns a fixture, evaluator, or implementation detail. The reviewer recommends no product anchor. | 905 |

A recommended link can support one branch of a property. For example, a test that checks what happens when a release is revoked need only exercise that branch to supply evidence for revocation. A test of the full priority order also needs cases where conditions overlap. An empty `recommended_anchors` field records that the reviewer found no existing fit. An empty `uncovered_behavior` field records that they found no additional design promise to specify.

For a straightforward link, `use_matches_registry_plugin_names_exactly` tries an underscore alias for a hyphenated registry plugin. Lookup fails and the alias never enters the configuration. This exercises [manager name equality](../design/plugin-managers.md#A-pm-names) and the failure branch of [plugin selection by name](../design/plugins.md#A-plugin-use-name).

Test names can overstate the evidence. `session_start_returns_plugin_context` sends a user prompt and asserts that at least one hook trace exists. Linking it as evidence that SessionStart returned plugin context would promise more than the assertion checks.

## Decisions to discuss

- **When can a skill be removed during an incomplete sync?** A fixture installs `crate-d`, then revokes it and makes a registry unreadable. Sync removes `crate-d`’s skill and preserves an unrelated stale skill. The test supplies a precedence rule between [revocation](../design/plugins.md#A-plugin-revocation) and [cleanup after an unreadable registry](../design/installation.md#A-installation-sync). Search for `a_revocation_removes_installed_skills_even_when_a_registry_is_unreadable`.
- **Which Cargo version does a registry request use?** Three cases specify a single ranked choice. An explicit requirement wins. Otherwise Cargo pins a matching workspace dependency to its resolved version. With neither, Cargo uses `*`. The [Cargo chapter](../design/cargo.md) explains which packages Cargo discovers. The registry requirement is a separate decision. Search for `registry_spec_prefers_explicit_version`, `registry_spec_pins_workspace_version_exactly`, and `registry_spec_falls_back_to_wildcard`.
- **What does registering an agent install before its first sync?** An ignored real-binary fixture checks that `init --add-agent claude` registers the global hook while the home and project skill directories remain empty. The [agent delivery anchor](../design/agents.md#A-agent-delivery) describes where selected skills go. The timing of registration and delivery merits its own statement. Search for `a_symposium_trial_registers_hooks_without_syncing`.
- **When does telemetry stop recording individual extension targets?** `the_129th_public_target_joins_one_overflow_row` fills the daily allowance of 128 public rows. An already admitted target keeps its row. Additional distinct targets share an overflow row that omits their identities. This component test can support an anchor about aggregation. The running application currently supplies no observations to this component.

## Which operations record telemetry today?

Enabling telemetry saves the user preference. Neither shipped CLI writes usage events. Both enter the same dispatcher. Its hook and sync paths never call the event writer or retention routine, and the newer aggregate storage has no production caller. The telemetry commands read the preference and existing daily event files or update the preference.

A fresh Rust workspace exercised the built `symposium` binary with Claude configured, auto-sync on, and telemetry enabled. `SessionStart` scanned the workspace and returned a skill-reload response. An explicit `sync` also completed. A `PreToolUse(Read)` handler from the `plugin-own-script0` fixture returned `own-script-fired`. `UserPromptSubmit`, `PostToolUse`, `Stop`, and `status` also completed. The hook trace captured the events. The telemetry directory and private telemetry state file were still absent. After seeding one expired event file and repeating `SessionStart`, `telemetry show` returned just the seeded event and `telemetry status` counted one. Session start neither appended a row nor expired the old file.

The recording RFD assigns future producers to the outer hook wrapper, plugin dispatcher, full sync, Claude skill-use adapter and top-level command dispatcher. It reserves production activation for step 7, after producers and versioned consent are in place. The flow diagrams describe the current implementation, so a telemetry write does not belong on those diagrams yet.

## Scope

The reviewed checkout has 54 design anchors. The source inventory contains 2,230 entries. Exactly 111 tests carry explicit evidence links, leaving **2,119 unlinked entries**. The full identifiers of the 111 exclusions agree with the Anchor Book verification run, which passed all 95 criteria.

Linux `cargo nextest list --workspace` enumerated 2,218 tests. Of these, 2,107 lack links, including all 32 ignored by default. The inventory also includes five tests gated to other platforms, four compile-only SDK doctests, two hidden evaluation-task checks and one ignored documentation illustration. Each row retains its inventory status. The reviewers inspected assertions. For an ignored scenario, the recommendation describes what it checks when run.

The full default Linux run passed all 2,186 tests it executed and skipped 32. All four workspace doctests passed.

The report is a snapshot of the working tree. It proposes links to anchor **properties**. Adding a link in source may require a new criterion under that property, especially for an outcome the existing examples do not exercise. The current verification configuration discovers annotations under `src/` and `tests/`. A test in another crate needs that source included before it can serve as executable evidence there.
