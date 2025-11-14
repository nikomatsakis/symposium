import * as assert from "assert";
import * as vscode from "vscode";

suite("Webview Lifecycle Tests", () => {
  test("Chat view should persist tabs across hide/show", async function () {
    // This test may need more time for webview operations and agent spawning
    this.timeout(20000);

    // Activate the extension
    const extension = vscode.extensions.getExtension("symposium.symposium");
    assert.ok(extension);
    await extension.activate();

    // Show the chat view (open activity bar item)
    await vscode.commands.executeCommand("symposium.chatView.focus");

    // Give webview time to initialize
    await new Promise((resolve) => setTimeout(resolve, 1000));

    // Simulate creating a tab (this would normally come from the webview)
    console.log("Creating test tab...");
    await vscode.commands.executeCommand(
      "symposium.test.simulateNewTab",
      "test-tab-1",
    );

    // Give time for agent to spawn and session to be created
    await new Promise((resolve) => setTimeout(resolve, 3000));

    // Verify the tab was created
    let tabs = (await vscode.commands.executeCommand(
      "symposium.test.getTabs",
    )) as string[];
    console.log(`Tabs after creation: ${tabs}`);
    assert.ok(tabs.includes("test-tab-1"), "Tab should exist after creation");

    // Close the view by focusing something else
    // We'll use the settings view as a way to "close" the chat view
    console.log("Hiding chat view...");
    await vscode.commands.executeCommand("symposium.settingsView.focus");
    await new Promise((resolve) => setTimeout(resolve, 500));

    // Reopen the chat view
    console.log("Reopening chat view...");
    await vscode.commands.executeCommand("symposium.chatView.focus");

    // Give webview time to restore
    await new Promise((resolve) => setTimeout(resolve, 1000));

    // Verify the tab still exists after reopening
    tabs = (await vscode.commands.executeCommand(
      "symposium.test.getTabs",
    )) as string[];
    console.log(`Tabs after reopen: ${tabs}`);
    assert.ok(
      tabs.includes("test-tab-1"),
      "Tab should persist after view hide/show",
    );
  });
});
