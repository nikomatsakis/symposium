// This file runs in the webview context (browser environment)
import { MynahUI, ChatItem, ChatItemType } from "@aws/mynah-ui";

// Browser API declarations for webview context
declare const acquireVsCodeApi: any;
declare const window: any & {
  SYMPOSIUM_SESSION_ID: string;
};

// Import uuid - note: webpack will bundle this for browser
import { v4 as uuidv4 } from "uuid";

const vscode = acquireVsCodeApi();

let mynahUI: MynahUI;

// Track which messages we've seen per tab and mynah UI state
interface WebviewState {
  sessionId: string;
  lastSeenIndex: { [tabId: string]: number };
  mynahTabs?: any; // Mynah UI tabs state
}

// Get session ID from window (embedded by extension)
const currentSessionId = window.SYMPOSIUM_SESSION_ID;
console.log(`Session ID: ${currentSessionId}`);

// Load saved state and check if we need to clear it
const savedState = vscode.getState() as WebviewState | undefined;
let lastSeenIndex: { [tabId: string]: number } = {};
let mynahTabs: any = undefined;

if (
  !savedState ||
  !savedState.sessionId ||
  savedState.sessionId !== currentSessionId
) {
  if (savedState) {
    console.log(
      `Session ID mismatch or missing (saved: ${savedState.sessionId}, current: ${currentSessionId}), clearing state`,
    );
  }
  // Clear persisted state
  vscode.setState(undefined);
  // Start fresh
  lastSeenIndex = {};
  mynahTabs = undefined;
} else {
  // Keep existing state - session ID matches
  lastSeenIndex = savedState.lastSeenIndex ?? {};
  mynahTabs = savedState.mynahTabs;
  if (mynahTabs) {
    console.log("Restoring mynah tabs from saved state");
  }
}

const config: any = {
  rootSelector: "#mynah-root",
  loadStyles: true,
  config: {
    texts: {
      mainTitle: "Symposium",
      noTabsOpen: "### Join the symposium by opening a tab",
    },
  },
  defaults: {
    store: {
      tabTitle: "Symposium",
    },
  },
  onTabAdd: (tabId: string) => {
    // Notify extension that a new tab was created
    console.log("New tab created:", tabId);
    vscode.postMessage({
      type: "new-tab",
      tabId: tabId,
    });
    // Save state when tab is added
    saveState();
  },
  onTabRemove: (tabId: string) => {
    // Save state when tab is closed
    console.log("Tab removed:", tabId);
    saveState();
  },
  onChatPrompt: (tabId: string, prompt: any) => {
    // Generate UUID for this message
    const messageId = uuidv4();

    // Send prompt to extension with tabId and messageId
    vscode.postMessage({
      type: "prompt",
      tabId: tabId,
      messageId: messageId,
      prompt: prompt.prompt,
    });

    // Add the user's prompt to the chat
    mynahUI.addChatItem(tabId, {
      type: ChatItemType.PROMPT,
      body: prompt.prompt,
    });

    // Add placeholder for the streaming answer
    mynahUI.addChatItem(tabId, {
      type: ChatItemType.ANSWER_STREAM,
      messageId: messageId,
      body: "",
    });

    // Save state when prompt is sent
    saveState();
  },
};

// If we have saved tabs, initialize with them
if (mynahTabs) {
  config.tabs = mynahTabs;
  console.log("Initializing MynahUI with restored tabs");
}

mynahUI = new MynahUI(config);
console.log("MynahUI initialized");

// Tell extension we're ready to receive messages
vscode.postMessage({ type: "webview-ready" });

// Save state helper
function saveState() {
  // Get current tabs from mynah UI
  const currentTabs = mynahUI?.getAllTabs();

  const state: WebviewState = {
    sessionId: currentSessionId,
    lastSeenIndex,
    mynahTabs: currentTabs,
  };
  vscode.setState(state);
  console.log("Saved state with session ID:", currentSessionId);
}

// Handle messages from the extension
window.addEventListener("message", (event: MessageEvent) => {
  const message = event.data;

  // Check if we've already seen this message
  const currentLastSeen = lastSeenIndex[message.tabId] ?? -1;
  if (message.index <= currentLastSeen) {
    console.log(
      `Ignoring duplicate message ${message.index} for tab ${message.tabId}`,
    );
    return;
  }

  // Process the message
  if (message.type === "response-chunk") {
    // Update the streaming answer with the new chunk
    mynahUI.updateChatAnswerWithMessageId(message.tabId, message.messageId, {
      body: message.chunk,
    });
  } else if (message.type === "response-complete") {
    // Mark the stream as complete
    mynahUI.endMessageStream(message.tabId, message.messageId);
  }

  // Update lastSeenIndex and save state
  lastSeenIndex[message.tabId] = message.index;
  saveState();

  // Send acknowledgment
  vscode.postMessage({
    type: "message-ack",
    tabId: message.tabId,
    index: message.index,
  });
});
