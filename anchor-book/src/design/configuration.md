# Configuration storage

`config.toml` holds user choices including agents, registries and plugin enablement. `state.toml` records what Symposium has done across runs. A `use` entry can be workspace-scoped even though the configuration file is user-wide.

> **`A-config-location`** (draft): **User choices live outside the project**
> Property: `SYMPOSIUM_HOME` selects the user directory. Otherwise an absolute `XDG_CONFIG_HOME` selects its `symposium` subdirectory, with `~/.symposium` as fallback. Cache and logs have separate XDG overrides. With only `XDG_CONFIG_HOME` set, their fallbacks are `cache/` and `logs/` under the selected configuration directory.
> Authority: `R-storage`
> - Criterion `state-home` [test]: Set an absolute `XDG_STATE_HOME` with `SYMPOSIUM_HOME` unset and resolve logs under `symposium/logs` there.

> **`A-config-unreadable`** (draft): **A missing file and a damaged file have different write semantics**
> Property: A missing configuration can be created from defaults. A file that cannot be read or parsed puts the current invocation on defaults with auto-update off. Saving is refused and sync stops before changing installed skills or hooks.
> Authority: `R-storage`
> - Criterion `missing` [test]: Start with a missing file and save the configuration. Observe `config.toml` created.
> - Criterion `bad-file` [test]: Load malformed TOML and invalid UTF-8 in separate runs. Observe auto-update off, a refused save and the original bytes.
> - Criterion `sync` [test]: Damage the configuration after installing content. Run sync and observe the existing skills and hooks retained.

> **`A-config-ownership`** (draft): **Persistent records identify what Symposium may withdraw**
> Property: The MCP ledger records the agent, destination file and server name for entries Symposium writes. Removal uses those records. Trial state also records whether `plugin try` added a global use entry, so ending the trial preserves pre-existing enablement.
> Authority: `R-storage`
> - Criterion `mcp` [test]: Remove an agent with both registered and unrelated server entries. Observe only its registered entries removed.
> - Criterion `trial` [test]: Try and remove a dormant plugin that already had a global use entry. Observe the earlier enablement preserved.
