/**
 * Extension Registry - Minimal stub
 *
 * Extension management is handled by the Symposium Rust agent via ConfigAgent.
 * This file is kept minimal for now - VS Code just spawns `symposium-acp-agent run`.
 */

// Minimal types kept for any remaining references
export interface Distribution {
  local?: { command: string; args?: string[] };
  npx?: { package: string };
  pipx?: { package: string };
  url?: { url: string };
  symposium?: { subcommand: string };
}
