use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};

pub const MAX_NAME_LEN_EXCLUSIVE: usize = 15;
pub const MAX_DESCRIPTION_LEN_EXCLUSIVE: usize = 100;
const PROFILE_PREFIX: &str = "AGENTS.";
const PROFILE_SUFFIX: &str = ".md";
pub const DESCRIPTIONS_FILE_NAME: &str = "descriptions.toml";

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Profile {
    pub name: String,
    pub description: String,
    pub path: PathBuf,
}

pub fn repository_agents_dir(work_dir: &Path) -> PathBuf {
    whitman_root(work_dir)
        .unwrap_or_else(|| work_dir.to_path_buf())
        .join(".whitman")
        .join("agents")
}

pub fn list_profiles(agents_dir: &Path) -> Result<Vec<Profile>> {
    if !agents_dir.exists() {
        return Ok(Vec::new());
    }

    let mut profiles = Vec::new();
    for entry in fs::read_dir(agents_dir)
        .with_context(|| format!("failed to read {}", agents_dir.display()))?
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

pub fn create_profile_file(agents_dir: &Path, name: &str, description: &str) -> Result<Profile> {
    validate_profile_name(name)?;
    validate_profile_description(description)?;

    fs::create_dir_all(agents_dir)
        .with_context(|| format!("failed to create {}", agents_dir.display()))?;
    let path = agents_dir.join(profile_file_name(name)?);
    if path.exists() {
        bail!("profile already exists: {}", path.display());
    }

    let mut descriptions = read_profile_descriptions(agents_dir)?;
    if descriptions.contains_key(name) {
        bail!("profile description already exists for {name}");
    }

    fs::write(&path, "# Instructions\n")
        .with_context(|| format!("failed to write {}", path.display()))?;
    descriptions.insert(name.to_string(), parse_description(description)?);
    write_profile_descriptions(agents_dir, &descriptions)?;
    parse_profile_file(&path)
}

pub fn parse_profile_file(path: &Path) -> Result<Profile> {
    let name = profile_name_from_path(path)?;
    validate_profile_name(&name)
        .with_context(|| format!("invalid profile file {}", path.display()))?;

    let agents_dir = path
        .parent()
        .ok_or_else(|| anyhow!("profile path has no parent directory: {}", path.display()))?;
    let descriptions = read_profile_descriptions(agents_dir)?;
    let description = descriptions.get(&name).cloned().ok_or_else(|| {
        anyhow!(
            "missing profile description for '{}' in {}",
            name,
            descriptions_path(agents_dir).display()
        )
    })?;
    let description = parse_description(&description).with_context(|| {
        format!(
            "invalid profile description in {}",
            descriptions_path(agents_dir).display()
        )
    })?;

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
        bail!("profile file name must match AGENTS.<name>.md: {file_name}");
    }

    let name = &file_name[PROFILE_PREFIX.len()..file_name.len() - PROFILE_SUFFIX.len()];
    Ok(name.to_string())
}

pub fn profile_file_name(name: &str) -> Result<String> {
    validate_profile_name(name)?;
    Ok(format!("{PROFILE_PREFIX}{name}{PROFILE_SUFFIX}"))
}

pub fn old_profile_path(agents_dir: &Path) -> PathBuf {
    agents_dir.join("AGENTS.old.md")
}

pub fn descriptions_path(agents_dir: &Path) -> PathBuf {
    agents_dir.join(DESCRIPTIONS_FILE_NAME)
}

pub fn read_profile_descriptions(agents_dir: &Path) -> Result<BTreeMap<String, String>> {
    let path = descriptions_path(agents_dir);
    if !path.exists() {
        return Ok(BTreeMap::new());
    }

    let contents =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    parse_descriptions_file(&contents).with_context(|| format!("invalid {}", path.display()))
}

pub fn write_profile_description(agents_dir: &Path, name: &str, description: &str) -> Result<()> {
    validate_profile_name(name)?;
    let mut descriptions = read_profile_descriptions(agents_dir)?;
    descriptions.insert(name.to_string(), parse_description(description)?);
    write_profile_descriptions(agents_dir, &descriptions)
}

