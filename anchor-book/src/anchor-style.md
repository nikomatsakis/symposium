# Anchor style guide

Write for a reviewer who wants to understand Symposium without opening the code. An anchor earns its place when it settles a design question that reviewer would care about. Leave room for the implementation to change wherever the rule still holds.

## Put the answer in the main sentence

Read the title with the property and criteria collapsed. “Managers describe the workspace” leaves the reader asking what a manager actually says. “Managers report workspace members and direct dependencies. Managers that find a root must agree on it” tells a reader what they can ask for and what happens when several managers answer. Use the property for the precise scope and the criteria for concrete cases. [Workspace queries](design/plugin-managers.md#A-pm-workspace)

## Introduce the nouns before the operation

Before writing `offers(deps)`, explain who supplies `deps` and what counts as an *offer*. Suppose `my-app` depends on `widget`. Cargo reports `widget`’s resolved package ID even when the crate ships no plugin. If Cargo also offers a plugin, it supplies its own package ID, the manifest, and the plugin directory. Now a reader can follow that ID through discovery and understand how a plugin can be offered while it is still inactive. [Names and identifiers](design/names.md) · [What a manager offers](design/plugin-managers.md#A-pm-offer)

## Split rules that answer independent questions

“Listing and loading answer different questions” leaves both questions open. Give each operation its own anchor. Given the workspace dependencies, **which plugins does this manager offer?** Given one package ID, **which plugins does that package supply?** A reviewer can now challenge either answer on its own. [List offers](design/plugin-managers.md#A-pm-offers) · [Load one package](design/plugin-managers.md#A-pm-load)

## Keep the branches of one choice together

Imagine writing “native handlers can run” and “portable handlers can run” as separate anchors. Both hold even in an implementation that always runs both groups. The design choice disappears. State the scope, the order, and the fallback together: **For each plugin, select all matching native handlers if any match. Otherwise select all matching Symposium-format handlers.** The words *for each* and *all* carry as much design meaning as *otherwise*. [Hook selection](design/hooks.md#A-hook-selection)

## Choose an observation that could prove the rule wrong

To test native priority, give one plugin both a matching native handler and a matching portable handler. Have each return a distinct marker. Which markers reach the agent? Then make the native handler’s predicate false and ask again. The first case distinguishes priority from running everything. The second distinguishes fallback from always selecting the native format. Several criteria can support one anchor when they exercise different branches of the same decision. [Hook selection criteria](design/hooks.md#A-hook-selection)

## Walk the flow to find what the anchors leave open

Start with one event in one workspace. Follow the sequence diagram and ask at each arrow what the recipient can rely on. A diagram can say that Symposium *prepares* a dependency notice. Can it already mark the notice delivered? An anchor answers that question: delivery counts only when the final reply lets the event proceed and the agent can read the notice. Link the step to the anchor that owns the answer. An arrow you cannot explain is a good place to look for a missing rule. [Session start](design/session-start.md#A-flow-session) · [Notice delivery](design/discovery-consent.md#A-notice-delivery)

Be precise about what the evidence establishes. A test of a recorder writing an event tells you what happens when the recorder is called. Claiming that session start calls it takes evidence from session start. Likewise, when proposing a new rule, say what someone will be able to observe and keep its implementation status visible. [Bootstrap and intent](design-documents/bootstrap.md)
