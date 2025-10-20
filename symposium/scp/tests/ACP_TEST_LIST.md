# ACP Implementation Test Suite

This document tracks test coverage for our ACP (Agent Client Protocol) implementation against the `agent-client-protocol` crate.

## Agent-Side Tests (Messages agents receive from editors)

### Requests (agent receives, must respond)

- [ ] **initialize** - Agent initialization handshake
- [ ] **authenticate** - Agent authentication
- [ ] **session/new** - Create a new session
- [ ] **session/load** - Load existing session
- [ ] **session/prompt** - Send prompt to agent
- [ ] **session/set_mode** - Change session mode

### Notifications (agent receives, no response)

- [ ] **session/cancel** - Cancel current operation

## Editor-Side Tests (Messages editors receive from agents)

### Requests (editor receives, must respond)

- [ ] **session/request_permission** - Agent requests user permission for tool call
- [ ] **fs/read_text_file** - Agent requests to read file
- [ ] **fs/write_text_file** - Agent requests to write file
- [ ] **terminal/create** - Agent requests to create terminal
- [ ] **terminal/output** - Agent requests terminal output/input
- [ ] **terminal/release** - Agent releases terminal control
- [ ] **terminal/wait_for_exit** - Agent waits for terminal process to exit
- [ ] **terminal/kill** - Agent kills terminal process

### Notifications (editor receives, no response)

- [ ] **session/update** - Agent sends session state update

## Integration Tests

### Basic Communication

- [ ] **Agent-to-Editor round-trip** - Agent sends request, editor responds
- [ ] **Editor-to-Agent round-trip** - Editor sends request, agent responds
- [ ] **Bidirectional session** - Both sides send requests during same session
- [ ] **Multiple concurrent requests** - Handle multiple in-flight requests

### Error Handling

- [ ] **Unknown ACP method** - Graceful handling of unknown methods
- [ ] **Invalid ACP parameters** - Handle malformed request parameters
- [ ] **ACP error responses** - Properly format and handle ACP error codes
- [ ] **Session not found** - Handle requests for non-existent sessions

### Serialization Compatibility

- [ ] **Request serialization** - Our requests match ACP crate format
- [ ] **Response serialization** - Our responses match ACP crate format
- [ ] **Notification serialization** - Our notifications match ACP crate format
- [ ] **Complex types** - ToolCallUpdate, PermissionOption, etc. serialize correctly

### Type Compatibility

- [ ] **SessionId** - Our usage matches ACP crate
- [ ] **ToolCallUpdate** - Structure matches ACP crate
- [ ] **PermissionOption** - Structure matches ACP crate
- [ ] **Terminal types** - Terminal IDs and operations match

## Notes

- Tests should validate against actual `agent-client-protocol` crate types
- Focus on ensuring our JsonRpcHandler implementations correctly route ACP messages
- Verify that our extension trait methods produce valid ACP requests
- Test both happy path and error scenarios for each message type
