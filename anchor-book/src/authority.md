# Design basis

These draft requirements name the responsibilities behind the anchors. The source paths are relative to Symposium revision `ab351035`.

> **`R-ecosystems`** (draft): **Let ecosystems describe their packages**
> Statement: A manager supplies workspace and package facts in its own naming system. Symposium owns manager selection, validation and activation.
> Source: PM interface RFD and `symposium-sdk/src/pm/mod.rs`.

> **`R-plugins`** (draft): **Distribute capabilities together**
> Statement: A publisher can distribute several agent capabilities as one plugin and refer to another published package. Users control dependency enablement.
> Source: Registry-centric plugin distribution RFD and `src/discovery.rs`.

> **`R-conditions`** (draft): **Let authors state applicability**
> Statement: Plugin authors can gate contributions on workspace facts and extend the conditions available during sync.
> Source: `src/predicate.rs` and the predicate caching RFD.

> **`R-events`** (draft): **Dispatch plugin hooks through Symposium**
> Statement: One agent event can reach several plugins. The host must receive the decisions and context that result within its event budget.
> Source: `md/design/hook-flow.md` and `src/hook.rs`.

> **`R-adapters`** (draft): **Integrate each agent on its own terms**
> Statement: Each agent receives extensions through the formats, scopes and capabilities its adapter implements.
> Source: `md/design/agents.md` and `src/agents/`.

> **`R-storage`** (draft): **Keep user choices and installation records across invocations**
> Statement: Symposium must recover the user’s configured choices and remember which entries it wrote into other applications.
> Source: `md/design/configuration-loading.md` and `md/design/state.md`.

> **`R-installation`** (draft): **Install and withdraw owned content**
> Statement: Plugins can bring executable tools and skills. Synchronizing or withdrawing them must respect the ownership of files and user choices.
> Source: `src/installation.rs`, `src/sync.rs` and `md/design/sync-agent-flow.md`.
