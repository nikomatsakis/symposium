# Integration test audit

337 Rust test declarations in top-level `tests/`, including `tests/agent_plugins/`, Unix/Windows-conditional tests and live-agent modes. Fixture Rust source, shell helpers and `agent_harness/run_scenario.py` support those tests. This audit compares the properties and criteria in the current anchorbook with assertions in the test source. It does not claim that the live, platform-specific or network-dependent cases were executed. Ledger counts: 76 covered, 125 partial, 133 gap, 3 support.

## Offer classification and identity

An explicit disable takes precedence over use, wildcard enablement and a revocation in discovery/status. Noninteractive discovery leaves consent untouched. Selecting a registry plugin does not consent to a namesake dependency. A failed manager name comparison cannot grant enablement or bypass a disable.

Nearest anchors: A-plugin-enablement, A-pm-names, A-consent-record. Representative and related evidence (includes already-covered baseline/control cases):

- `tests/audits.rs:221` `disable_outranks_a_revocation`
- `tests/audits.rs:370` `use_refuses_a_revoked_crate`
- `tests/enablement.rs:306` `plugins_entries_match_registry_plugin_names_exactly`
- `tests/enablement.rs:1053` `use_on_a_disabled_plugin_errors`
- `tests/enablement.rs:1226` `consent_prompt_never_fires_non_interactively`
- `tests/enablement.rs:1410` `declined_dependency_plugin_stays_inactive`
- `tests/enablement.rs:1520` `status_reports_a_used_but_disabled_plugin_as_declined`
- `tests/enablement.rs:1647` `choosing_registry_plugin_does_not_enable_same_named_dependency`
- `tests/remote_pm.rs:699` `failed_normalization_cannot_bypass_disable_or_allow_cleanup`
- `tests/remote_pm_enablement.rs:191` `consent_preserves_distinct_noncargo_names_and_survives_reload`

## User selection and scope

`use` normally records one canonical, workspace-scoped selection; `--global` records an unscoped selection. Repeating a selection is idempotent, an unknown or disabled package cannot create it, and removal changes only the specified scope. Global delivery and registered user-level MCP can be installed and withdrawn outside any workspace.

Nearest anchors: A-plugin-sources, A-plugin-enablement, A-config-ownership. Representative and related evidence (includes already-covered baseline/control cases):

- `tests/enablement.rs:124` `use_records_workspace_entry_and_installs`
- `tests/enablement.rs:163` `use_global_records_unscoped_entry`
- `tests/enablement.rs:192` `use_unknown_name_errors`
- `tests/enablement.rs:221` `use_registry_content_is_a_noop`
- `tests/enablement.rs:343` `use_global_installs_registry_content_for_the_user`
- `tests/enablement.rs:366` `a_global_package_reaches_a_workspace_it_does_not_match`
- `tests/enablement.rs:401` `use_global_installs_a_crate_under_its_version`
- `tests/enablement.rs:431` `use_global_again_changes_nothing`
- `tests/enablement.rs:575` `use_global_outside_a_workspace_installs_for_the_user`
- `tests/enablement.rs:669` `use_global_outside_a_workspace_evaluates_custom_predicates`
- `tests/enablement.rs:728` `use_global_outside_a_workspace_registers_mcp_servers_for_the_user`
- `tests/enablement.rs:816` `use_remove_unregisters_the_mcp_servers_of_a_nested_registry_entry`
- `tests/enablement.rs:879` `use_remove_reaps_and_then_errors`
- `tests/enablement.rs:916` `use_remove_respects_scope`
- `tests/enablement.rs:1053` `use_on_a_disabled_plugin_errors`

## Whole-plugin delivery

Delivery scope and agent determine the actual installation unit and registration. A globally delivered plugin contains its entire skill set even where plugin/group/skill gates would hide it during workspace evaluation. A per-project fallback can preserve skills when plugin settings are unreadable or a user-owned directory prevents plugin installation. Scope moves and agent removal retire obsolete owned installs. Agent configuration relocation is honored.

Nearest anchors: A-agent-delivery, A-flow-activation. Representative and related evidence (includes already-covered baseline/control cases):

