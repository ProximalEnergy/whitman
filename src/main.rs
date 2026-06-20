use std::env;
use std::io::{self, Write};

use anyhow::{Context, Result, bail};
use clap::Parser;
use whitman::agents_file::{ApplyOutcome, apply_profile};
use whitman::profile::{default_profiles_dir, list_profiles};
use whitman::tui::run_profile_picker;

#[derive(Debug, Parser)]
#[command(
    name = "whitman",
    version,
    about = "Choose a global profile and link it to ./AGENTS.md"
)]
struct Cli {}

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let _cli = Cli::parse();

    let profiles_dir = default_profiles_dir()?;
    let profiles = list_profiles(&profiles_dir)?;
    if profiles.is_empty() {
        bail!(
            "no profiles found in {}. Add files named agents.<name>.md.",
            profiles_dir.display()
        );
    }

    let Some(selected) = run_profile_picker(&profiles)? else {
        bail!("cancelled");
    };

    let work_dir = env::current_dir().context("failed to determine current directory")?;
    let mut confirmer = |message: &str| prompt_yes_no(message);
    let outcome = apply_profile(&work_dir, &profiles_dir, &selected, &mut confirmer)?;

    match outcome {
        ApplyOutcome::Linked => {
            println!("Linked ./AGENTS.md to profile '{}'.", selected.name);
        }
        ApplyOutcome::ReplacedWhitmanSymlink => {
            println!("Updated ./AGENTS.md to profile '{}'.", selected.name);
        }
        ApplyOutcome::ConvertedExistingFile { old_profile_path } => {
            println!(
                "Saved existing ./AGENTS.md to {} and linked profile '{}'.",
                old_profile_path.display(),
                selected.name
            );
        }
    }

    Ok(())
}

fn prompt_yes_no(message: &str) -> Result<bool> {
    print!("{message} [y/N] ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(matches!(input.trim(), "y" | "Y" | "yes" | "YES" | "Yes"))
}
