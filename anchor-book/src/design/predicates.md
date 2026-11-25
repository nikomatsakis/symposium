# Predicates

A predicate tests whether a plugin or contribution applies here. For example, `depends-on(widget)` asks whether a workspace member directly depends on `widget`. File, environment, and shell checks can supply other conditions.

> **`A-predicate-grammar`** (draft): **Every listed predicate call must pass, and a nonempty depends-on list must match at least one entry**
> Property: Each `predicates` entry is a function call in the grammar below. Every entry must hold. Where a `depends-on` field is accepted, its entries are dependency atoms. At least one atom in a nonempty list must match, in addition to the `predicates` entries.
> Authority: `R-conditions`
> - Criterion `calls` [test]: Parse `any(depends-on(a), path_exists(rg))` as two calls inside `any`. Reject `all()` because it has no operands.
> - Criterion `atoms` [test]: Read `serde, tokio` in dependency shorthand as alternatives. Reject `depends-on(serde)` in that field.
> - Criterion `bare-name` [test]: Reject `serde`, `tokio>=1.0`, and `*` as standalone entries in `predicates`.
> - Criterion `custom-argument` [test]: Parse `foo(bar(baz))` as a call to `foo` with the text `bar(baz)` as its argument.

```text
predicate   ::= "depends-on(" atom ")" | "workspace-member()"
              | "env(" variable [ "=" value ] ")"
              | "path_exists(" text ")" | "shell(" text ")"
              | "not(" predicate ")"
              | "any(" operands ")" | "all(" operands ")"
              | custom-name "(" text ")"
operands    ::= predicate { "," predicate }
atom        ::= "*" | package-name [ requirement ]
requirement ::= ( "==" | "=" | ">=" | "<=" | ">" | "<" | "^" | "~" ) version-spec
```

`package-name` uses one or more ASCII letters, digits, hyphens, or underscores. A version requirement uses Cargo-style semver syntax and attaches directly to the name without spaces. For example, `widget=1.2.3` means compatible with `1.2.3`, and `widget==1.2.3` requires that exact release. A range can combine comparisons, as in `depends-on(widget>=1.2,<2)`. `*` matches even a workspace with no dependencies.

`custom-name` starts with an ASCII letter and continues with ASCII letters, digits, or underscores. Built-in names and the retired name `crate` are reserved. `text` is the argument between parentheses with surrounding whitespace trimmed. It may be empty or contain commas and nested parentheses. In `env`, the first `=` separates the nonempty variable name from its value, so `env(KEY=a=b)` tests for the value `a=b`.

For example, this plugin applies when the workspace directly depends on either a qualifying `widget` release or `serde`, and `CI` is unset:

```toml
depends-on = ["widget>=1.2", "serde"]
predicates = ["not(env(CI))"]
```

> **`A-predicate-expressions`** (draft): **Plugin authors combine conditions with all, any and not**
> Property: Authors can compose predicates with `all`, `any` and `not`. In a context with a supplied working directory, relative file checks and shell checks use that directory.
> Authority: `R-conditions`
> - Criterion `composition` [test]: With serde present and tokio absent, evaluate negation, conjunction and disjunction over those dependencies.
> - Criterion `working-directory` [test]: Put a marker in a temporary workspace and evaluate file and shell checks with that directory as context.

> **`A-predicate-extension`** (draft): **A registry plugin can supply a command whose successful exit lets another plugin’s skill apply**
> Property: A registry plugin can declare a named command that another plugin uses as a skill gate. During sync, a successful exit admits the contribution and a failing exit withholds it.
> Authority: `R-conditions`
> - Criterion `pass` [test]: Make the provider command exit successfully and observe the consumer skill installed.
> - Criterion `fail` [test]: Make the provider command fail in a fresh workspace and observe the consumer skill absent.

> **`A-predicate-reuse`** (draft): **Later syncs can reuse a custom predicate result in the same workspace**
> Property: Sync persists custom predicate results in a workspace-specific cache. The same command can answer a later evaluation from disk. A time watch can bound reuse when a fresh context reads that cache.
> Authority: `R-conditions`
> - Criterion `cache-hit` [test]: Run a counting predicate with no watches and load the persisted cache in a fresh context. Observe one execution in total.
> - Criterion `expired` [test]: Have the predicate emit a short time watch. Load its cached result in a fresh context after expiration and observe a second execution.

> **`A-predicate-name`** (draft): **A custom predicate name must have exactly one provider**
> Property: A uniquely defined custom predicate is available by name. If two or more providers declare the same name, Symposium reports the collision and withholds that name from lookup.
> Authority: `R-conditions`
> - Criterion `providers` [test]: Register distinct names, then the same name from two and three plugins. Only the distinct names are available.
