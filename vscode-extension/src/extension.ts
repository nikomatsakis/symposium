import * as vscode from "vscode";
import { ToadPanelProvider } from "./toadPanelProvider";
import { Logger } from "./logger";

export const logger = new Logger("Symposium");

export function activate(context: vscode.ExtensionContext) {
  logger.important("extension", "Symposium extension is now active");

  const toadProvider = new ToadPanelProvider(context);

  context.subscriptions.push(
    vscode.window.registerWebviewViewProvider(
      ToadPanelProvider.viewType,
      toadProvider,
      { webviewOptions: { retainContextWhenHidden: true } },
    ),
  );

  context.subscriptions.push(
    vscode.commands.registerCommand("symposium.openChat", () => {
      vscode.commands.executeCommand("symposium.toadView.focus");
    }),
  );
}

export function deactivate() {
  // Toad process is intentionally left running so it survives reloads.
}
