# Installation

A plugin author declares how a hook finds its tools. Running `symposium sync` installs selected skills where each configured agent reads them.

> **`A-installation-runnable`** (draft): **A hook’s plugin-relative script path resolves from the manifest that declares it**
> Property: A hook can use a named installation or declare its runnable inline. A relative script belonging to the plugin is resolved against its declaring manifest, so it can run from a different workspace. A bare executable can be looked up on PATH.
> Authority: `R-installation`
> - Criterion `plugin-script` [test]: Invoke hooks using relative script declarations from the plugin fixture and observe both script markers.
> - Criterion `crate-script` [test]: Invoke the hook shipped inside a dependency with a relative script path in its manifest.

> **`A-installation-sync`** (draft): **Sync removes stale skills it owns and preserves user-managed skill directories**
> Property: Installed skills carry a `.symposium` marker. Sync removes stale owned copies and preserves user-managed skill directories and destination symlinks. A failed staging copy leaves the previous skill in place. An unreadable registry suspends cleanup of its previously installed skills.
> Authority: `R-installation`
> - Criterion `owned` [test]: Withdraw a skill and sync. Observe the stale installed copy removed.
> - Criterion `user` [test]: Place user-managed content and a user symlink in a skills directory. Observe both survive sync.
> - Criterion `copy-failure` [test]: Make a source file unreadable during refresh and observe the previous installed skill intact.
> - Criterion `unreadable-registry` [test]: Make a registry unreadable after installing a skill and sync. Observe the installed skill retained.

> **`A-installation-setup`** (draft): **A hook’s setup commands run before its handler and keep their stdout out of the agent’s reply**
> Property: A hook may reference a named installation or carry the setup inline. The installation’s setup commands run before the hook command. Their stdout stays out of the response that the agent parses.
> Authority: `R-installation`
> - Criterion `ordering` [test]: Have setup create the marker the hook reads. Observe the hook find the marker for both named and inline declarations.
> - Criterion `stdout` [test]: Make an install command print during a hook. Observe valid agent output without the installer’s message.

> **`A-sync-manager-failure`** (draft): **An unavailable manager suspends cleanup. A package error still permits cleanup**
> Property: If a manager fails to list dependencies or stops responding during load, sync preserves previously installed contributions that now appear absent. If the manager responds with an ordinary package error, stale skills and MCP registrations remain eligible for removal.
> Authority: `R-installation`
> - Criterion `unavailable` [test]: After installing a skill and MCP server, crash the manager or time out its load request. Both survive. A failed dependency listing also preserves the existing MCP registration.
> - Criterion `package-error` [test]: Repeat with a manager that answers the load request with a package error. Observe the stale skill and registration removed.
