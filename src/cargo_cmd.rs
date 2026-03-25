use std::ffi::OsString;
use std::process::ExitCode;
use symposium_rtk::cargo_cmd::{self, CargoCommand};

pub fn run(args: Vec<String>) -> ExitCode {
    let Some(subcommand) = args.first() else {
        eprintln!("Usage: symposium cargo <subcommand> [args...]");
        return ExitCode::FAILURE;
    };

    let rest = &args[1..];

    // For known commands, use rtk's filtered output.
    // For everything else, pass through to cargo directly.
    let known = match subcommand.as_str() {
        "check" => Some(CargoCommand::Check),
        "build" => Some(CargoCommand::Build),
        "test" => Some(CargoCommand::Test),
        "clippy" => Some(CargoCommand::Clippy),
        "install" => Some(CargoCommand::Install),
        "nextest" => Some(CargoCommand::Nextest),
        _ => None,
    };

    let result = match known {
        Some(cmd) => cargo_cmd::run(cmd, rest, 0),
        None => {
            let os_args: Vec<OsString> = args.iter().map(OsString::from).collect();
            cargo_cmd::run_passthrough(&os_args, 0)
        }
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::FAILURE
        }
    }
}
