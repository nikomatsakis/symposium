import * as vscode from "vscode";
import { HomerActor } from "./homerActor";

interface IndexedMessage {
  index: number;
  type: string;
  tabId: string;
  messageId: string;
  chunk?: string;
}

export class ChatViewProvider implements vscode.WebviewViewProvider {
  public static readonly viewType = "symposium.chatView";
  #view?: vscode.WebviewView;
  #agent: HomerActor;
  #tabToSession: Map<string, string> = new Map(); // tabId → sessionId
  #messageQueues: Map<string, IndexedMessage[]> = new Map(); // tabId → queue of unacked messages
  #nextMessageIndex: Map<string, number> = new Map(); // tabId → next index to assign
  #extensionUri: vscode.Uri;
  #sessionId: string;

  constructor(
    extensionUri: vscode.Uri,
    context: vscode.ExtensionContext,
    sessionId: string,
  ) {
    this.#extensionUri = extensionUri;
    this.#sessionId = sessionId;
    // Create singleton agent
    this.#agent = new HomerActor();
  }

  public resolveWebviewView(
    webviewView: vscode.WebviewView,
    context: vscode.WebviewViewResolveContext,
    _token: vscode.CancellationToken,
  ) {
    this.#view = webviewView;

    webviewView.webview.options = {
      enableScripts: true,
      localResourceRoots: [this.#extensionUri],
    };

    webviewView.webview.html = this.#getHtmlForWebview(webviewView.webview);

    // Handle webview visibility changes
    webviewView.onDidChangeVisibility(() => {
      if (webviewView.visible) {
        console.log("Webview became visible");
        this.#onWebviewVisible();
      } else {
        console.log("Webview became hidden");
        this.#onWebviewHidden();
      }
    });

    // Handle messages from the webview
    webviewView.webview.onDidReceiveMessage(async (message) => {
      switch (message.type) {
        case "new-tab":
          // Create a new session for this tab
          const sessionId = this.#agent.createSession();
          this.#tabToSession.set(message.tabId, sessionId);

          // Initialize message tracking for this tab
          this.#messageQueues.set(message.tabId, []);
          this.#nextMessageIndex.set(message.tabId, 0);

          console.log(`Created session ${sessionId} for tab ${message.tabId}`);
          break;

        case "message-ack":
          // Webview acknowledged a message - remove from queue
          this.#handleMessageAck(message.tabId, message.index);
          break;

        case "prompt":
          console.log(
            `Received prompt for tab ${message.tabId}, message ${message.messageId}`,
          );

          // Get the session for this tab
          const promptSessionId = this.#tabToSession.get(message.tabId);
          if (!promptSessionId) {
            console.error(`No session found for tab ${message.tabId}`);
            return;
          }

          console.log(`Processing prompt with session ${promptSessionId}`);

          // Stream the response progressively
          for await (const chunk of this.#agent.processPrompt(
            promptSessionId,
            message.prompt,
          )) {
            console.log(`Sending chunk for message ${message.messageId}`);
            this.#sendToWebview({
              type: "response-chunk",
              tabId: message.tabId,
              messageId: message.messageId,
              chunk: chunk,
            });
          }

          // Send final message to indicate streaming is complete
          console.log(
            `Sending response-complete for message ${message.messageId}`,
          );
          this.#sendToWebview({
            type: "response-complete",
            tabId: message.tabId,
            messageId: message.messageId,
          });
          break;

        case "webview-ready":
          // Webview is initialized and ready to receive messages
          console.log("Webview ready - replaying queued messages");
          this.#replayQueuedMessages();
          break;
      }
    });
  }

  #handleMessageAck(tabId: string, ackedIndex: number) {
    const queue = this.#messageQueues.get(tabId);
    if (!queue) {
      return;
    }

    // Remove all messages with index <= ackedIndex
    const remaining = queue.filter((msg) => msg.index > ackedIndex);
    this.#messageQueues.set(tabId, remaining);

    console.log(
      `Acked message ${ackedIndex} for tab ${tabId}, ${remaining.length} messages remain in queue`,
    );
  }

  #replayQueuedMessages() {
    if (!this.#view) {
      return;
    }

    // Replay all queued messages for all tabs
    for (const [tabId, queue] of this.#messageQueues.entries()) {
      for (const message of queue) {
        console.log(`Replaying message ${message.index} for tab ${tabId}`);
        this.#view.webview.postMessage(message);
      }
    }
  }

  #sendToWebview(message: any) {
    if (!this.#view) {
      return;
    }

    const tabId = message.tabId;
    if (!tabId) {
      console.error("Message missing tabId:", message);
      return;
    }

    // Assign index to message
    const index = this.#nextMessageIndex.get(tabId) ?? 0;
    this.#nextMessageIndex.set(tabId, index + 1);

    const indexedMessage: IndexedMessage = {
      index,
      ...message,
    };

    // Add to queue (unacked messages)
    const queue = this.#messageQueues.get(tabId) ?? [];
    queue.push(indexedMessage);
    this.#messageQueues.set(tabId, queue);

    // Send if webview is visible
    if (this.#view.visible) {
      console.log(`Sending message ${index} for tab ${tabId}`);
      this.#view.webview.postMessage(indexedMessage);
    } else {
      console.log(`Queued message ${index} for tab ${tabId} (webview hidden)`);
    }
  }

  #onWebviewVisible() {
    // Visibility change detected - webview will send "webview-ready" when initialized
    console.log("Webview became visible");
  }

  #onWebviewHidden() {
    // Nothing to do - messages stay queued until acked
    console.log("Webview became hidden");
  }

  #getHtmlForWebview(webview: vscode.Webview) {
    const scriptUri = webview.asWebviewUri(
      vscode.Uri.joinPath(this.#extensionUri, "out", "webview.js"),
    );

    return `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Symposium Chat</title>
    <style>
        body {
            margin: 0;
            padding: 0;
            overflow: hidden;
        }
        #mynah-root {
            width: 100%;
            height: 100vh;
        }
    </style>
</head>
<body>
    <div id="mynah-root"></div>
    <script>
        // Embed session ID so it's available immediately
        window.SYMPOSIUM_SESSION_ID = "${this.#sessionId}";
    </script>
    <script src="${scriptUri}"></script>
</body>
</html>`;
  }
}
