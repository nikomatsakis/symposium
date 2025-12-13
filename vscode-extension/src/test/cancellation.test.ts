import * as assert from "assert";
import * as vscode from "vscode";
import { logger } from "../extension";
import { LogEvent } from "../logger";

suite("Cancellation Tests", () => {
  test("Should cancel previous prompt when sending a new one", async function () {
    // This test needs time for agent spawning and response
    this.timeout(30000);

    // Capture log events
    const logEvents: LogEvent[] = [];
    const logDisposable = logger.onLog((event) => {
      logEvents.push(event);
    });

    // Activate the extension
    const extension = vscode.extensions.getExtension("symposium.symposium");
    assert.ok(extension);
    await extension.activate();

    // Show the chat view
    await vscode.commands.executeCommand("symposium.chatView.focus");
    await new Promise((resolve) => setTimeout(resolve, 1000));

    // Create a tab
    console.log("Creating test tab...");
    await vscode.commands.executeCommand(
      "symposium.test.simulateNewTab",
      "test-tab-cancellation",
    );

    // Wait for agent to spawn and session to be created
    await new Promise((resolve) => setTimeout(resolve, 3000));

    // Verify tab exists
    const tabs = (await vscode.commands.executeCommand(
      "symposium.test.getTabs",
    )) as string[];
    assert.ok(tabs.includes("test-tab-cancellation"), "Tab should exist");

    // Start capturing agent responses
    await vscode.commands.executeCommand(
      "symposium.test.startCapturingResponses",
      "test-tab-cancellation",
    );

    // Send first prompt - don't await the response
    console.log("Sending first prompt...");
    const firstPromptPromise = vscode.commands.executeCommand(
      "symposium.test.sendPrompt",
      "test-tab-cancellation",
      "Tell me a very long story about a dragon",
    );

    // Wait briefly to let the first prompt start processing
    await new Promise((resolve) => setTimeout(resolve, 500));

    // Verify there's an active prompt
    const hasActivePrompt = (await vscode.commands.executeCommand(
      "symposium.test.hasActivePrompt",
      "test-tab-cancellation",
    )) as boolean;
    console.log(`Has active prompt after first send: ${hasActivePrompt}`);

    // Clear log events to focus on cancellation
    const preSecondPromptLogCount = logEvents.length;

    // Send second prompt - this should trigger cancellation of the first
    console.log("Sending second prompt (should trigger cancellation)...");
    await vscode.commands.executeCommand(
      "symposium.test.sendPrompt",
      "test-tab-cancellation",
      "Hello, how are you?",
    );

    // Wait for response
    await new Promise((resolve) => setTimeout(resolve, 2000));

    // Wait for first prompt to complete (it may have been cancelled)
    await firstPromptPromise;

    // Get the response
    const response = (await vscode.commands.executeCommand(
      "symposium.test.getResponse",
      "test-tab-cancellation",
    )) as string;

    console.log(`Response received: ${response.slice(0, 100)}...`);

    // Stop capturing
    await vscode.commands.executeCommand(
      "symposium.test.stopCapturingResponses",
      "test-tab-cancellation",
    );

    // Clean up
    logDisposable.dispose();

    // Check for cancellation log events
    const cancellationEvents = logEvents.filter(
      (e) =>
        e.category === "agent" &&
        e.message === "Cancelling previous prompt before new one",
    );

    const cancelSessionEvents = logEvents.filter(
      (e) => e.category === "agent" && e.message === "Cancelling session",
    );

    console.log(`\nCancellation test summary:`);
    console.log(`- Total log events: ${logEvents.length}`);
    console.log(
      `- Cancellation trigger events: ${cancellationEvents.length}`,
    );
    console.log(`- Cancel session events: ${cancelSessionEvents.length}`);
    console.log(`- Response length: ${response.length} characters`);

    // Verify that cancellation was triggered
    assert.ok(
      cancellationEvents.length >= 1,
      "Should have triggered cancellation of previous prompt",
    );
    assert.ok(
      cancelSessionEvents.length >= 1,
      "Should have sent cancel to agent session",
    );

    // Verify we still got a response (from the second prompt)
    assert.ok(response.length > 0, "Should receive a response from the agent");
  });
});