- `tests/agent_plugins/claude.rs:47` `project_plugins_reach_claude_as_plugins_in_the_project`
- `tests/agent_plugins/claude.rs:124` `moving_a_plugin_between_scopes_leaves_nothing_behind`
- `tests/agent_plugins/claude.rs:157` `per_skill_copies_from_before_give_way_to_the_project_plugin`
- `tests/agent_plugins/claude.rs:180` `removing_claude_removes_its_project_plugins`
- `tests/agent_plugins/claude.rs:206` `a_users_own_project_plugin_is_left_alone`
- `tests/agent_plugins/codex.rs:8` `codex_gets_the_entries_its_own_plugin_install_writes`
- `tests/agent_plugins/copilot.rs:27` `copilot_gets_the_entries_its_own_plugin_install_writes`
- `tests/agent_plugins/copilot.rs:86` `settings_symposium_cannot_read_are_left_alone_and_the_skills_arrive_per_project`
- `tests/agent_plugins/main.rs:72` `every_delivering_agent_keeps_the_delivery_contracts`
- `tests/agent_plugins/main.rs:176` `a_global_plugin_carries_all_its_skills_whatever_the_workspace`
- `tests/agent_plugins/main.rs:239` `a_skill_an_agent_takes_as_a_plugin_does_not_rename_its_namesake`
- `tests/agent_plugins/main.rs:308` `an_unreadable_registry_keeps_global_plugins_until_a_sync_can_read_it`
- `tests/agent_plugins/main.rs:360` `an_entry_the_copy_skips_is_reported_once_per_sync`
- `tests/enablement.rs:343` `use_global_installs_registry_content_for_the_user`
- `tests/enablement.rs:366` `a_global_package_reaches_a_workspace_it_does_not_match`
- `tests/enablement.rs:401` `use_global_installs_a_crate_under_its_version`
- `tests/enablement.rs:431` `use_global_again_changes_nothing`
- `tests/enablement.rs:575` `use_global_outside_a_workspace_installs_for_the_user`
- `tests/enablement.rs:669` `use_global_outside_a_workspace_evaluates_custom_predicates`
- `tests/enablement.rs:728` `use_global_outside_a_workspace_registers_mcp_servers_for_the_user`
- `tests/init_sync.rs:328` `sync_installs_antigravity_at_project_scope`
- `tests/init_sync.rs:380` `sync_gives_opencode_a_skills_folder_of_its_own`
- `tests/init_sync.rs:414` `sync_installs_skills`
- `tests/init_sync.rs:721` `copilot_uses_vendor_neutral_skill_path`

## Native plugin package input

Portable Agent Plugins packages contribute supported skills and MCP servers; unsupported components and out-of-package skill links are skipped and reported. An exported stdio server preserves its package-relative working directory and private PATH. A foreign plugin.json in a member does not erase otherwise discoverable skills.

Nearest anchors: A-pm-offer, A-path-entries, A-cargo-embedded. Representative and related evidence (includes already-covered baseline/control cases):

- `tests/agent_plugin_mcp_cwd.rs:84` `exported_mcp_server_preserves_the_package_working_directory`
- `tests/agent_plugin_mcp_cwd.rs:122` `exported_mcp_server_starts_with_its_private_path`
- `tests/agent_plugin_read.rs:76` `registry_packages_install_their_skills_and_report_what_they_skip`
- `tests/agent_plugin_read.rs:101` `use_wakes_a_dormant_package_and_remove_reaps_it`
- `tests/agent_plugin_read.rs:238` `a_workspace_member_with_a_foreign_plugin_json_keeps_its_skills`
- `tests/agent_plugin_read.rs:265` `a_dependency_package_waits_for_consent_and_reports_what_it_skips`
- `tests/agent_plugin_read.rs:325` `a_package_skill_link_outside_the_package_is_skipped_and_reported`
- `tests/agent_plugin_read.rs:381` `plugin_validate_reports_packages_and_fails_only_on_errors`

## Validation and import

Validation accepts a registry, skill, package, either manifest, or a Cargo crate, and consistently obtains missing TOML identity from a supported JSON manifest. Warnings are distinguishable from validation errors. Import creates a usable package from supported Claude skills/commands/hooks and reports entries it cannot translate.

Nearest anchors: A-pm-offer, A-installation-runnable. Representative and related evidence (includes already-covered baseline/control cases):

- `tests/agent_plugin_read.rs:381` `plugin_validate_reports_packages_and_fails_only_on_errors`
- `tests/agent_plugin_validation.rs:58` `registry_parent_validates_a_toml_plugin_with_json_identity`
- `tests/agent_plugin_validation.rs:68` `direct_toml_validation_uses_json_identity_fallback`
- `tests/agent_plugin_validation.rs:73` `bare_toml_filename_validation_uses_json_identity_fallback`
- `tests/agent_plugin_validation.rs:81` `package_directory_validation_accepts_both_manifests`
- `tests/agent_plugin_validation.rs:86` `direct_json_validation_accepts_a_package_with_toml`
- `tests/plugin_import.rs:60` `import_writes_a_package_and_reports_what_it_leaves_out`
- `tests/plugin_import.rs:179` `an_imported_package_installs_its_skills_and_runs_its_hooks_in_claude`
- `tests/plugin_import.rs:214` `import_preserves_a_command_with_windows_line_endings`
- `tests/plugin_validate.rs:37` `validates_a_registry_folder`
- `tests/plugin_validate.rs:51` `validates_a_plugin_folder`
- `tests/plugin_validate.rs:60` `validates_a_skill_folder`
- `tests/plugin_validate.rs:69` `validates_a_crate_with_package_metadata`
- `tests/plugin_validate.rs:78` `validates_a_crate_manifest_without_a_name`

