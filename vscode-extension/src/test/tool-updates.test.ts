import * as assert from "assert";
import { SymposiumClient, ToolCallInfo } from "../acpAgentActor";

suite("Tool Update Forwarding", () => {
  test("Should forward tool_call even when status is omitted", async () => {
    const toolCalls: ToolCallInfo[] = [];

    const client = new SymposiumClient({
      onAgentText: () => {},
      onUserText: () => {},
      onAgentComplete: () => {},
      onToolCall: (_sessionId, toolCall) => {
        toolCalls.push(toolCall);
      },
    });

    await client.sessionUpdate({
      sessionId: "test-session",
      update: {
        sessionUpdate: "tool_call",
        toolCallId: "tool-1",
        title: "Read src/main.rs",
        // status intentionally omitted
      },
    } as any);

    assert.strictEqual(toolCalls.length, 1);
    assert.strictEqual(toolCalls[0].toolCallId, "tool-1");
    assert.strictEqual(toolCalls[0].title, "Read src/main.rs");
    assert.strictEqual(toolCalls[0].status, "in_progress");
  });

  test("Should forward tool_call_update even when status is omitted", async () => {
    const toolCallUpdates: ToolCallInfo[] = [];

    const client = new SymposiumClient({
      onAgentText: () => {},
      onUserText: () => {},
      onAgentComplete: () => {},
      onToolCall: () => {},
      onToolCallUpdate: (_sessionId, toolCall) => {
        toolCallUpdates.push(toolCall);
      },
    });

    // Initial tool call establishes title/status.
    await client.sessionUpdate({
      sessionId: "test-session",
      update: {
        sessionUpdate: "tool_call",
        toolCallId: "tool-2",
        title: "Search for replace_goal_chapters",
        status: "in_progress",
      },
    } as any);

    // Update with output only (status omitted).
    await client.sessionUpdate({
      sessionId: "test-session",
      update: {
        sessionUpdate: "tool_call_update",
        toolCallId: "tool-2",
        rawOutput: { matches: 3 },
        // status intentionally omitted
      },
    } as any);

    assert.strictEqual(toolCallUpdates.length, 1);
    assert.strictEqual(toolCallUpdates[0].toolCallId, "tool-2");
    assert.strictEqual(toolCallUpdates[0].title, "Search for replace_goal_chapters");
    assert.strictEqual(toolCallUpdates[0].status, "in_progress");
    assert.deepStrictEqual(toolCallUpdates[0].rawOutput, { matches: 3 });
  });
});

