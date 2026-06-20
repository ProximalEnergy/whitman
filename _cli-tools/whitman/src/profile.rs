use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};
use directories::UserDirs;

pub const MAX_NAME_LEN_EXCLUSIVE: usize = 15;
pub const MAX_DESCRIPTION_LEN_EXCLUSIVE: usize = 100;
const PROFILE_PREFIX: &str = "agents.";
const PROFILE_SUFFIX: &str = ".md";
const DESCRIPTION_PREFIX: &str = "<!-- whitman:";
const DESCRIPTION_SUFFIX: &str = "-->";

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Profile {
    pub name: String,
    pub description: String,
    pub path: PathBuf,
}

pub fn default_profiles_dir() -> Result<PathBuf> {
    let home = UserDirs::new()
        .ok_or_else(|| anyhow!("could not determine the user's home directory"))?
        .home_dir()
        .to_path_buf();

    Ok(home.join(".whitman").join("profiles"))
}

pub fn list_profiles(profiles_dir: &Path) -> Result<Vec<Profile>> {
    if !profiles_dir.exists() {
        return Ok(Vec::new());
    }

    let mut profiles = Vec::new();
    for entry in fs::read_dir(profiles_dir)
        .with_context(|| format!("failed to read {}", profiles_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() || !is_profile_file_name(&path) {
            continue;
        }
        profiles.push(parse_profile_file(&path)?);
    }

    profiles.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(profiles)
}

pub fn parse_profile_file(path: &Path) -> Result<Profile> {
    let name = profile_name_from_path(path)?;
    validate_profile_name(&name)
        .with_context(|| format!("invalid profile file {}", path.display()))?;

    let contents =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    let first_line = contents
        .lines()
        .next()
        .ok_or_else(|| anyhow!("profile {} is empty", path.display()))?;
    let description = parse_description(first_line)
        .with_context(|| format!("invalid profile description in {}", path.display()))?;

    Ok(Profile {
        name,
        description,
        path: path.to_path_buf(),
    })
}

pub fn profile_name_from_path(path: &Path) -> Result<String> {
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| {
            anyhow!(
                "profile path has no valid UTF-8 file name: {}",
                path.display()
            )
        })?;

    if !file_name.starts_with(PROFILE_PREFIX) || !file_name.ends_with(PROFILE_SUFFIX) {
        bail!("profile file name must match agents.<name>.md: {file_name}");
    }

    let name = &file_name[PROFILE_PREFIX.len()..file_name.len() - PROFILE_SUFFIX.len()];
    Ok(name.to_string())
}

pub fn profile_file_name(name: &str) -> Result<String> {
    validate_profile_name(name)?;
    Ok(format!("{PROFILE_PREFIX}{name}{PROFILE_SUFFIX}"))
}

pub fn old_profile_path(profiles_dir: &Path) -> PathBuf {
    profiles_dir.join("agents.old.md")
}

pub fn validate_profile_name(name: &str) -> Result<()> {
    if name.is_empty() {
        bail!("profile name cannot be empty");
    }
    if name.chars().count() >= MAX_NAME_LEN_EXCLUSIVE {
        bail!(
            "profile name must be under {} characters",
            MAX_NAME_LEN_EXCLUSIVE
        );
    }
    if !name
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || character == '_' || character == '-')
    {
        bail!("profile name may only contain ASCII letters, numbers, underscores, and hyphens");
    }
    Ok(())
}

pub fn parse_description(line: &str) -> Result<String> {
    let trimmed = line.trim();
    if !trimmed.starts_with(DESCRIPTION_PREFIX) || !trimmed.ends_with(DESCRIPTION_SUFFIX) {
        bail!("first line must be formatted as <!-- whitman: description -->");
    }

    let description =
        trimmed[DESCRIPTION_PREFIX.len()..trimmed.len() - DESCRIPTION_SUFFIX.len()].trim();
    if description.is_empty() {
        bail!("description cannot be empty");
    }
    if description.chars().count() >= MAX_DESCRIPTION_LEN_EXCLUSIVE {
        bail!(
            "description must be under {} characters",
            MAX_DESCRIPTION_LEN_EXCLUSIVE
        );
    }

    Ok(description.to_string())
}

pub fn filter_profiles<'a>(profiles: &'a [Profile], query: &str) -> Vec<&'a Profile> {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return profiles.iter().collect();
    }

    profiles
        .iter()
        .filter(|profile| {
            profile.name.to_lowercase().contains(&query)
                || profile.description.to_lowercase().contains(&query)
        })
        .collect()
}

fn is_profile_file_name(path: &Path) -> bool {
    path.file_name()
        .and_then(|value| value.to_str())
        .is_some_and(|file_name| {
            file_name.starts_with(PROFILE_PREFIX) && file_name.ends_with(PROFILE_SUFFIX)
        })
}