## Audit lookup and retirement

Revocations can match a release or a loaded payload digest; a declined offer outranks a revoked status. A fetched audit verdict is visible during status and consent without granting permission. Unfetched audit lists are separate from unreadable plugin registries and provide no plugins. Revocations retire the package’s own installed skills and MCP entries even when ordinary cleanup is suspended, and a rejected `use` leaves configuration unchanged.

Nearest anchors: A-plugin-revocation, A-flow-consent, A-config-ownership. Representative and related evidence (includes already-covered baseline/control cases):

- `tests/audits.rs:197` `a_crate_revoked_by_version_does_not_load`
- `tests/audits.rs:208` `a_crate_revoked_by_digest_does_not_load`
- `tests/audits.rs:221` `disable_outranks_a_revocation`
- `tests/audits.rs:252` `status_says_an_active_payload_is_audited`
- `tests/audits.rs:284` `a_candidate_awaiting_consent_shows_its_audit_answer`
- `tests/audits.rs:325` `the_session_start_hint_says_whether_a_candidate_is_audited`
- `tests/audits.rs:370` `use_refuses_a_revoked_crate`
- `tests/audits.rs:430` `revocations_do_not_apply_to_registry_plugins`
- `tests/audits.rs:453` `a_revoked_crate_leaves_hook_stdout_alone`
- `tests/audits.rs:500` `a_revocation_removes_installed_skills_even_when_a_registry_is_unreadable`
- `tests/audits.rs:630` `a_revocation_unregisters_the_crates_mcp_servers_at_user_scope`
- `tests/audits.rs:687` `an_unfetched_list_answers_not_audited_and_does_not_hold_back_cleanup`
- `tests/audits.rs:743` `content_in_the_list_is_never_loaded_as_a_plugin`
- `tests/audits.rs:781` `plugin_digest_prints_the_digest_or_why_there_is_none`
- `tests/audits_crates_io.rs:109` `a_used_crate_that_is_not_a_dependency_can_be_revoked`
- `tests/audits_crates_io.rs:155` `a_digest_revocation_checks_the_release_that_loaded`

## Source acquisition and cancellation

Interrupting acquisition or timing out an external PM or installer stops its descendants. A canceled PM request does not silently respawn the process within the same instance. Runtime shutdown does not wait indefinitely for a timed-out Git probe.

Nearest anchors: A-installation-setup, A-pm-protocol. Representative and related evidence (includes already-covered baseline/control cases):

- `tests/cargo_descendant_timeout.rs:72` `timed_out_binstall_stops_installer_descendants`
- `tests/cargo_descendant_timeout.rs:77` `timed_out_cargo_install_stops_installer_descendants`
- `tests/git_acquisition_timeout.rs:88` `timed_out_git_probe_does_not_hold_runtime_shutdown`
- `tests/installer_interrupt.rs:207` `ctrl_c_stops_source_acquisition`
- `tests/installer_interrupt.rs:212` `ctrl_c_stops_an_installer_in_the_foreground_group`
- `tests/installer_interrupt.rs:217` `ctrl_c_stops_session_start_refresh`
- `tests/remote_pm_cancellation.rs:9` `a_timed_out_remote_pm_stops_its_child_processes`
- `tests/remote_pm_cancellation.rs:147` `abandoning_initialize_stops_descendants_and_does_not_respawn`
- `tests/remote_pm_cancellation.rs:152` `abandoning_an_operation_stops_descendants_and_does_not_respawn`
- `tests/remote_pm_cancellation.rs:157` `refusing_a_handshake_stops_descendants`

## Custom predicate invocation

An extension predicate can be provided by a different plugin; its declared command receives the caller’s predicate argument and its result gates the consuming skill.

Nearest anchors: A-predicate-extension, A-predicate-grammar. Representative and related evidence (includes already-covered baseline/control cases):

- `tests/custom_predicates.rs:95` `sync_custom_predicate_receives_correct_argument`
- `tests/custom_predicates.rs:129` `sync_custom_predicate_wrong_argument_fails`

## CLI lookup and reporting

Search groups case-insensitive registry hits by origin; status explains a package’s state, format, provenance and applicable version, including dormant or declined selections. Available plugin subcommands, including chained plugins’ commands, participate in dispatch and audience-grouped help. An agent is directed to that help when crate-aware commands apply. The two CLI names parse the same commands and share setup state. Machine-readable report levels expose installations and the decisions to install or skip.

Nearest anchors: A-plugin-sources, A-plugin-enablement, A-package-owner. Representative and related evidence (includes already-covered baseline/control cases):

