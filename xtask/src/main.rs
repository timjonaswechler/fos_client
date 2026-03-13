use anyhow::{bail, Result};
use clap::{Parser, Subcommand};
use std::process::Command;

#[derive(Parser)]
#[command(name = "cargo xtask", about = "Dev task runner for fos_client")]
struct Cli {
    #[command(subcommand)]
    command: Task,
}

#[derive(Subcommand)]
enum Task {
    /// Create a new release: bump version, commit, tag, push
    Release {
        /// Version to release (e.g. 0.2.0 or v0.2.0)
        version: String,
    },
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args()
        .enumerate()
        .filter_map(|(i, a)| if i == 1 && a == "xtask" { None } else { Some(a) })
        .collect();

    let cli = Cli::parse_from(args);

    match cli.command {
        Task::Release { version } => release(version),
    }
}

fn release(version: String) -> Result<()> {
    // Strip leading 'v' — prevent vv0.1.0
    let version = version.trim_start_matches('v');

    // Validate semver format x.y.z
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() < 3 || parts.iter().any(|p| p.parse::<u32>().is_err()) {
        bail!("Invalid version: '{}'. Expected format: x.y.z (e.g. 0.2.0)", version);
    }

    let tag = format!("v{}", version);

    // Read current version from Cargo.toml
    let cargo_toml = std::fs::read_to_string("Cargo.toml")?;
    let current = cargo_toml
        .lines()
        .find(|l| l.starts_with("version = "))
        .and_then(|l| l.split('"').nth(1))
        .unwrap_or("unknown");

    println!("Current: {} → New: {}", current, version);

    // Check working tree is clean
    let status = cmd("git", &["status", "--porcelain"])?;
    if !status.trim().is_empty() {
        bail!("Working tree is dirty. Commit or stash changes first.");
    }

    // Bump version in Cargo.toml
    let updated = cargo_toml.replacen(
        &format!("version = \"{}\"", current),
        &format!("version = \"{}\"", version),
        1,
    );
    std::fs::write("Cargo.toml", updated)?;

    // Commit
    cmd("git", &["add", "Cargo.toml"])?;
    cmd("git", &["commit", "-m", &format!("chore(release): {}", tag)])?;

    // Tag
    cmd("git", &["tag", &tag])?;

    // Push
    cmd("git", &["push", "origin", "main"])?;
    cmd("git", &["push", "origin", &tag])?;

    println!("✅ Released {}", tag);
    Ok(())
}

fn cmd(program: &str, args: &[&str]) -> Result<String> {
    let output = Command::new(program).args(args).output()?;
    if !output.status.success() {
        bail!(
            "{} {} failed:\n{}",
            program,
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
