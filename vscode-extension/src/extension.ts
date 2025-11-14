import * as vscode from "vscode";
import { ChatViewProvider } from "./chatViewProvider";
import { v4 as uuidv4 } from "uuid";

export function activate(context: vscode.ExtensionContext) {
  console.log("Symposium extension is now active");

  // Generate extension activation ID for this VSCode session
  const extensionActivationId = uuidv4();
  console.log(`Generated extension activation ID: ${extensionActivationId}`);

  // Register the webview view provider
  const provider = new ChatViewProvider(
    context.extensionUri,
    context,
    extensionActivationId,
  );
  context.subscriptions.push(
    vscode.window.registerWebviewViewProvider(
      ChatViewProvider.viewType,
      provider,
    ),
  );

  // Register the command to open chat
  context.subscriptions.push(
    vscode.commands.registerCommand("symposium.openChat", () => {
      vscode.commands.executeCommand("symposium.chatView.focus");
    }),
  );

  // Debug command to inspect saved state
  context.subscriptions.push(
    vscode.commands.registerCommand("symposium.inspectState", async () => {
      const state = context.workspaceState.get("symposium.chatState");
      const stateJson = JSON.stringify(state, null, 2);
      const doc = await vscode.workspace.openTextDocument({
        content: stateJson,
        language: "json",
      });
      await vscode.window.showTextDocument(doc);
    }),
  );
}

export function deactivate() {}
