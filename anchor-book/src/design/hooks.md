# Hooks

The agent calls `symposium hook`. Symposium selects plugin handlers for that event, runs them and returns one response through the agent adapter. A plugin author can handle an event in Symposium’s portable format or in an agent’s native format.

Follow [a session starting in a workspace](session-start.md) from the agent’s invocation to the context it receives.

> **`A-hook-selection`** (draft): **For each plugin, select all matching native handlers if any match. Otherwise select all matching Symposium-format handlers**
> Property: For each active plugin, Symposium checks the event, matcher, agent restriction, and handler predicates. If any handlers in the calling agent’s native format match, Symposium selects all of those handlers. Otherwise it selects all matching `format = "symposium"` handlers. If neither group matches, the plugin contributes no handler for this event. Selection finishes before any handler runs.
> Authority: `R-events`
> - Criterion `native` [test]: On Claude, offer matching Claude and portable handlers in one plugin. Observe only the native marker.
> - Criterion `fallback` [test]: Run the same plugin on Copilot and observe only the portable marker.
> - Criterion `gated` [test]: Make the native handler’s predicate false and observe the portable handler run.
> - Criterion `all` [test]: Give one plugin two matching native handlers and one portable handler. Both native handlers are selected. Give it two portable handlers and a gated-out native handler and both portable handlers are selected.
> - Criterion `none` [test]: Restrict the plugin’s only handler to Claude. Dispatch for Codex and observe an empty selection.

> **`A-hook-results`** (draft): **A tool denial survives a later handler’s rewrite**
> Property: Symposium combines plugin outputs for the event. A tool denial survives a later rewrite. A handler that runs out of time contributes no decision. Context from multiple completed handlers can appear in the same response.
> Authority: `R-events`
> - Criterion `deny` [test]: Issue a denial and a later rewrite for the same tool call. Observe the denial in each tested agent response.
> - Criterion `timeout` [test]: Run a slow handler beside a denying handler and repeat with a tool matching only the slow handler. Observe the denial in the first case and ordinary continuation in the second.
> - Criterion `context` [test]: Have multiple handlers contribute context on PostToolUse and observe their messages together.

> **`A-hook-matcher`** (draft): **Tool hooks match every tool with * or search the tool name with a regular expression**
> Property: For a tool event, `*` matches every tool. Any other matcher is a regular expression searched against the tool name. For an event without a tool, every valid matcher passes. An invalid matcher makes the manifest invalid.
> Authority: `R-events`
> - Criterion `event` [test]: Match a tool name against accepting and rejecting patterns, then apply a tool-specific pattern to session start.
> - Criterion `invalid` [test]: Load a manifest with an unbalanced matcher expression and observe a validation error.

> **`A-hook-budget`** (draft): **Plugin handlers share eight seconds, with at most five seconds per running handler**
> Property: Command preparation and execution share eight seconds across the selected handlers. Each running handler gets at most five seconds and at most the remaining time. A timed-out handler contributes no result.
> Authority: `R-events`
> - Criterion `setup` [test]: Delay installation for nine seconds and observe dispatch return within the shared budget.
> - Criterion `timeout` [test]: Hang one handler. Observe a later denial when time remains and ordinary continuation when only the hanging handler matches.

> **`A-hook-directory`** (draft): **A hook command runs in the event directory**
> Property: When the event supplies an existing working directory, Symposium runs the selected command there. A plugin-relative script path still resolves from its declaring manifest.
> Authority: `R-events`
> - Criterion `directory` [test]: Invoke from a different directory and observe the handler read its workspace marker. Run a plugin-relative script from another workspace.

> **`A-hook-timeout-children`** (draft): **Timing out a hook stops its child commands on Unix**
> Property: On Unix, timing out a running handler also stops child commands in its process group.
> Authority: `R-events`
> - Criterion `child` [test]: Have a child write a file after the handler deadline. After that delay the file is still absent.
