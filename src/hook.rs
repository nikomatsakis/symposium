use std::io::Read;
use std::process::ExitCode;

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum HookEvent {
    /// Claude Code PreToolUse hook
    #[value(name = "claude:pre-tool-use")]
    ClaudePreToolUse,
}

pub fn run(event: HookEvent) -> ExitCode {
    let mut input = String::new();
    if let Err(e) = std::io::stdin().read_to_string(&mut input) {
        tracing::warn!(?event, error = %e, "failed to read hook stdin");
        return ExitCode::SUCCESS;
    }

    tracing::info!(?event, "hook invoked");
    tracing::debug!(?event, payload = %input, "hook payload");

    ExitCode::SUCCESS
}
