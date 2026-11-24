# Hooks

The agent calls `symposium hook`. Symposium selects plugin handlers for that event, runs them and returns one response through the agent adapter. A plugin author can handle an event in Symposium’s portable format or in an agent’s native format.

> **`A-hook-selection`** (draft): **Format selection happens per plugin and event**
> Property: Eligible handlers in the calling agent’s native format take priority within a plugin. When none match, eligible portable handlers can run. A native handler for a different agent cannot serve as the fallback.
> Authority: `R-events`
> - Criterion `native` [test]: On Claude, offer matching Claude and portable handlers in one plugin. Observe only the native marker.
> - Criterion `fallback` [test]: Run the same plugin on Copilot and observe only the portable marker.
> - Criterion `gated` [test]: Make the native handler’s predicate false and observe the portable handler run.

> **`A-hook-results`** (draft): **One hook cannot erase another hook’s completed denial**
> Property: Symposium combines plugin outputs for the event. A tool denial survives a later rewrite. A handler that runs out of time contributes no decision. Context from multiple completed handlers can appear in the same response.
> Authority: `R-events`
> - Criterion `deny` [test]: Issue a denial and a later rewrite for the same tool call. Observe the denial in each tested agent response.
> - Criterion `timeout` [test]: Run a slow handler beside a denying handler and repeat with a tool matching only the slow handler. Observe the denial in the first case and ordinary continuation in the second.
> - Criterion `context` [test]: Have multiple handlers contribute context on PostToolUse and observe their messages together.
