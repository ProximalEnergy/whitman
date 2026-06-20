use std::env;
use std::io::{self, Write};

use anyhow::{Context, Result, bail};
use clap::Parser;
use whitman::agents_file::{ApplyOutcome, apply_profile};
use whitman::profile::{list_profiles, repository_agents_dir};
use whitman::tui::run_profile_picker;

#[derive(Debug, Parser)]
#[command(
    name = "whitman",
    version,
    about = "Choose a repository profile and link it to ./AGENTS.md"
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

    let work_dir = env::current_dir().context("failed to determine current directory")?;
    let agents_dir = repository_agents_dir(&work_dir);
    let mut profiles = list_profiles(&agents_dir)?;

    let Some(selected) = run_profile_picker(&agents_dir, &mut profiles)? else {
        bail!("cancelled");
    };

    let mut confirmer = |message: &str| prompt_yes_no(message);
    let outcome = apply_profile(&work_dir, &agents_dir, &selected, &mut confirmer)?;

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
