/**
 * AcpAgentActor - Real ACP agent integration
 *
 * Spawns an ACP agent process (e.g., elizacp, Claude Code) and manages
 * communication via the Agent Client Protocol over stdio.
 */

import { spawn, ChildProcess } from "child_process";
import { Writable, Readable } from "stream";
import * as acp from "@agentclientprotocol/sdk";
import * as vscode from "vscode";
import { AgentConfiguration } from "./agentConfiguration";
import { logger } from "./extension";

/**
 * Tool call information passed to callbacks
 */
export interface ToolCallInfo {
  toolCallId: string;
  title: string;
  status: acp.ToolCallStatus;
  kind?: acp.ToolKind;
  rawInput?: Record<string, unknown>;
  rawOutput?: Record<string, unknown>;
}

/**
 * Slash command information passed to callbacks
 */
export interface SlashCommandInfo {
  name: string;
  description: string;
  inputHint?: string;
}

/**
 * Callback interface for agent events
 */
export interface AcpAgentCallbacks {
  onAgentText: (agentSessionId: string, text: string) => void;
  onUserText: (agentSessionId: string, text: string) => void;
  onAgentComplete: (agentSessionId: string) => void;
  onRequestPermission?: (
    params: acp.RequestPermissionRequest,
  ) => Promise<acp.RequestPermissionResponse>;
  onToolCall?: (agentSessionId: string, toolCall: ToolCallInfo) => void;
  onToolCallUpdate?: (agentSessionId: string, toolCall: ToolCallInfo) => void;
  onAvailableCommands?: (
    agentSessionId: string,
    commands: SlashCommandInfo[],
  ) => void;
}

/**
 * Implementation of the ACP Client interface
 */
export class SymposiumClient implements acp.Client {
  // Cache tool call state so updates that omit fields still render correctly.
  private toolCalls: Map<string, ToolCallInfo> = new Map();

  constructor(private callbacks: AcpAgentCallbacks) {}

  async requestPermission(
    params: acp.RequestPermissionRequest,
  ): Promise<acp.RequestPermissionResponse> {
    if (this.callbacks.onRequestPermission) {
      return this.callbacks.onRequestPermission(params);
    }

    // Default: auto-approve read operations, reject everything else
    logger.debug("approval", "Permission requested (default handler)", {
      title: params.toolCall.title,
      kind: params.toolCall.kind,
    });

    if (params.toolCall.kind === "read") {
      const allowOption = params.options.find(
        (opt) => opt.kind === "allow_once",
      );
      if (allowOption) {
        return {
          outcome: { outcome: "selected", optionId: allowOption.optionId },
        };
      }
    }

    const rejectOption = params.options.find(
      (opt) => opt.kind === "reject_once",
    );
    if (rejectOption) {
      return {
        outcome: { outcome: "selected", optionId: rejectOption.optionId },
      };
    }

    // Fallback: cancel
    return { outcome: { outcome: "cancelled" } };
  }

