# Execution flow

From an agent’s working directory, follow how Symposium chooses managers, finds the workspace, discovers plugin offers, and dispatches hooks. Each flow anchor contains a sequence diagram. Follow an underlined message to its detailed flow or design anchor. Select a participant to open the anchors governing its part in the flow.

- <span id="A-flow-session"></span>[Session start](session-start.md#A-flow-session). Follow the workspace lookup, optional sync, session notices, and reply to the agent.
- [Choosing the plugin managers](manager-selection.md#A-flow-managers). See when Symposium uses built-in Cargo, starts a configured command, or includes a registry.
- [Discovering a Rust workspace](workspace-discovery.md#A-flow-workspace). Follow `app` and its `widget` dependency from a working directory to an embedded skill offer.
- <span id="A-flow-consent"></span>[Dependency discovery and consent](discovery-consent.md#A-flow-consent). Follow a dependency from a manager’s plugin offer to the user’s consent decision.
- [Activating a plugin](plugin-activation.md#A-flow-activation). Follow an enabled package from its manager to the contributions eligible in this workspace.
- <span id="A-flow-dispatch"></span>[Hook dispatch](hook-dispatch.md#A-flow-dispatch). Follow one event through handler selection and execution to the agent’s reply.