- `tests/agent_plugin_read.rs:179` `status_and_search_show_a_package_with_its_version_and_format`
- `tests/binary.rs:22` `both_binaries_parse_every_subcommand_the_same`
- `tests/binary.rs:89` `help_names_the_invoked_binary`
- `tests/binary.rs:132` `symposium_init_after_cargo_agents_init_changes_nothing`
- `tests/dispatch.rs:4` `help`
- `tests/dispatch.rs:22` `unknown_command`
- `tests/enablement.rs:945` `search_finds_registry_content_tagged_by_origin`
- `tests/enablement.rs:984` `search_lists_a_registry_entry_once`
- `tests/enablement.rs:1007` `search_renders_grouped_by_origin`
- `tests/enablement.rs:1091` `status_reports_candidate_then_used`
- `tests/enablement.rs:1157` `status_reports_dormant_registry_plugin`
- `tests/enablement.rs:1520` `status_reports_a_used_but_disabled_plugin_as_declined`
- `tests/enablement.rs:1555` `status_lists_each_declined_name_once`
- `tests/enablement.rs:1593` `status_reports_a_pluginless_dependency_only_when_used`
- `tests/help_render.rs:13` `help_lists_plugin_vended`
- `tests/hook_agent.rs:39` `session_start_hints_discovery_when_subcommands_apply`
- `tests/init_sync.rs:3441` `report_json_info_emits_installed_events`
- `tests/init_sync.rs:3488` `report_json_debug_emits_decision_events`
- `tests/init_sync.rs:3530` `report_json_shows_skipped_skills`
- `tests/subcommand_dispatch.rs:17` `dispatches_known_subcommand`
- `tests/subcommand_dispatch.rs:36` `help_shows_plugin_subcommand`
- `tests/subcommand_dispatch.rs:82` `dispatches_subcommand_from_chained_crate`
- `tests/subcommand_dispatch.rs:102` `help_shows_chained_crate_subcommand`
- `tests/subcommand_dispatch.rs:120` `unknown_subcommand_errors`

## Agent input and hook execution

The event’s payload directory scopes hook predicates and process execution; manifest-relative resources and the native plugin-root variable resolve separately. Requirements expose named helper paths and the hook receives positional arguments. An adapter maps native occurrences to Symposium events, including suppressing later Antigravity invocations within one turn and delivering Codex Stop.

Nearest anchors: A-flow-session, A-hook-selection, A-installation-runnable. Representative and related evidence (includes already-covered baseline/control cases):

- `tests/hook_antigravity.rs:94` `hook_predicates_hold_in_the_payload_workspace`
- `tests/hook_antigravity.rs:113` `plugin_hooks_run_in_the_payload_workspace`
- `tests/hook_format_priority.rs:88` `claude_only_hook_fires_on_claude_with_its_plugin_root`
- `tests/plugin_dispatch.rs:11` `inline_shell_hook_emits_context`
- `tests/plugin_dispatch.rs:105` `hook_with_requirements_runs`
- `tests/plugin_dispatch.rs:135` `shell_hook_receives_positional_args`
- `tests/plugin_dispatch.rs:263` `helper_env_var_is_set_for_hook`
- `tests/plugin_dispatch.rs:332` `later_invocations_of_a_turn_reach_no_plugin_hook`
- `tests/plugin_dispatch.rs:374` `codex_stop_reaches_plugin_stop_hooks`
- `tests/plugin_own_script.rs:45` `scripts_run_under_a_relative_symposium_home_from_another_workspace`

## Host-specific hook outputs and exits

Host mapping covers more than a portable tool denial: native ask and user messages survive context aggregation, an empty Antigravity decision is handled, and a blocking Stop continues its turn via stdout. Exit 2 delivers the event-specific blocking response; usable context or a denial returned with exit 1 can still contribute. A malformed portable response contributes nothing. Warnings and unavailable trace logging do not prevent a parseable reply.

Nearest anchors: A-agent-output, A-hook-results, A-flow-dispatch, A-dispatch-stdout. Representative and related evidence (includes already-covered baseline/control cases):

- `tests/hook_antigravity.rs:37` `native_ask_survives_a_context_only_hook`
- `tests/hook_antigravity.rs:56` `native_object_without_a_decision_denies`
- `tests/hook_antigravity.rs:72` `native_user_message_survives_joined_context`
- `tests/hook_antigravity.rs:132` `a_blocking_stop_hook_continues_the_turn`
- `tests/hook_context.rs:64` `native_plain_text_is_context`
- `tests/hook_context.rs:124` `prompt_context_survives_a_non_zero_exit`
- `tests/hook_decisions.rs:113` `plugin_exit_2_denies_the_tool_call_on_every_agent`
- `tests/hook_decisions.rs:132` `plugin_deny_survives_a_non_zero_exit`
- `tests/hook_decisions.rs:190` `portable_output_that_is_not_canonical_is_ignored`
- `tests/hook_decisions.rs:218` `plugin_exit_2_on_a_prompt_passes_through`
- `tests/hook_decisions.rs:239` `hook_command_exits_2_with_the_reason_on_stderr`
- `tests/hook_stdout.rs:78` `hook_runs_when_the_logs_dir_is_unwritable`

