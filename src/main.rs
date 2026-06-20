use std::env;
use std::io::{self, Write};
use std::path::Path;

use anyhow::{Context, Result, bail};
use clap::Parser;
use whitman::agents_file::{ApplyOutcome, apply_profile};
use whitman::profile::{
    Profile, create_profile_file, list_profiles, repository_agents_dir,
    validate_profile_description, validate_profile_name,
};
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
    if profiles.is_empty() {
        let Some(profile) = prompt_create_profile(&agents_dir)? else {
            bail!("cancelled");
        };
        profiles.push(profile);
    }

    let Some(selected) = run_profile_picker(&profiles)? else {
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

fn prompt_create_profile(agents_dir: &Path) -> Result<Option<Profile>> {
    println!("No profiles found in {}.", agents_dir.display());
    if !prompt_yes_no("Create one now?")? {
        return Ok(None);
    }

    let name = prompt_profile_name()?;
    let description = prompt_profile_description()?;
    let profile = create_profile_file(agents_dir, &name, &description)?;
    println!("Created {}.", profile.path.display());
    Ok(Some(profile))
}

fn prompt_profile_name() -> Result<String> {
    loop {
        let name = prompt_line("Profile name")?;
        match validate_profile_name(&name) {
            Ok(()) => return Ok(name),
            Err(error) => println!("Invalid profile name: {error:#}"),
        }
    }
}

fn prompt_profile_description() -> Result<String> {
    loop {
        let description = prompt_line("Description")?;
        match validate_profile_description(&description) {
            Ok(_) => return Ok(description),
            Err(error) => println!("Invalid description: {error:#}"),
        }
    }
}

fn prompt_line(message: &str) -> Result<String> {
    print!("{message}: ");
    io::stdout().flush()?;

    let mut input = String::new();
    let bytes = io::stdin().read_line(&mut input)?;
    if bytes == 0 {
        bail!("cancelled");
    }
    Ok(input.trim().to_string())
}