  async sessionUpdate(params: acp.SessionNotification): Promise<void> {
    const update = params.update;

    switch (update.sessionUpdate) {
      case "agent_message_chunk":
        if (update.content.type === "text") {
          const text = update.content.text;
          logger.debug("agent", "Text chunk", {
            length: text.length,
            text: text.length > 50 ? text.slice(0, 50) + "..." : text,
          });
          this.callbacks.onAgentText(params.sessionId, update.content.text);
        }
        break;
      case "tool_call":
        {
          const status = update.status ?? "in_progress";
          const toolCall: ToolCallInfo = {
            toolCallId: update.toolCallId,
            title: update.title,
            status,
            kind: update.kind,
            rawInput: update.rawInput,
            rawOutput: update.rawOutput,
          };

          logger.debug("agent", "Tool call", {
            toolCallId: toolCall.toolCallId,
            title: toolCall.title,
            status: toolCall.status,
          });

          this.toolCalls.set(update.toolCallId, toolCall);
          this.callbacks.onToolCall?.(params.sessionId, toolCall);
        }
        break;
      case "tool_call_update": {
        const previous = this.toolCalls.get(update.toolCallId);

        const title = update.title ?? previous?.title ?? "";
        const status = update.status ?? previous?.status ?? "in_progress";

        const toolCall: ToolCallInfo = {
          toolCallId: update.toolCallId,
          title,
          status,
          kind: update.kind ?? previous?.kind,
          rawInput: update.rawInput ?? previous?.rawInput,
          rawOutput: update.rawOutput ?? previous?.rawOutput,
        };

        logger.debug("agent", "Tool call update", {
          toolCallId: toolCall.toolCallId,
          title: toolCall.title,
          status: toolCall.status,
        });

        this.toolCalls.set(update.toolCallId, toolCall);
        this.callbacks.onToolCallUpdate?.(params.sessionId, toolCall);

        // Clean up cache when tool call completes
        if (toolCall.status === "completed" || toolCall.status === "failed") {
          this.toolCalls.delete(update.toolCallId);
        }
        break;
      }
      case "available_commands_update": {
        const commands: SlashCommandInfo[] = update.availableCommands.map(
          (cmd) => ({
            name: cmd.name,
            description: cmd.description,
            inputHint: cmd.input?.hint,
          }),
        );
        logger.debug("agent", "Available commands update", {
          count: commands.length,
          commands: commands.map((c) => c.name),
        });
        if (this.callbacks.onAvailableCommands) {
          this.callbacks.onAvailableCommands(params.sessionId, commands);
        }
        break;
      }
      case "user_message_chunk": {
        if (update.content.type === "text") {
          const text = update.content.text;
          logger.debug("user", "Text chunk", {
            length: text.length,
            text: text.length > 50 ? text.slice(0, 50) + "..." : text,
          });
          this.callbacks.onUserText(params.sessionId, update.content.text);
        }
        break;
      }
    }
  }

  async readTextFile(
    params: acp.ReadTextFileRequest,
  ): Promise<acp.ReadTextFileResponse> {
    // TODO: Implement file reading through VSCode APIs
    logger.warn("fs", "Read file requested but not implemented", {
      path: params.path,
    });
    throw new Error("File reading not yet implemented");
  }

  async writeTextFile(
    params: acp.WriteTextFileRequest,
  ): Promise<acp.WriteTextFileResponse> {
    // TODO: Implement file writing through VSCode APIs
    logger.warn("fs", "Write file requested but not implemented", {
      path: params.path,
    });
    throw new Error("File writing not yet implemented");
  }
}

export class AcpAgentActor {
  private connection?: acp.ClientSideConnection;
  private agentProcess?: ChildProcess;
  private callbacks: AcpAgentCallbacks;

  constructor(callbacks: AcpAgentCallbacks) {
    this.callbacks = callbacks;
  }

  /**
   * Initialize the ACP connection by spawning the agent process
   * @param config - Agent configuration (just workspace folder now)
   * @param conductorCommand - Path to the conductor/agent binary
   */
  async initialize(
    config: AgentConfiguration,
    conductorCommand: string,
  ): Promise<void> {
    // Read settings to build the command
    const vsConfig = vscode.workspace.getConfiguration("symposium");

    // Get log level if configured
    let agentLogLevel = vsConfig.get<string>("agentLogLevel", "");
    if (!agentLogLevel) {
      const generalLogLevel = vsConfig.get<string>("logLevel", "error");
      if (generalLogLevel === "debug") {
        agentLogLevel = "debug";
      }
    }

    // Build the spawn command and args - just use "run" mode
    // Symposium's ConfigAgent handles agent selection and mods
    const spawnArgs: string[] = ["run"];

    if (agentLogLevel) {
      spawnArgs.push("--log", agentLogLevel);
    }

    const traceDir = vsConfig.get<string>("traceDir", "");
    if (traceDir) {
      spawnArgs.push("--trace-dir", traceDir);
    }

    const proxySpawnArgs = vsConfig.get<string[]>("proxySpawnArgs", []);
    for (const arg of proxySpawnArgs) {
      spawnArgs.push(arg);
    }

    logger.important("agent", "Spawning ACP agent", {
      command: conductorCommand,
      args: spawnArgs,
    });

    // Spawn the agent process
    this.agentProcess = spawn(conductorCommand, spawnArgs, {
      stdio: ["pipe", "pipe", "pipe"],
      env: process.env,
      cwd: config.workspaceFolder.uri.fsPath,
    });

    // Capture stderr and pipe to logger
    if (this.agentProcess.stderr) {
      this.agentProcess.stderr.on("data", (data: Buffer) => {
        const lines = data
          .toString()
          .split("\n")
          .filter((line) => line.trim());
        for (const line of lines) {
          logger.info("agent-stderr", line);
        }
      });
    }

    // Create streams for communication
    const input = Writable.toWeb(this.agentProcess.stdin!);
    const output = Readable.toWeb(
      this.agentProcess.stdout!,
    ) as ReadableStream<Uint8Array>;

    // Create the client connection
    const client = new SymposiumClient(this.callbacks);
    const stream = acp.ndJsonStream(input, output);
    this.connection = new acp.ClientSideConnection((_agent) => client, stream);

    // Initialize the connection
    const initResult = await this.connection.initialize({
      protocolVersion: acp.PROTOCOL_VERSION,
      clientCapabilities: {
        fs: {
          readTextFile: false, // TODO: Enable when implemented
          writeTextFile: false,
        },
      },
    });

    logger.important("agent", "Connected to ACP agent", {
      protocolVersion: initResult.protocolVersion,
    });
  }

