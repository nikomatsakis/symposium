#!/usr/bin/env cargo
//! Symposium CI Tool
//!
//! Builds and tests all Symposium components for continuous integration

use anyhow::{Context, Result, anyhow};
use clap::Parser;
use std::path::PathBuf;
use std::process::Command;

#[derive(Parser)]
#[command(
    name = "ci",
    about = "Symposium CI tool for building and testing all components",
    long_about = r#"
Symposium CI tool for building and testing all components

Examples:
  cargo ci                             # Check compilation (default)
  cargo ci check                       # Check that all components compile
  cargo ci test                        # Run all tests

Components:
  - Rust workspace (cargo check --workspace)
"#
)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Parser)]
enum Commands {
    /// Check that all components compile
    Check,
    /// Run all tests
    Test,
}

fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Some(Commands::Check) => run_check(),
        Some(Commands::Test) => run_test(),
        None => run_check(), // Default to check
    }
}

/// Check that all components compile
fn run_check() -> Result<()> {
    println!("🤖 Symposium CI Check");
    println!("{}", "=".repeat(26));

    // Check basic prerequisites
    check_rust()?;

    // Check all Rust workspace components compile
    check_rust_workspace()?;

    println!("\n✅ All components check passed!");
    Ok(())
}

/// Run all tests
fn run_test() -> Result<()> {
    println!("🤖 Symposium CI Test");
    println!("{}", "=".repeat(25));

    // Check basic prerequisites
    check_rust()?;

    // Run tests for all Rust workspace components
    run_rust_tests()?;

    println!("\n✅ All tests completed!");
    Ok(())
}

fn check_rust() -> Result<()> {
    if which::which("cargo").is_err() {
        return Err(anyhow!(
            "❌ Error: Cargo not found. Please install Rust first.\n   Visit: https://rustup.rs/"
        ));
    }
    Ok(())
}

fn get_repo_root() -> Result<PathBuf> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .context("❌ CI tool must be run via cargo. CARGO_MANIFEST_DIR not found.")?;

    let manifest_path = PathBuf::from(manifest_dir);
    // If we're in the ci/ directory, go up to workspace root
    if manifest_path.file_name() == Some(std::ffi::OsStr::new("ci")) {
        if let Some(parent) = manifest_path.parent() {
            return Ok(parent.to_path_buf());
        }
    }
    Ok(manifest_path)
}

/// Check Rust workspace compilation
fn check_rust_workspace() -> Result<()> {
    let repo_root = get_repo_root()?;

    println!("🦀 Checking Rust workspace...");
    println!("   Checking in: {}", repo_root.display());

    let output = Command::new("cargo")
        .args(["check", "--workspace"])
        .current_dir(&repo_root)
        .output()
        .context("Failed to execute cargo check")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!(
            "❌ Failed to check Rust workspace:\n   Error: {}",
            stderr.trim()
        ));
    }

    println!("✅ Rust workspace check passed!");
    Ok(())
}

/// Run Rust tests
fn run_rust_tests() -> Result<()> {
    let repo_root = get_repo_root()?;

    println!("🦀 Running Rust tests...");
    println!("   Testing workspace in: {}", repo_root.display());

    let status = Command::new("cargo")
        .args(["test", "--workspace"])
        .env("RUST_BACKTRACE", "1")
        .current_dir(&repo_root)
        .status()
        .context("Failed to execute cargo test")?;

    if !status.success() {
        return Err(anyhow!("❌ Rust tests failed"));
    }

    println!("✅ Rust tests passed!");
    Ok(())
}
