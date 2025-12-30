import * as assert from "assert";
import * as vscode from "vscode";

suite("Settings Test Suite", () => {
  // Test that settings are properly registered in package.json
  suite("Settings Registration", () => {
    test("symposium.requireModifierToSend should be registered", async () => {
      const config = vscode.workspace.getConfiguration("symposium");
      const inspect = config.inspect<boolean>("requireModifierToSend");

      assert.ok(inspect, "Setting should exist");
      assert.strictEqual(
        inspect.defaultValue,
        false,
        "Default value should be false",
      );
    });
  });

  // Test that settings can be read and written
  suite("Settings Read/Write", () => {
    // Store original values to restore after tests
    let originalRequireModifier: boolean | undefined;

    suiteSetup(async () => {
      const config = vscode.workspace.getConfiguration("symposium");
      originalRequireModifier = config.get<boolean>("requireModifierToSend");
    });

    suiteTeardown(async () => {
      // Restore original values
      const config = vscode.workspace.getConfiguration("symposium");
      if (originalRequireModifier !== undefined) {
        await config.update(
          "requireModifierToSend",
          originalRequireModifier,
          vscode.ConfigurationTarget.Global,
        );
      }
    });

    test("requireModifierToSend can be toggled", async () => {
      // Get current value
      const initialValue =
        vscode.workspace
          .getConfiguration("symposium")
          .get<boolean>("requireModifierToSend") ?? false;

      // Toggle it
      await vscode.workspace
        .getConfiguration("symposium")
        .update(
          "requireModifierToSend",
          !initialValue,
          vscode.ConfigurationTarget.Global,
        );

      // Must re-fetch config to see updated value
      const newValue = vscode.workspace
        .getConfiguration("symposium")
        .get<boolean>("requireModifierToSend");
      assert.strictEqual(newValue, !initialValue, "Value should be toggled");

      // Toggle back
      await vscode.workspace
        .getConfiguration("symposium")
        .update(
          "requireModifierToSend",
          initialValue,
          vscode.ConfigurationTarget.Global,
        );
    });
  });

  // Test that settings flow correctly to webview HTML generation
  suite("Settings Flow to Webview", () => {
    test("Chat view loads with settings without error", async function () {
      this.timeout(10000);

      // Activate the extension
      const extension = vscode.extensions.getExtension(
        "symposium-dev.symposium",
      );
      assert.ok(extension);
      await extension.activate();

      // Set a known value for the setting
      const originalValue = vscode.workspace
        .getConfiguration("symposium")
        .get<boolean>("requireModifierToSend");

      // Update setting to true
      await vscode.workspace
        .getConfiguration("symposium")
        .update(
          "requireModifierToSend",
          true,
          vscode.ConfigurationTarget.Global,
        );

      // Focus the chat view to trigger HTML generation with the setting
      await vscode.commands.executeCommand("symposium.chatView.focus");
      await new Promise((resolve) => setTimeout(resolve, 500));

      // Verify setting persisted (re-fetch config)
      const currentValue = vscode.workspace
        .getConfiguration("symposium")
        .get<boolean>("requireModifierToSend");
      assert.strictEqual(
        currentValue,
        true,
        "Setting should be true after update",
      );

      // Restore original value
      await vscode.workspace
        .getConfiguration("symposium")
        .update(
          "requireModifierToSend",
          originalValue ?? false,
          vscode.ConfigurationTarget.Global,
        );
    });

    test("Settings view loads and responds to configuration changes", async function () {
      this.timeout(10000);

      // Activate the extension
      const extension = vscode.extensions.getExtension(
        "symposium-dev.symposium",
      );
      assert.ok(extension);
      await extension.activate();

      // Focus the settings view
      await vscode.commands.executeCommand("symposium.settingsView.focus");
      await new Promise((resolve) => setTimeout(resolve, 500));

      // Update a setting - the SettingsViewProvider listens for changes
      // and sends updated config to the webview
      const originalValue =
        vscode.workspace
          .getConfiguration("symposium")
          .get<boolean>("requireModifierToSend") ?? false;

      await vscode.workspace
        .getConfiguration("symposium")
        .update(
          "requireModifierToSend",
          !originalValue,
          vscode.ConfigurationTarget.Global,
        );

      // Give time for configuration change event to fire
      await new Promise((resolve) => setTimeout(resolve, 200));

      // Verify setting changed (re-fetch config)
      const newValue = vscode.workspace
        .getConfiguration("symposium")
        .get<boolean>("requireModifierToSend");
      assert.strictEqual(newValue, !originalValue, "Setting should be toggled");

      // Restore
      await vscode.workspace
        .getConfiguration("symposium")
        .update(
          "requireModifierToSend",
          originalValue,
          vscode.ConfigurationTarget.Global,
        );
    });
  });
});
