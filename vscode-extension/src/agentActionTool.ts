/**
 * Agent Action Tool
 *
 * This tool is invoked when an ACP agent requests permission to execute an
 * internal tool. VS Code's confirmation UI handles user approval, and the
 * result is communicated back through the message history.
 */

import * as vscode from "vscode";

/**
 * Input schema for the symposium-agent-action tool.
 * Matches the ACP RequestPermissionRequest.tool_call fields.
 */
export interface AgentActionInput {
  toolCallId: string;
  title?: string;
  kind?: string;
}

/**
 * Tool implementation for agent action permission requests.
 *
 * When the agent wants to execute an internal tool (like bash or file edit),
 * we emit a LanguageModelToolCallPart for this tool. VS Code shows its
 * confirmation UI, and when approved, invoke() is called.
 */
export class AgentActionTool
  implements vscode.LanguageModelTool<AgentActionInput>
{
  /**
   * Prepare the invocation - customize the confirmation UI.
   */
  async prepareInvocation(
    options: vscode.LanguageModelToolInvocationPrepareOptions<AgentActionInput>,
    _token: vscode.CancellationToken,
  ): Promise<vscode.PreparedToolInvocation> {
    const { title, kind } = options.input;

    // Build a descriptive message for the confirmation dialog
    const actionDescription = title || kind || "execute an action";

    return {
      invocationMessage: `Executing: ${actionDescription}`,
      confirmationMessages: {
        title: "Agent Action",
        message: new vscode.MarkdownString(
          `Allow the agent to **${actionDescription}**?`,
        ),
      },
    };
  }

  /**
   * Invoke the tool - called after user approves.
   *
   * We return an empty result since the actual tool execution happens
   * on the agent side. This just signals approval.
   */
  async invoke(
    _options: vscode.LanguageModelToolInvocationOptions<AgentActionInput>,
    _token: vscode.CancellationToken,
  ): Promise<vscode.LanguageModelToolResult> {
    // Return empty content - the approval is signaled by the tool result
    // appearing in the message history
    return new vscode.LanguageModelToolResult([]);
  }
}