## Hook and PM deadlines

Handler preparation shares the total dispatch budget. A hook invocation bounds external-manager work across handshakes, operations and multiple managers so a silent manager leaves time for healthy plugins to answer before the host deadline.

Nearest anchors: A-hook-results, A-pm-protocol. Representative and related evidence (includes already-covered baseline/control cases):

- `tests/hook_timeout.rs:52` `hook_setup_respects_the_total_budget`
- `tests/hook_timeout.rs:84` `timed_out_hook_fails_open`
- `tests/remote_pm.rs:203` `a_pm_that_never_answers_times_out`
- `tests/remote_pm_hook_deadline.rs:152` `a_silent_pm_preserves_codex_hook_output_before_its_deadline`
- `tests/remote_pm_hook_deadline.rs:157` `a_silent_pm_preserves_copilot_hook_output_before_its_deadline`
- `tests/remote_pm_hook_deadline.rs:162` `several_silent_pms_share_one_hook_budget`
- `tests/remote_pm_hook_deadline.rs:167` `initialization_and_operations_share_one_hook_budget`

## Agent configuration and hook registration

`init` distinguishes detected agents from explicit choices, retains that distinction across reruns/migration, and handles explicit adds/removes and unknown names. Noninteractive init needs detection or an explicit choice. Hooks default to global scope and a selected project scope is preserved. A registration failure for one agent does not abort skill sync or replace malformed settings.

Nearest anchors: A-agent-delivery, A-config-unreadable. Representative and related evidence (includes already-covered baseline/control cases):

- `tests/init_sync.rs:115` `init_records_detected_agents_on_first_run`
- `tests/init_sync.rs:130` `init_with_flags_does_not_record_detected_agents`
- `tests/init_sync.rs:145` `init_defaults_to_global_hooks`
- `tests/init_sync.rs:165` `init_hook_scope_project`
- `tests/init_sync.rs:188` `init_preserves_existing_hook_scope`
- `tests/init_sync.rs:207` `init_creates_config`
- `tests/init_sync.rs:224` `non_interactive_init_requires_detection_or_an_explicit_agent`
- `tests/init_sync.rs:507` `sync_continues_when_an_agents_hooks_cannot_be_registered`
- `tests/init_sync.rs:740` `removing_agent_removes_hooks`
- `tests/init_sync.rs:785` `removing_an_agent_reaps_its_skills_in_the_workspace`
- `tests/init_sync.rs:889` `add_agent_is_additive`
- `tests/init_sync.rs:945` `sync_skips_an_unknown_agent`
- `tests/init_sync.rs:967` `init_removes_an_unknown_agent`
- `tests/init_sync.rs:990` `init_rejects_removing_an_unknown_unconfigured_agent`
- `tests/init_sync.rs:3755` `init_migrates_existing_choices_and_persists_explicit_add_remove`
- `tests/init_sync.rs:3783` `init_keeps_automatic_choices_automatic_on_rerun`

## Skill origins, collisions and copying

At each destination, a shared canonical skill source installs once. Distinct origins with the same skill name coexist with suffixes; removing the conflict restores the simple name. Existing user content and dangling links reserve their paths. Concurrent syncs preserve complete copies. Installed material is gitignored. Allowed source symlinks are copied; rejected, broken or out-of-package entries are reported while valid content still arrives.

Nearest anchors: A-installation-sync, A-agent-delivery. Representative and related evidence (includes already-covered baseline/control cases):

- `tests/agent_plugin_read.rs:325` `a_package_skill_link_outside_the_package_is_skipped_and_reported`
- `tests/agent_plugins/main.rs:239` `a_skill_an_agent_takes_as_a_plugin_does_not_rename_its_namesake`
- `tests/agent_plugins/main.rs:360` `an_entry_the_copy_skips_is_reported_once_per_sync`
- `tests/init_sync.rs:292` `sync_reports_a_skill_whose_name_is_rejected`
- `tests/init_sync.rs:414` `sync_installs_skills`
- `tests/init_sync.rs:469` `sync_skips_invalid_skill_frontmatter`
- `tests/init_sync.rs:605` `sync_keeps_skills_seen_through_a_linked_skills_dir`
- `tests/init_sync.rs:699` `sync_cleans_up_without_a_user_plugins_directory`
- `tests/init_sync.rs:1646` `sync_installations_are_gitignored`
- `tests/init_sync.rs:1749` `sync_dedups_same_crate_origin_across_plugins`
- `tests/init_sync.rs:1791` `sync_dedups_same_source_path_across_plugins`
- `tests/init_sync.rs:1826` `sync_keeps_distinct_plugin_origins_with_same_skill_name`
- `tests/init_sync.rs:1876` `sync_demotes_to_suffixed_when_conflict_appears`
- `tests/init_sync.rs:1950` `sync_promotes_to_unsuffixed_when_conflict_disappears`
- `tests/init_sync.rs:2004` `sync_falls_back_to_hashed_name_when_user_dir_in_the_way`
- `tests/init_sync.rs:2059` `sync_falls_back_to_hashed_name_when_a_dangling_link_is_in_the_way`
- `tests/init_sync.rs:2089` `concurrent_syncs_keep_every_skill`
- `tests/init_sync.rs:2164` `sync_keeps_distinct_groups_within_one_plugin`
- `tests/init_sync.rs:2199` `sync_keeps_distinct_standalone_origins_at_different_paths`
- `tests/init_sync.rs:3576` `sync_follows_symlinks_in_skill_content`
- `tests/init_sync.rs:3670` `sync_installs_skills_from_a_symlinked_group_directory`

