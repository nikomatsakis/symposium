/**
 * Editor state tracker - writes active file and selection state to a JSON file
 * that the editor-context proxy reads to inject into agent prompts.
 */

import * as vscode from "vscode";
import * as fs from "fs";
import * as os from "os";
import * as path from "path";
import * as crypto from "crypto";

interface EditorState {
  activeFile: string | null;
  languageId: string | null;
  selection: {
    text: string;
    startLine: number;
    endLine: number;
  } | null;
  workspaceFolders: string[];
}

export class EditorStateTracker implements vscode.Disposable {
  private disposables: vscode.Disposable[] = [];
  private writeTimer: ReturnType<typeof setTimeout> | undefined;
  private stateFilePath: string;

  constructor() {
    this.stateFilePath = EditorStateTracker.defaultStateFilePath();

    // Track active editor changes
    this.disposables.push(
      vscode.window.onDidChangeActiveTextEditor(() => this.scheduleWrite()),
    );

    // Track selection changes
    this.disposables.push(
      vscode.window.onDidChangeTextEditorSelection(() => this.scheduleWrite()),
    );

    // Write initial state
    this.writeState();
  }

  /** Path to the state file, for passing to the spawned process. */
  get filePath(): string {
    return this.stateFilePath;
  }

  /** Debounced write - avoids thrashing during rapid cursor movement. */
  private scheduleWrite(): void {
    if (this.writeTimer) {
      clearTimeout(this.writeTimer);
    }
    this.writeTimer = setTimeout(() => this.writeState(), 100);
  }

  private writeState(): void {
    const editor = vscode.window.activeTextEditor;
    const state: EditorState = {
      activeFile:
        editor?.document.uri.scheme === "file"
          ? editor.document.uri.fsPath
          : null,
      languageId: editor?.document.languageId ?? null,
      selection: null,
      workspaceFolders:
        vscode.workspace.workspaceFolders?.map((f) => f.uri.fsPath) ?? [],
    };

    // Include selection only when non-empty
    if (editor && !editor.selection.isEmpty) {
      const text = editor.document.getText(editor.selection);
      state.selection = {
        text,
        startLine: editor.selection.start.line + 1, // 1-based
        endLine: editor.selection.end.line + 1,
      };
    }

    try {
      // Write to temp file then rename for atomicity — prevents the Rust
      // proxy from reading a partially-written file.
      const tmp = this.stateFilePath + ".tmp";
      fs.writeFileSync(tmp, JSON.stringify(state), "utf-8");
      fs.renameSync(tmp, this.stateFilePath);
    } catch {
      // Best-effort - don't crash if temp dir is unavailable
    }
  }

  /** Generate a stable, workspace-scoped path in the temp directory. */
  static defaultStateFilePath(): string {
    const workspaceId =
      vscode.workspace.workspaceFolders?.[0]?.uri.fsPath ?? "default";
    const hash = crypto
      .createHash("sha256")
      .update(workspaceId)
      .digest("hex")
      .slice(0, 12);
    return path.join(os.tmpdir(), `symposium-editor-state-${hash}.json`);
  }

  dispose(): void {
    if (this.writeTimer) {
      clearTimeout(this.writeTimer);
    }
    // Clean up state file
    try {
      fs.unlinkSync(this.stateFilePath);
    } catch {
      // ignore
    }
    for (const d of this.disposables) {
      d.dispose();
    }
  }
}
