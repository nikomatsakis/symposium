# Agent support

An adapter selects where Symposium registers hooks and extensions, which event formats it accepts, and how a decision reaches the agent. The same plugin can therefore have different installed files and wire responses on different agents.

> **`A-agent-output`** (draft): **The adapter encodes the tool decision for its host**
> Property: For a portable PreToolUse denial, Claude reads the decision from JSON and Kiro reads exit status 2 with the reason on stderr. A rewrite on Claude requests confirmation. The Codex and Kiro adapters omit a rewrite they cannot deliver while preserving the user’s approval decision.
> Authority: `R-adapters`
> - Criterion `block` [test]: Return exit 2 from a blocking plugin and observe the host-specific reason and decision from each tested adapter.
> - Criterion `rewrite` [test]: Request a portable rewrite and inspect the reply for Claude, Codex and Kiro.

> **`A-agent-delivery`** (draft): **Each adapter selects the installation unit its agent reads**
> Property: An adapter that delivers a selected skill as part of a plugin suppresses the redundant per-skill copy for that destination. A second agent using a separate skills directory still receives its own copy.
> Authority: `R-adapters`
> - Criterion `two-agents` [test]: Deliver a global plugin to a plugin-capable agent, then add OpenCode. Observe the plugin, OpenCode’s skill, and the absence of the redundant copy.

> **`A-agent-global-skills`** (draft): **A globally installed whole plugin carries all its skills**
> Property: When an adapter installs a whole plugin for the user, its bundle includes every skill regardless of whether its workspace conditions hold during sync.
> Authority: `R-adapters`
> - Criterion `bundle` [test]: Globally select a plugin with dependency and environment conditions on its skills. Inspect the installed Claude plugin and find all three skills.

> **`A-agent-antigravity-prompt`** (draft): **Antigravity invocation zero dispatches the prompt hook**
> Property: For each Antigravity `PreInvocation`, Symposium dispatches `UserPromptSubmit` when `invocationNum` is zero or omitted. It skips later invocations in the turn.
> Authority: `R-adapters`
> - Criterion `counter` [test]: Submit counters 0, 1 and 7 and a payload without a counter. Only zero and the missing counter dispatch.