## Workspace-authored skills

With agents-syncing enabled, Symposium propagates workspace or member `.agents/skills` to agents reading another directory, keeps the authored original unmarked, refreshes edited copies, and removes only its copies when disabled or withdrawn. User-managed destinations are preserved.

Nearest anchors: A-installation-sync, A-flow-activation. Representative and related evidence (includes already-covered baseline/control cases):

- `tests/init_sync.rs:2240` `agents_syncing_propagates_user_authored_skill`
- `tests/init_sync.rs:2280` `agents_syncing_noop_when_only_agents_path_used`
- `tests/init_sync.rs:2320` `agents_syncing_disabled_skips_propagation`
- `tests/init_sync.rs:2347` `agents_syncing_cleans_up_removed_user_skill`
- `tests/init_sync.rs:2379` `agents_syncing_disabling_removes_previously_propagated_skills`
- `tests/init_sync.rs:2415` `agents_syncing_installs_member_agents_skills`
- `tests/init_sync.rs:2439` `agents_syncing_does_not_overwrite_user_managed_target`
- `tests/init_sync.rs:2473` `agents_syncing_detects_modified_source_skill`

## MCP inventory and cleanup

MCP registrations obey scope and dependency gates. Invalid manifests and incomplete dependency discovery preserve previously owned MCP entries. Removing an agent or losing a fully enumerated contribution removes only owned entries. Global removal can find a plugin’s server even if its registry entry is nested.

Nearest anchors: A-config-ownership, A-installation-sync. Representative and related evidence (includes already-covered baseline/control cases):

- `tests/agent_plugin_read.rs:101` `use_wakes_a_dormant_package_and_remove_reaps_it`
- `tests/enablement.rs:728` `use_global_outside_a_workspace_registers_mcp_servers_for_the_user`
- `tests/enablement.rs:816` `use_remove_unregisters_the_mcp_servers_of_a_nested_registry_entry`
- `tests/init_sync.rs:1005` `sync_filters_mcp_servers_by_crates`
- `tests/init_sync.rs:1126` `sync_preserves_mcp_servers_when_plugin_manifest_is_invalid`
- `tests/pm_failure_cleanup.rs:65` `failed_dependency_listing_keeps_the_installed_mcp_server`
- `tests/remote_pm.rs:699` `failed_normalization_cannot_bypass_disable_or_allow_cleanup`
- `tests/remote_pm_enablement.rs:139` `a_crash_during_load_preserves_installed_skills_and_mcp`
- `tests/remote_pm_enablement.rs:144` `a_timeout_during_load_preserves_installed_skills_and_mcp`
- `tests/remote_pm_enablement.rs:149` `a_package_error_does_not_disable_cleanup`
- `tests/remote_pm_enablement.rs:154` `auto_enabled_noncargo_dependencies_install_skills_and_mcp`

## Cargo skill groups and source selection

Cargo skill source entries can name a path or redirect to another crate. Explicit `skills = []` suppresses the default skills directory; malformed metadata falls back to it, and missing declared directories yield no skills. `crate info` and sync honor local members, path dependencies and patched crates.

Nearest anchors: A-cargo-embedded, A-plugin-chains, A-predicate-grammar. Representative and related evidence (includes already-covered baseline/control cases):

- `tests/init_sync.rs:1240` `sync_installs_plugin_skill_group`
- `tests/init_sync.rs:1284` `sync_installs_skill_from_crate_path`
- `tests/init_sync.rs:1498` `crate_info_resolves_patched_crate_to_local_path`
- `tests/init_sync.rs:1527` `crate_info_resolves_workspace_member_to_its_directory`
- `tests/init_sync.rs:1591` `sync_installs_skill_from_patched_crate`
- `tests/init_sync.rs:1613` `crate_info_resolves_path_dependency`
- `tests/init_sync.rs:3132` `sync_installs_skill_from_named_crate_source`
- `tests/init_sync.rs:3216` `sync_crate_metadata_explicit_optout`
- `tests/init_sync.rs:3244` `sync_crate_metadata_multiple_entries`
- `tests/init_sync.rs:3322` `sync_crate_metadata_redirect_target_optout`
- `tests/init_sync.rs:3381` `sync_crate_metadata_malformed_falls_back`
- `tests/init_sync.rs:3409` `sync_crate_metadata_missing_path_dir`

