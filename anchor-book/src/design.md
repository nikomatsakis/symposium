# Symposium design anchors

Symposium gives coding agents skills, hook handlers, and tools that apply to the project they are working in. A Rust dependency can ship a plugin that teaches an agent how to use the crate. Cargo can offer that plugin to Symposium. The user’s choices determine whether Symposium may enable it, and the plugin’s conditions determine where it applies.

Start with [packages, plugins, and their names](design/names.md), then follow [what happens when Claude opens a session in a Rust workspace](design/session-start.md#A-flow-session).

| Chapter | Design question |
| --- | --- |
| [Packages, plugins, and their names](design/names.md) | What does a package ID identify? How does it relate to a plugin name or a skill name? |
| [Execution flow](design/flows.md) | How does an event reach a workspace, surface plugin offers, and produce one reply? |
| [Plugin managers](design/plugin-managers.md) | Who selects the managers? What can Symposium ask each of them? |
| [Plugins](design/plugins.md) | What enables a plugin and what else can it bring into the active set? |
| [Predicates](design/predicates.md) | Which facts decide whether a contribution applies? |
| [Hooks](design/hooks.md) | Which handlers run for an event, and what happens to their answers? |
| [Agent support](design/agents.md) | What does the adapter decide for each host? |
| [Configuration storage](design/configuration.md) | Where do choices and ownership records live across runs? |
| [Installation](design/installation.md) | How does a plugin run its tools, and which installed files can sync replace or remove? |

The [bootstrap document](design-documents/bootstrap.md) explains how the first anchors were reconstructed from Symposium at `ab351035`. The [review guide](review.md) explains how to run an observation or try a different case.