pub fn remove_profile_description(agents_dir: &Path, name: &str) -> Result<()> {
    validate_profile_name(name)?;
    let mut descriptions = read_profile_descriptions(agents_dir)?;
    descriptions.remove(name);
    write_profile_descriptions(agents_dir, &descriptions)
}

pub fn profile_description_exists(agents_dir: &Path, name: &str) -> Result<bool> {
    validate_profile_name(name)?;
    Ok(read_profile_descriptions(agents_dir)?.contains_key(name))
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

pub fn parse_description(contents: &str) -> Result<String> {
    let description = contents.trim();
    if description.is_empty() {
        bail!("description cannot be empty");
    }
    if description.lines().count() > 1 {
        bail!("description must be a single line");
    }
    if description.chars().count() >= MAX_DESCRIPTION_LEN_EXCLUSIVE {
        bail!(
            "description must be under {} characters",
            MAX_DESCRIPTION_LEN_EXCLUSIVE
        );
    }

    Ok(description.to_string())
}

pub fn validate_profile_description(line: &str) -> Result<String> {
    parse_description(line)
}

fn write_profile_descriptions(
    agents_dir: &Path,
    descriptions: &BTreeMap<String, String>,
) -> Result<()> {
    fs::create_dir_all(agents_dir)
        .with_context(|| format!("failed to create {}", agents_dir.display()))?;
    let path = descriptions_path(agents_dir);
    let mut contents = String::new();
    for (name, description) in descriptions {
        validate_profile_name(name)?;
        let description = parse_description(description)?;
        contents.push_str(name);
        contents.push_str(" = \"");
        contents.push_str(&escape_description(&description));
        contents.push_str("\"\n");
    }
    fs::write(&path, contents).with_context(|| format!("failed to write {}", path.display()))
}

fn parse_descriptions_file(contents: &str) -> Result<BTreeMap<String, String>> {
    let mut descriptions = BTreeMap::new();
    for (index, raw_line) in contents.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let (name, value) = line.split_once('=').ok_or_else(|| {
            anyhow!(
                "line {} must be formatted as name = \"description\"",
                index + 1
            )
        })?;
        let name = name.trim();
        validate_profile_name(name)
            .with_context(|| format!("invalid profile name on line {}", index + 1))?;
        let description = parse_quoted_description(value.trim())
            .with_context(|| format!("invalid description on line {}", index + 1))?;
        if descriptions
            .insert(name.to_string(), parse_description(&description)?)
            .is_some()
        {
            bail!("duplicate profile description for {name}");
        }
    }

    Ok(descriptions)
}

fn parse_quoted_description(value: &str) -> Result<String> {
    if !value.starts_with('"') || !value.ends_with('"') || value.len() < 2 {
        bail!("description must be a quoted string");
    }

    let inner = &value[1..value.len() - 1];
    let mut parsed = String::new();
    let mut escaped = false;
    for character in inner.chars() {
        if escaped {
            match character {
                '"' => parsed.push('"'),
                '\\' => parsed.push('\\'),
                'n' => parsed.push('\n'),
                'r' => parsed.push('\r'),
                't' => parsed.push('\t'),
                other => bail!("unsupported escape sequence \\{other}"),
            }
            escaped = false;
        } else if character == '\\' {
            escaped = true;
        } else {
            parsed.push(character);
        }
    }

    if escaped {
        bail!("description cannot end with an unfinished escape");
    }

    Ok(parsed)
}

fn escape_description(description: &str) -> String {
    description.replace('\\', "\\\\").replace('"', "\\\"")
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

fn whitman_root(work_dir: &Path) -> Option<PathBuf> {
    work_dir
        .ancestors()
        .find(|path| path.join(".whitman").exists())
        .or_else(|| work_dir.ancestors().find(|path| path.join(".git").exists()))
        .map(Path::to_path_buf)
}