  /**
   * Create a new agent session
   * @param workspaceFolder - Workspace folder to use as working directory
   * @returns Agent session ID
   */
  async createSession(workspaceFolder: string): Promise<string> {
    if (!this.connection) {
      throw new Error("ACP connection not initialized");
    }

    // Create a session with the ACP agent
    const result = await this.connection.newSession({
      cwd: workspaceFolder,
      mcpServers: [],
    });

    logger.important("agent", "Created agent session", {
      sessionId: result.sessionId,
      cwd: workspaceFolder,
    });
    return result.sessionId;
  }

  /**
   * Send a prompt to an agent session
   * This returns immediately - responses come via callbacks
   *
   * @param agentSessionId - Agent session identifier
   * @param prompt - User prompt text
   */
  async sendPrompt(
    agentSessionId: string,
    prompt: string | acp.ContentBlock[],
  ): Promise<void> {
    if (!this.connection) {
      throw new Error("ACP connection not initialized");
    }

    // Build content blocks
    const contentBlocks: acp.ContentBlock[] =
      typeof prompt === "string" ? [{ type: "text", text: prompt }] : prompt;

    // Log the prompt (truncate text for logging)
    const textContent = contentBlocks
      .filter((b) => b.type === "text")
      .map((b) => (b as { type: "text"; text: string }).text)
      .join("");
    const truncatedPrompt =
      textContent.length > 100
        ? textContent.slice(0, 100) + "..."
        : textContent;
    const resourceCount = contentBlocks.filter(
      (b) => b.type === "resource",
    ).length;

    logger.debug("agent", "Sending prompt to agent session", {
      agentSessionId,
      promptLength: textContent.length,
      prompt: truncatedPrompt,
      resourceCount,
    });

    // Send the prompt (this will complete when agent finishes)
    const promptResult = await this.connection.prompt({
      sessionId: agentSessionId,
      prompt: contentBlocks,
    });

    logger.debug("agent", "Prompt completed", {
      stopReason: promptResult.stopReason,
    });

    // Notify completion
    this.callbacks.onAgentComplete(agentSessionId);
  }

  /**
   * Cancel an ongoing prompt turn for a session.
   *
   * Sends a session/cancel notification to the agent. The agent should:
   * - Stop all language model requests as soon as possible
   * - Abort all tool call invocations in progress
   * - Respond to the original prompt with stopReason: "cancelled"
   *
   * @param agentSessionId - Agent session identifier
   */
  async cancelSession(agentSessionId: string): Promise<void> {
    if (!this.connection) {
      throw new Error("ACP connection not initialized");
    }

    logger.debug("agent", "Cancelling session", { agentSessionId });
    await this.connection.cancel({ sessionId: agentSessionId });
  }

  /**
   * Cleanup - kill the agent process
   */
  dispose(): void {
    if (this.agentProcess) {
      this.agentProcess.kill();
      this.agentProcess = undefined;
    }
  }
}
