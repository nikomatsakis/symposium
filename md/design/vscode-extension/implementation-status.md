# Implementation Status

This chapter tracks what's been implemented, what's in progress, and what's planned for the VSCode extension.

## Core Architecture

- [x] Three-layer architecture (webview/extension/agent)
- [x] Message routing with UUID-based identification
- [x] HomerActor mock agent with session support
- [x] Webview state persistence with session ID checking
- [x] Message buffering when webview is hidden
- [x] Message deduplication via last-seen-index tracking

## Error Handling

- [x] Agent crash detection (partially implemented - detection works, UI error display incomplete)
- [ ] Complete error recovery UX (restart agent button, error notifications)
- [ ] Agent health monitoring and automatic restart

## Agent Lifecycle

- [x] Agent spawn on extension activation (partially implemented - spawn/restart works, graceful shutdown incomplete)
- [ ] Graceful agent shutdown on extension deactivation
- [ ] Agent process supervision and restart on crash

## ACP Integration

- [ ] Replace HomerActor with real ACP agent
- [ ] ACP message protocol implementation
- [ ] ACP capability negotiation
- [ ] Tool calls from agent to extension

## State Management

- [x] Webview state persistence within session
- [x] Chat history persistence across hide/show cycles
- [ ] Draft text persistence (FIXME: partially typed prompts are lost on hide/show)
- [ ] Session restoration after VSCode restart
- [ ] Workspace-specific state persistence
- [ ] Tab history and conversation export
