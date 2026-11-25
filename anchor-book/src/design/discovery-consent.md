# Dependency discovery and consent

Suppose `my-app` depends on `widget`. Cargo reports that dependency, and Symposium asks the ecosystem managers for plugin offers. If Cargo finds a plugin embedded in `widget`, its offer identifies the package and supplies the manifest and content directory. These are the [discovery queries](plugin-managers.md#A-pm-offers).

> **`A-flow-consent`** (draft): **Session start can tell the agent about a pending plugin while the plugin stays inactive**
> Property: During discovery Symposium compares offers against revocations and user choices. A pending plugin may be named in the agent’s context. Its offer alone grants no permission to run it.
> Authority: `R-plugins`
> - Criterion `pending` [test]: Open a session with an undecided dependency plugin. Observe a notice naming the package, pointing to `symposium use`, and directing the agent to leave the decision to the user.

<figure class="chapter-diagram" data-anchor="A-flow-consent">

```mermaid
sequenceDiagram
    participant S as Symposium
    participant M as Ecosystem managers
    participant A as Agent
    link S: Enablement @ plugins.html#35;A-plugin-enablement
    link S: Revocations @ plugins.html#35;A-plugin-revocation
    link M: Dependency IDs @ names.html#35;A-package-owner
    link M: Offers for dependencies @ plugin-managers.html#35;A-pm-offers
    link A: Session response @ hook-dispatch.html#35;A-flow-dispatch
    S->>M: [List direct workspace dependencies](workspace-discovery.html#A-flow-workspace)
    M-->>S: [Package IDs](names.html#A-package-owner)
    S->>M: [List plugins offered for those IDs](plugin-managers.html#A-pm-offers)
    M-->>S: [Plugin offers](plugin-managers.html#A-pm-offer)
    loop Each offer
        S->>S: [Apply the first matching consent rule](discovery-consent.html#A-offer-precedence)
        alt Pending consent
            S->>S: [Add package to the consent notice](plugins.html#A-plugin-enablement)
        else Enabled
            S->>S: [Eligible for workspace activation](plugin-activation.html#A-flow-activation)
        else Declined or revoked
            S->>S: [Keep plugin inactive](discovery-consent.html#A-offer-precedence)
        end
    end
    opt Pending packages
        S->>S: [Dispatch handlers and compose the final reply](hook-dispatch.html#A-flow-dispatch)
        opt Final reply allows the event and carries a readable notice
            S->>S: [Record the named releases as delivered](discovery-consent.html#A-notice-delivery)
        end
        S-->>A: [Return the host-specific response](hook-dispatch.html#A-dispatch-stdout)
    end
```

<figcaption><a href="#A-consent-record">How Symposium remembers the user’s decision</a></figcaption>
</figure>

When a session notice names a pending plugin, the user can review it in an interactive `symposium sync` or select it with `symposium use`. A [revocation](plugins.md#A-plugin-revocation) can still withhold the package after approval.

> **`A-consent-record`** (draft): **Approving a dependency saves its name in auto-enable. Declining saves it in disable**
> Property: Approving a dependency offer records its package name in `auto-enable`. Declining records it in `disable`. Future discovery uses that decision, and sync can install the approved package’s applicable contributions.
> Authority: `R-plugins`
> - Criterion `remember` [test]: Approve `crate-a`, sync, and observe its installed skill and an empty pending list. Decline another dependency and inspect the saved disable entry.

For later workspace changes, [the package ID determines which releases can be named again](#A-notice-release).

An enabled plugin’s [predicates](predicates.md#A-predicate-expressions) determine which contributions apply. A session handler that passes its gates enters [hook dispatch](hook-dispatch.md).

> **`A-offer-precedence`** (draft): **For a resolved offer, check disable first, then revocation, then use, then auto-enable. No match means pending**
> Property: Symposium declines an offer whose package identity it cannot resolve. For a resolved offer, the first match wins: `disable` (declined), revocation (revoked), `use` (explicitly enabled), then `auto-enable` (automatically enabled). With no match, the offer stays pending even if it has a positive audit.
> Authority: `R-plugins`
> - Criterion `identity` [test]: Have the manager return an unusable name comparison. Observe that explicit use and auto-enable cannot grant the package access.
> - Criterion `revoked` [test]: Revoke an offered release. Explicit use and auto-enable both yield to revocation. Adding a disable entry yields a decline.
> - Criterion `grants` [test]: Combine `auto-enable` with explicit use or with disable and observe used or declined, respectively.
> - Criterion `pending` [test]: Offer an audited release with no user grant and observe it remains a candidate.

> **`A-notice-delivery`** (draft): **A notice counts as delivered only if the final reply lets the event proceed and the agent can read the notice**
> Property: Symposium records delivery when the final reply carries the notice in a form the calling agent can read and allows the event to proceed. Otherwise a later workspace change can surface the offer again.
> Authority: `R-plugins`
> - Criterion `final-reply` [test]: Block a prompt or use an event whose agent cannot read the notice. A later change surfaces the package again. Lift the block before the reply and the delivered package is suppressed.

> **`A-notice-release`** (draft): **After delivery, mid-session notices skip the same package release and can announce a new version**
> Property: After delivery, later workspace changes suppress the same package ID. A newly observed version of that package can be named again.
> Authority: `R-plugins`
> - Criterion `release` [test]: Change an unrelated dependency, add another pending package, and change a previously reported package’s version. Observe which releases are named after each change.
