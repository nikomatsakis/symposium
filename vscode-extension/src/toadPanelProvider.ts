import * as vscode from "vscode";
import * as net from "net";
import * as http from "http";
import { ChildProcess, spawn } from "child_process";
import { getConductorCommand } from "./binaryPath";
import { logger } from "./extension";

interface ToadProcessInfo {
  pid: number;
  port: number;
}

const STATE_KEY = "symposium.toadProcess";

export class ToadPanelProvider implements vscode.WebviewViewProvider {
  static readonly viewType = "symposium.toadView";

  private context: vscode.ExtensionContext;
  private childProcess: ChildProcess | undefined;

  constructor(context: vscode.ExtensionContext) {
    this.context = context;
  }

  async resolveWebviewView(
    webviewView: vscode.WebviewView,
    _resolveContext: vscode.WebviewViewResolveContext,
    _token: vscode.CancellationToken,
  ): Promise<void> {
    webviewView.webview.options = {
      enableScripts: true,
      localResourceRoots: [], // all content comes from localhost
    };

    try {
      const port = await this.ensureToadRunning();
      webviewView.webview.html = this.getHtml(port);
      logger.important("toad", `Toad panel connected on port ${port}`);
    } catch (err: any) {
      logger.error("toad", "Failed to start Toad", { error: err.message });
      webviewView.webview.html = this.getErrorHtml(err.message);
    }
  }

  private async ensureToadRunning(): Promise<number> {
    // Check if we have a previously running process
    const saved = this.context.globalState.get<ToadProcessInfo>(STATE_KEY);
    if (saved) {
      const alive = await this.isPortResponding(saved.port);
      if (alive) {
        logger.info("toad", `Reusing existing Toad process`, {
          pid: saved.pid,
          port: saved.port,
        });
        return saved.port;
      }
      logger.info("toad", `Previous Toad process is gone, starting new one`);
    }

    // Start a new Toad process
    const config = vscode.workspace.getConfiguration("symposium");
    const configuredPort = config.get<number>("toadPort", 0);
    const port =
      configuredPort > 0 ? configuredPort : await this.findFreePort();

    await this.spawnToad(port);
    return port;
  }

  private async spawnToad(port: number): Promise<void> {
    const config = vscode.workspace.getConfiguration("symposium");
    const toadCommand = config.get<string>("toadCommand", "toad");
    const conductorCommand = getConductorCommand(this.context);

    const workspaceFolder =
      vscode.workspace.workspaceFolders?.[0]?.uri.fsPath ?? process.cwd();

    const args = [
      "acp",
      `${conductorCommand} run`,
      "--serve",
      "--port",
      String(port),
    ];

    logger.info("toad", `Spawning: ${toadCommand} ${args.join(" ")}`, {
      cwd: workspaceFolder,
    });

    const child = spawn(toadCommand, args, {
      cwd: workspaceFolder,
      stdio: ["ignore", "pipe", "pipe"],
      detached: true,
    });

    child.unref(); // allow the process to survive extension host restarts

    // Capture stderr for logging
    child.stderr?.on("data", (data: Buffer) => {
      logger.debug("toad-stderr", data.toString().trimEnd());
    });

    child.stdout?.on("data", (data: Buffer) => {
      logger.debug("toad-stdout", data.toString().trimEnd());
    });

    child.on("error", (err) => {
      logger.error("toad", `Toad process error: ${err.message}`);
    });

    child.on("exit", (code, signal) => {
      logger.info("toad", `Toad process exited`, { code, signal });
      this.context.globalState.update(STATE_KEY, undefined);
      this.childProcess = undefined;
    });

    this.childProcess = child;

    // Save process info for reconnection after reload
    if (child.pid) {
      await this.context.globalState.update(STATE_KEY, {
        pid: child.pid,
        port,
      } as ToadProcessInfo);
    }

    // Wait for the server to become ready
    await this.waitForPort(port, 15000);
  }

  private async findFreePort(): Promise<number> {
    return new Promise((resolve, reject) => {
      const server = net.createServer();
      server.listen(0, () => {
        const addr = server.address();
        if (addr && typeof addr === "object") {
          const port = addr.port;
          server.close(() => resolve(port));
        } else {
          server.close(() => reject(new Error("Could not determine port")));
        }
      });
      server.on("error", reject);
    });
  }

  private async isPortResponding(port: number): Promise<boolean> {
    return new Promise((resolve) => {
      const req = http.get(`http://localhost:${port}/`, (res) => {
        res.resume(); // consume response
        resolve(true);
      });
      req.on("error", () => resolve(false));
      req.setTimeout(2000, () => {
        req.destroy();
        resolve(false);
      });
    });
  }

  private async waitForPort(port: number, timeoutMs: number): Promise<void> {
    const start = Date.now();
    while (Date.now() - start < timeoutMs) {
      if (await this.isPortResponding(port)) {
        return;
      }
      await new Promise((r) => setTimeout(r, 300));
    }
    throw new Error(
      `Toad server did not become ready on port ${port} within ${timeoutMs}ms`,
    );
  }

  private getHtml(port: number): string {
    return `<!DOCTYPE html>
<html lang="en" style="height: 100%; margin: 0; padding: 0;">
<head>
  <meta charset="UTF-8">
  <meta http-equiv="Content-Security-Policy"
        content="default-src 'none'; frame-src http://localhost:${port}; style-src 'unsafe-inline'; script-src 'unsafe-inline';">
</head>
<body style="height: 100%; margin: 0; padding: 0; overflow: hidden;">
  <iframe
    id="toad-frame"
    src="http://localhost:${port}/"
    style="width: 100%; height: 100%; border: none;"
    sandbox="allow-scripts allow-same-origin allow-forms allow-popups"
  ></iframe>
  <script>
    const frame = document.getElementById('toad-frame');
    frame.addEventListener('load', () => {
      frame.focus();
      try { frame.contentWindow.focus(); } catch(e) {}
    });
    // Also focus on any click in the webview
    document.addEventListener('click', () => {
      frame.focus();
      try { frame.contentWindow.focus(); } catch(e) {}
    });
  </script>
</body>
</html>`;
  }

  private getErrorHtml(message: string): string {
    return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src 'unsafe-inline';">
</head>
<body style="padding: 20px; font-family: var(--vscode-font-family); color: var(--vscode-foreground);">
  <h3>Failed to start Symposium</h3>
  <p>${this.escapeHtml(message)}</p>
  <p>Make sure <code>toad</code> is installed and available on your PATH.</p>
  <p>You can configure the path in Settings: <code>symposium.toadCommand</code></p>
</body>
</html>`;
  }

  private escapeHtml(text: string): string {
    return text
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;");
  }
}
