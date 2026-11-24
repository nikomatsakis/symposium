# Symposium design anchors

When you add a dependency, its package manager can offer a plugin. Symposium decides whether to enable it. A predicate can narrow when its skill applies, and sync can deliver that skill to one agent as a package and to another as a directory. The chapters below pin who makes each of those decisions.

| Chapter | Design question |
| --- | --- |
| [Plugin managers](design/plugin-managers.md) | Who selects the managers? What can Symposium ask each of them? |
| [Plugins](design/plugins.md) | What enables a plugin and what else can it bring into the active set? |
| [Predicates](design/predicates.md) | Which facts decide whether a contribution applies? |
| [Hooks](design/hooks.md) | Which handlers run for an event, and what happens to their answers? |
| [Agent support](design/agents.md) | What does the adapter decide for each host? |
| [Configuration storage](design/configuration.md) | Where do choices and ownership records live across runs? |
| [Installation](design/installation.md) | How does a plugin run its tools, and which installed files can sync replace or remove? |

These anchors are a first reading of `origin/main` at `ab351035`. Each names a design choice and gives an observation you can challenge without reading its implementation. The [review guide](review.md) explains how to run an observation or try a different case.
