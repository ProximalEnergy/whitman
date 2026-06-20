use std::fs;
use std::path::{Component, Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};

use crate::platform::create_file_symlink;
use crate::profile::{Profile, old_profile_path};

pub const AGENTS_FILE_NAME: &str = "AGENTS.md";
const OLD_PROFILE_HEADER: &[u8] = b"<!-- whitman: Converted from AGENTS.md -->\n";

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ApplyOutcome {
    Linked,
    ReplacedWhitmanSymlink,
    ConvertedExistingFile { old_profile_path: PathBuf },
}

pub trait Confirm {
    fn confirm(&mut self, message: &str) -> Result<bool>;
}

impl<F> Confirm for F
where
    F: FnMut(&str) -> Result<bool>,
{
    fn confirm(&mut self, message: &str) -> Result<bool> {
        self(message)
    }
}

pub fn apply_profile(
    work_dir: &Path,
    agents_dir: &Path,
    profile: &Profile,
    confirmer: &mut impl Confirm,
) -> Result<ApplyOutcome> {
    fs::create_dir_all(agents_dir)
        .with_context(|| format!("failed to create {}", agents_dir.display()))?;

    let agents_path = work_dir.join(AGENTS_FILE_NAME);
    if !agents_path.exists() && !is_symlink(&agents_path)? {
        create_file_symlink(&profile.path, &agents_path)?;
        return Ok(ApplyOutcome::Linked);
    }

    let metadata = fs::symlink_metadata(&agents_path)
        .with_context(|| format!("failed to inspect {}", agents_path.display()))?;

    if metadata.file_type().is_symlink() {
        ensure_whitman_symlink(&agents_path, agents_dir)?;
        fs::remove_file(&agents_path)
            .with_context(|| format!("failed to remove {}", agents_path.display()))?;
        create_file_symlink(&profile.path, &agents_path)?;
        return Ok(ApplyOutcome::ReplacedWhitmanSymlink);
    }

    if !metadata.is_file() {
        bail!(
            "{} exists and is not a regular file or symlink; refusing to replace it",
            agents_path.display()
        );
    }

    let old_path = old_profile_path(agents_dir);
    if old_path.exists() || is_symlink(&old_path)? {
        let confirmed = confirmer.confirm(&format!(
            "{} already exists. Overwrite it with the current AGENTS.md?",
            old_path.display()
        ))?;
        if !confirmed {
            bail!("cancelled before overwriting {}", old_path.display());
        }
        remove_existing_profile(&old_path)?;
    }

    convert_agents_file_to_old_profile(&agents_path, &old_path)?;
    fs::remove_file(&agents_path)
        .with_context(|| format!("failed to remove {}", agents_path.display()))?;
    create_file_symlink(&profile.path, &agents_path)?;

    Ok(ApplyOutcome::ConvertedExistingFile {
        old_profile_path: old_path,
    })
}

fn convert_agents_file_to_old_profile(agents_path: &Path, old_path: &Path) -> Result<()> {
    let existing = fs::read(agents_path)
        .with_context(|| format!("failed to read {}", agents_path.display()))?;
    let mut converted = Vec::with_capacity(OLD_PROFILE_HEADER.len() + existing.len());
    converted.extend_from_slice(OLD_PROFILE_HEADER);
    converted.extend_from_slice(&existing);

    fs::write(old_path, converted)
        .with_context(|| format!("failed to write converted profile {}", old_path.display()))
}

pub fn ensure_whitman_symlink(agents_path: &Path, agents_dir: &Path) -> Result<()> {
    let target = fs::read_link(agents_path)
        .with_context(|| format!("failed to read symlink {}", agents_path.display()))?;
    let resolved_target = resolve_link_target(agents_path, &target)?;
    let agents_dir = normalize_absolute(agents_dir)?;

    if !resolved_target.starts_with(&agents_dir) {
        bail!(
            "{} is a symlink to {}, which is outside {}; refusing to overwrite it",
            agents_path.display(),
            resolved_target.display(),
            agents_dir.display()
        );
    }

    Ok(())
}

fn remove_existing_profile(path: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("failed to inspect {}", path.display()))?;
    if metadata.is_file() || metadata.file_type().is_symlink() {
        fs::remove_file(path).with_context(|| format!("failed to remove {}", path.display()))?;
        return Ok(());
    }

    bail!(
        "{} exists and is not a file; refusing to overwrite it",
        path.display()
    )
}

fn is_symlink(path: &Path) -> Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => Ok(metadata.file_type().is_symlink()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error).with_context(|| format!("failed to inspect {}", path.display())),
    }
}

fn resolve_link_target(link_path: &Path, target: &Path) -> Result<PathBuf> {
    let absolute = if target.is_absolute() {
        target.to_path_buf()
    } else {
        link_path
            .parent()
            .ok_or_else(|| anyhow!("{} has no parent directory", link_path.display()))?
            .join(target)
    };
    normalize_absolute(&absolute)
}

fn normalize_absolute(path: &Path) -> Result<PathBuf> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };

    let mut normalized = PathBuf::new();
    for component in absolute.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Normal(_) | Component::RootDir | Component::Prefix(_) => {
                normalized.push(component.as_os_str());
            }
        }
    }

    Ok(normalized)
}