## Workspace change detection

Mid-session hooks reuse workspace inputs and dependency answers while fresh, then resolve/sync after changes. Creating a nested workspace can invalidate root selection with an unchanged parent lockfile. A broken Cargo project reports the cause rather than ordinary absence. In live-agent modes, newly applicable skills can become usable during a session or on Claude’s first prompt after session-start sync.

Nearest anchors: A-pm-workspace, A-flow-session. Representative and related evidence (includes already-covered baseline/control cases):

- `tests/hook_agent.rs:77` `agent_reads_tokio_skill_after_cargo_add`
- `tests/hook_agent.rs:108` `skill_installed_by_session_start_loads_on_the_first_prompt`
- `tests/init_sync.rs:1563` `sync_reports_why_the_workspace_could_not_be_loaded`
- `tests/init_sync.rs:2955` `auto_sync_skips_when_cargo_lock_unchanged`
- `tests/init_sync.rs:3018` `auto_sync_reruns_when_cargo_lock_changes`
- `tests/remote_pm.rs:591` `an_event_where_nothing_changed_spawns_no_package_manager`
- `tests/workspace_freshness.rs:6` `creating_a_nested_workspace_invalidates_the_hook_workspace_cache`

## Self-update and persisted state

Version checks are mode-dependent and throttled using persisted state. In warning mode the session-start response can suggest self-update; in update mode a sync or hook invocation can re-exec the installed version.

Nearest anchors: A-config-unreadable, A-flow-session. Representative and related evidence (includes already-covered baseline/control cases):

- `tests/init_sync.rs:2551` `self_update_reports_up_to_date`
- `tests/init_sync.rs:2566` `self_update_detects_newer_version`
- `tests/init_sync.rs:2601` `state_toml_update_check_throttling`
- `tests/init_sync.rs:2623` `sync_triggers_update_check`
- `tests/init_sync.rs:2657` `sync_skips_update_check_when_throttled`
- `tests/init_sync.rs:2687` `self_update_skips_check_when_disabled`
- `tests/init_sync.rs:2912` `session_start_hook_warns_about_update_in_context`
- `tests/init_sync.rs:2844` `auto_update_re_execs_on_sync`
- `tests/init_sync.rs:2859` `auto_update_re_execs_on_hook`

## Mid-session consent notices

Mid-session sync notices new pending dependency releases on a supported prompt/tool response after a change to a previously synced workspace. Delivered offers are deduplicated per release; unrelated changes do not repeat old names. Newly installed plugins can produce host-specific reload guidance. Only a final nonblocking response that can carry the notice counts as delivery; the next change can retry an undelivered one.

Nearest anchors: A-flow-consent, A-session-reload. Representative and related evidence (includes already-covered baseline/control cases):

- `tests/plugin_notice.rs:93` `a_dependency_added_by_a_tool_call_is_noticed_once`
- `tests/plugin_notice.rs:117` `a_dependency_the_user_added_is_noticed_with_their_next_prompt`
- `tests/plugin_notice.rs:136` `agents_that_read_context_after_a_tool_call_get_the_notice`
- `tests/plugin_notice.rs:157` `a_plugin_a_mid_session_sync_installs_comes_with_how_to_load_it`
- `tests/plugin_notice.rs:179` `a_workspace_seen_for_the_first_time_gets_no_notice`
- `tests/plugin_notice.rs:243` `a_dependency_auto_enable_covers_is_installed_without_a_notice`
- `tests/plugin_notice.rs:272` `a_dependency_without_plugins_gets_no_notice`
- `tests/plugin_notice.rs:291` `a_noticed_dependency_is_not_named_again_after_an_unrelated_change`
- `tests/plugin_notice.rs:315` `a_later_dependency_is_named_alone`
- `tests/plugin_notice.rs:341` `a_new_version_of_a_noticed_dependency_is_named_again`
- `tests/plugin_notice.rs:368` `a_notice_lost_to_a_blocked_prompt_is_given_on_the_next_change`
- `tests/plugin_notice.rs:395` `a_notice_on_a_prompt_blocked_by_a_decision_is_given_on_the_next_change`
- `tests/plugin_notice.rs:422` `a_block_a_later_hook_lifts_still_delivers_the_notice`
- `tests/plugin_notice.rs:450` `a_notice_the_agent_cannot_read_is_given_on_the_next_change`

## Plugin trials

