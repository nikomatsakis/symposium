# Activating a plugin

A plugin is **active** when Symposium has selected it for the current workspace and its plugin-level conditions hold. Its skills, hooks, MCP servers, and subcommands can then be considered. Each contribution still has its own conditions. For example, `widget` can be active while none of its hooks match `SessionStart`.

> **`A-flow-activation`** (draft): **Sync and hook dispatch check each plugin’s activation conditions before using its contributions or following its references**
> Property: Symposium starts with plugins from configured registries and the workspace root and members. It also loads enabled packages through their owning managers and follows applicable package references from active plugins. A reached package can be withheld by revocation. Each plugin must pass its own predicates and any explicit-use requirement before its contributions or references are considered.
> Authority: `R-plugins`
> - Criterion `contributions` [test]: Approve the fixture dependency and sync. Observe its installed skill and MCP server, dispatched hook, and available subcommand.
> - Criterion `explicit-use` [test]: Select a dormant registry plugin with `use`, then remove that selection. Observe its skill appear and disappear.

<figure class="chapter-diagram" data-anchor="A-flow-activation">

```mermaid
sequenceDiagram
    participant S as Symposium
    participant C as User configuration
    participant M as Cargo manager
    participant H as Current operation
    Note over S,M: The workspace depends on widget
    S->>S: [Consider registry and workspace plugins](plugins.html#A-plugin-sources)
    S->>C: [Read use entries that apply to this workspace](plugins.html#A-plugin-use-scope)
    S->>S: [Resolve each selected name to one package](plugins.html#A-plugin-use-name)
    opt widget is enabled or reached by an applicable reference
        S->>M: [Load widget by package ID](plugin-managers.html#A-pm-load)
        M-->>S: [Return plugins supplied by widget](plugin-managers.html#A-pm-load)
        S->>S: [Check for a matching revocation](plugins.html#A-plugin-revocation)
        opt Package passes the audit check
            S->>S: [Check whether the plugin requires explicit use](plugins.html#A-plugin-dormancy)
            S->>S: [Evaluate the plugin predicates](predicates.html#A-predicate-expressions)
            opt Plugin applies
                S->>S: [Include widget and follow applicable package references](plugins.html#A-plugin-chains)
            end
        end
    end
    alt During sync
        S-->>H: [Reconcile applicable installed contributions](installation.html#A-installation-sync)
        opt A manager could not list dependencies or respond to a load request
            H->>H: [Preserve installed contributions during cleanup](installation.html#A-sync-manager-failure)
        end
    else During a hook
        S-->>H: [Select matching handlers from the active plugins](hook-dispatch.html#A-flow-dispatch)
    end
```

<figcaption><a href="hook-dispatch.html#A-flow-dispatch">Follow an active plugin into hook dispatch</a></figcaption>
</figure>

In the [Rust workspace example](workspace-discovery.md#A-flow-workspace), suppose `widget` is reachable only as a direct dependency. While consent is pending, its offer can appear in a session notice. Once the user approves it, sync can load it and install the `usage` skill if its conditions hold. Hook dispatch resolves the active set using locally available package content and checks each handler against the incoming event. The [consent record](discovery-consent.md#A-consent-record) persists across invocations. Symposium evaluates applicability in the workspace of each invocation.
