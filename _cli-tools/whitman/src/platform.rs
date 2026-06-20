use std::fs;
use std::io;
use std::path::Path;

use anyhow::{Context, Result};

pub fn create_file_symlink(target: &Path, link: &Path) -> Result<()> {
    if let Some(parent) = link.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    create_file_symlink_inner(target, link).with_context(|| {
        format!(
            "failed to symlink {} -> {}",
            link.display(),
            target.display()
        )
    })
}

#[cfg(unix)]
fn create_file_symlink_inner(target: &Path, link: &Path) -> Result<()> {
    std::os::unix::fs::symlink(target, link)?;
    Ok(())
}

#[cfg(windows)]
fn create_file_symlink_inner(target: &Path, link: &Path) -> Result<()> {
    match std::os::windows::fs::symlink_file(target, link) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::PermissionDenied => {
            anyhow::bail!("{}", windows_symlink_setup_instructions(&error))
        }
        Err(error) => Err(error.into()),
    }
}

#[cfg(not(windows))]
#[allow(dead_code)]
fn _keep_io_import_used(_: io::ErrorKind) {}

pub fn windows_symlink_setup_instructions(error: &io::Error) -> String {
    format!(
        "{error}. Windows file symlinks require Developer Mode or an elevated terminal. Enable Developer Mode in Settings > System > For developers, or run whitman from an Administrator terminal. Whitman does not copy profile files as a fallback."
    )
}