A trial links the working source so edits appear on the next sync, can add temporary global enablement for dormant content, respects existing gates and names, and reports what installed agents can actually see. Ending a trial removes its own registrations and copies even when the source disappeared. Other workspaces retire obsolete copies on their next sync.

Nearest anchors: A-config-ownership, A-plugin-enablement. Representative and related evidence (includes already-covered baseline/control cases):

- `tests/plugin_try.rs:144` `try_installs_a_dormant_plugin_in_every_agent`
- `tests/plugin_try.rs:176` `try_installs_a_bare_skill`
- `tests/plugin_try.rs:199` `try_installs_an_agent_plugins_package`
- `tests/plugin_try.rs:222` `try_leaves_a_gated_plugin_to_its_gate`
- `tests/plugin_try.rs:259` `try_checks_a_gated_plugin_the_user_enabled_everywhere`
- `tests/plugin_try.rs:312` `try_checks_a_plugin_whose_custom_predicate_holds`
- `tests/plugin_try.rs:342` `try_skips_the_check_when_the_custom_predicate_fails`
- `tests/plugin_try.rs:396` `try_edits_reach_the_agents_on_the_next_sync`
- `tests/plugin_try.rs:452` `try_remove_preserves_explicitly_recreated_enablement`
- `tests/plugin_try.rs:528` `try_refuses_a_name_already_taken`
- `tests/plugin_try.rs:558` `try_remove_leaves_everything_as_before`
- `tests/plugin_try.rs:597` `try_and_remove_reach_the_user_level_inside_and_outside_a_workspace`
- `tests/plugin_try.rs:632` `try_remove_by_name_works_after_the_folder_is_gone`
- `tests/plugin_try.rs:687` `try_remove_leaves_a_user_plugin_alone`
- `tests/plugin_try.rs:710` `try_remove_cleans_other_workspaces_on_their_next_sync`
- `tests/plugin_try.rs:776` `try_reports_what_each_agent_lists`
- `tests/plugin_try.rs:806` `try_skips_agents_that_are_not_installed`

## PM failures and cleanup

A transport failure makes the PM unavailable while healthy managers can contribute; an ordinary package/method error can leave the transport usable. When required inventory is incomplete, sync preserves installed skills and MCP entries; an ordinary per-package error does not suppress cleanup. Detached calls omit the workspace. The protocol also exposes fetch and refresh results.

Nearest anchors: A-pm-protocol, A-pm-load, A-installation-sync, A-config-ownership. Representative and related evidence (includes already-covered baseline/control cases):

- `tests/pm_failure_cleanup.rs:65` `failed_dependency_listing_keeps_the_installed_mcp_server`
- `tests/remote_pm.rs:126` `list_deps_and_fetch_round_trip`
- `tests/remote_pm.rs:146` `refresh_round_trips`
- `tests/remote_pm.rs:159` `an_error_response_leaves_the_pm_usable`
- `tests/remote_pm.rs:179` `a_pm_that_exits_immediately_fails_rather_than_answering_nothing`
- `tests/remote_pm.rs:203` `a_pm_that_never_answers_times_out`
- `tests/remote_pm.rs:229` `a_pm_that_returns_garbage_fails`
- `tests/remote_pm.rs:289` `a_missing_binary_fails`
- `tests/remote_pm.rs:418` `the_cargo_pm_reports_no_deps_outside_a_workspace`
- `tests/remote_pm.rs:483` `a_configured_package_manager_that_is_missing_degrades`
- `tests/remote_pm.rs:514` `a_detached_operation_hands_the_pm_no_workspace`
- `tests/remote_pm.rs:699` `failed_normalization_cannot_bypass_disable_or_allow_cleanup`
- `tests/remote_pm.rs:739` `a_pm_without_normalization_cannot_compare_names`
- `tests/remote_pm_enablement.rs:139` `a_crash_during_load_preserves_installed_skills_and_mcp`
- `tests/remote_pm_enablement.rs:144` `a_timeout_during_load_preserves_installed_skills_and_mcp`
- `tests/remote_pm_enablement.rs:149` `a_package_error_does_not_disable_cleanup`

## Configuration location edge cases

Empty environment-variable overrides are ignored so setup falls back to the configured home and leaves the current working directory untouched.

Nearest anchors: A-config-location. Representative and related evidence (includes already-covered baseline/control cases):

- `tests/init_sync.rs:3711` `empty_env_dirs_are_ignored`

## Supporting checks

- `tests/hook_agent.rs:14` `session_start_returns_plugin_context` — Harness smoke only: asserts a hook trace exists for a prompt; the test name overstates its assertion.
- `tests/hook_events_doc.rs:54` `doc_json_examples_roundtrip` — Public hook JSON examples round-trip; a compatibility check on the design manual.
- `tests/init_sync.rs:2581` `state_toml_tracks_version` — Persisted schema version round-trip; no user-facing event on its own.
