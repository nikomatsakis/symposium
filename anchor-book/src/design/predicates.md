# Predicates

A predicate answers whether a contribution applies in an evaluation context. `depends-on` reads the dependency set. `workspace-member()` refers to the plugin’s provenance. File, environment and shell checks let a workspace supply other facts.

> **`A-predicate-expressions`** (draft): **Applicability is a boolean expression**
> Property: Authors can compose predicates with `all`, `any` and `not`. In a context with a supplied working directory, relative file checks and shell checks use that directory.
> Authority: `R-conditions`
> - Criterion `composition` [test]: With serde present and tokio absent, evaluate negation, conjunction and disjunction over those dependencies.
> - Criterion `working-directory` [test]: Put a marker in a temporary workspace and evaluate file and shell checks with that directory as context.

> **`A-predicate-extension`** (draft): **Registry plugins can supply predicates for sync**
> Property: A registry plugin can declare a named command that another plugin uses as a skill gate. During sync, a successful exit admits the contribution and a failing exit withholds it.
> Authority: `R-conditions`
> - Criterion `pass` [test]: Make the provider command exit successfully and observe the consumer skill installed.
> - Criterion `fail` [test]: Make the provider command fail in a fresh workspace and observe the consumer skill absent.

> **`A-predicate-reuse`** (draft): **Sync can reuse custom predicate results across invocations**
> Property: Sync persists custom predicate results in a workspace-specific cache. The same command can answer a later evaluation from disk. A time watch can bound reuse when a fresh context reads that cache.
> Authority: `R-conditions`
> - Criterion `cache-hit` [test]: Run a counting predicate with no watches and load the persisted cache in a fresh context. Observe one execution in total.
> - Criterion `expired` [test]: Have the predicate emit a short time watch. Load its cached result in a fresh context after expiration and observe a second execution.
