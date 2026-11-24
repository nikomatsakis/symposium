# Installation

Installation has two audiences. A plugin author declares how to acquire and run supporting tools. A user runs `sync` to place selected content where the configured agents read it. Fetching a tool and owning an installed skill are separate operations.

> **`A-installation-runnable`** (draft): **A plugin can ship the tool its hook invokes**
> Property: A hook can use a named installation or declare its runnable inline. A relative script belonging to the plugin is resolved against its declaring manifest, so it can run from a different workspace. A bare executable can be looked up on PATH.
> Authority: `R-installation`
> - Criterion `plugin-script` [test]: Invoke hooks using relative script declarations from the plugin fixture and observe both script markers.
> - Criterion `crate-script` [test]: Invoke the hook shipped inside a dependency with a relative script path in its manifest.

> **`A-installation-sync`** (draft): **Sync reconciles the copies Symposium owns**
> Property: Installed skills carry a `.symposium` marker. Sync removes stale owned copies and preserves user-managed skill directories and destination symlinks. A failed staging copy leaves the previous skill in place. An unreadable registry suspends cleanup of its previously installed skills.
> Authority: `R-installation`
> - Criterion `owned` [test]: Withdraw a skill and sync. Observe the stale installed copy removed.
> - Criterion `user` [test]: Place user-managed content and a user symlink in a skills directory. Observe both survive sync.
> - Criterion `copy-failure` [test]: Make a source file unreadable during refresh and observe the previous installed skill intact.
> - Criterion `unreadable-registry` [test]: Make a registry unreadable after installing a skill and sync. Observe the installed skill retained.

> **`A-installation-setup`** (draft): **An installation can prepare a command before its hook runs**
> Property: A hook may reference a named installation or carry the setup inline. The installation’s setup commands run before the hook command. Their stdout stays out of the response that the agent parses.
> Authority: `R-installation`
> - Criterion `ordering` [test]: Have setup create the marker the hook reads. Observe the hook find the marker for both named and inline declarations.
> - Criterion `stdout` [test]: Make an install command print during a hook. Observe valid agent output without the installer’s message.
