use std::fs;
use std::path::Path;

use anyhow::Result;
use tempfile::tempdir;
use whitman::agents_file::{AGENTS_FILE_NAME, ApplyOutcome, apply_profile};
use whitman::platform::windows_symlink_setup_instructions;
use whitman::profile::{Profile, old_profile_path, parse_profile_file};

#[cfg(unix)]
use std::os::unix::fs as unix_fs;
#[cfg(windows)]
use std::os::windows::fs as windows_fs;

#[test]
fn creates_symlink_when_agents_file_is_absent() -> Result<()> {
    let fixture = Fixture::new()?;
    let profile = fixture.write_profile("work", "Work")?;
    let mut confirmer = |_message: &str| Ok(false);

    let Some(outcome) = apply_or_skip_symlink_permission(
        fixture.work_dir.path(),
        fixture.agents_dir.path(),
        &profile,
        &mut confirmer,
    )?
    else {
        return Ok(());
    };

    assert_eq!(outcome, ApplyOutcome::Linked);
    assert_symlink_target(
        &fixture.work_dir.path().join(AGENTS_FILE_NAME),
        &profile.path,
    )?;
    Ok(())
}

#[test]
fn replaces_existing_whitman_symlink() -> Result<()> {
    let fixture = Fixture::new()?;
    let old_profile = fixture.write_profile("old", "Old")?;
    let new_profile = fixture.write_profile("new", "New")?;
    if !symlink_file(
        &old_profile.path,
        &fixture.work_dir.path().join(AGENTS_FILE_NAME),
    )? {
        return Ok(());
    }
    let mut confirmer = |_message: &str| Ok(false);

    let Some(outcome) = apply_or_skip_symlink_permission(
        fixture.work_dir.path(),
        fixture.agents_dir.path(),
        &new_profile,
        &mut confirmer,
    )?
    else {
        return Ok(());
    };

    assert_eq!(outcome, ApplyOutcome::ReplacedWhitmanSymlink);
    assert_symlink_target(
        &fixture.work_dir.path().join(AGENTS_FILE_NAME),
        &new_profile.path,
    )?;
    Ok(())
}

#[test]
fn refuses_external_agents_symlink_without_overwrite() -> Result<()> {
    let fixture = Fixture::new()?;
    let profile = fixture.write_profile("work", "Work")?;
    let external = fixture.work_dir.path().join("external.md");
    fs::write(&external, "external")?;
    if !symlink_file(&external, &fixture.work_dir.path().join(AGENTS_FILE_NAME))? {
        return Ok(());
    }
    let mut confirmer = |_message: &str| Ok(true);

    let error = apply_profile(
        fixture.work_dir.path(),
        fixture.agents_dir.path(),
        &profile,
        &mut confirmer,
    )
    .unwrap_err()
    .to_string();

    assert!(error.contains("outside"));
    assert_symlink_target(&fixture.work_dir.path().join(AGENTS_FILE_NAME), &external)?;
    Ok(())
}

#[test]
fn converts_existing_agents_file_to_old_profile() -> Result<()> {
    let fixture = Fixture::new()?;
    let profile = fixture.write_profile("work", "Work")?;
    let agents_path = fixture.work_dir.path().join(AGENTS_FILE_NAME);
    fs::write(&agents_path, "legacy agents")?;
    let mut confirmer = |_message: &str| Ok(false);

    let Some(outcome) = apply_or_skip_symlink_permission(
        fixture.work_dir.path(),
        fixture.agents_dir.path(),
        &profile,
        &mut confirmer,
    )?
    else {
        return Ok(());
    };

    let expected_old_path = old_profile_path(fixture.agents_dir.path());
    assert_eq!(
        outcome,
        ApplyOutcome::ConvertedExistingFile {
            old_profile_path: expected_old_path.clone()
        }
    );
    let converted = fs::read_to_string(&expected_old_path)?;
    assert_eq!(
        converted,
        "<!-- whitman: Converted from AGENTS.md -->\nlegacy agents"
    );
    let old_profile = parse_profile_file(&expected_old_path)?;
    assert_eq!(old_profile.name, "old");
    assert_eq!(old_profile.description, "Converted from AGENTS.md");
    assert_symlink_target(&agents_path, &profile.path)?;
    Ok(())
}

#[test]
fn confirms_before_overwriting_existing_old_profile() -> Result<()> {
    let fixture = Fixture::new()?;
    let profile = fixture.write_profile("work", "Work")?;
    let agents_path = fixture.work_dir.path().join(AGENTS_FILE_NAME);
    fs::write(&agents_path, "legacy agents")?;
    let old_path = old_profile_path(fixture.agents_dir.path());
    fs::write(&old_path, "previous old")?;
    let mut confirmer = |_message: &str| Ok(false);

    let error = apply_profile(
        fixture.work_dir.path(),
        fixture.agents_dir.path(),
        &profile,
        &mut confirmer,
    )
    .unwrap_err()
    .to_string();

    assert!(error.contains("cancelled"));
    assert_eq!(fs::read_to_string(&agents_path)?, "legacy agents");
    assert_eq!(fs::read_to_string(&old_path)?, "previous old");
    Ok(())
}

#[test]
fn overwrites_existing_old_profile_after_confirmation() -> Result<()> {
    let fixture = Fixture::new()?;
    let profile = fixture.write_profile("work", "Work")?;
    let agents_path = fixture.work_dir.path().join(AGENTS_FILE_NAME);
    fs::write(&agents_path, "legacy agents")?;
    let old_path = old_profile_path(fixture.agents_dir.path());
    fs::write(&old_path, "previous old")?;
    let mut confirmer = |_message: &str| Ok(true);

    if apply_or_skip_symlink_permission(
        fixture.work_dir.path(),
        fixture.agents_dir.path(),
        &profile,
        &mut confirmer,
    )?
    .is_none()
    {
        return Ok(());
    }

    assert_eq!(
        fs::read_to_string(&old_path)?,
        "<!-- whitman: Converted from AGENTS.md -->\nlegacy agents"
    );
    assert_symlink_target(&agents_path, &profile.path)?;
    Ok(())
}

#[test]
fn windows_symlink_guidance_explains_no_copy_fallback() {
    let error = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
    let message = windows_symlink_setup_instructions(&error);

    assert!(message.contains("Developer Mode"));
    assert!(message.contains("Administrator"));
    assert!(message.contains("does not copy"));
}

struct Fixture {
    work_dir: tempfile::TempDir,
    agents_dir: tempfile::TempDir,
}

impl Fixture {
    fn new() -> Result<Self> {
        Ok(Self {
            work_dir: tempdir()?,
            agents_dir: tempdir()?,
        })
    }

    fn write_profile(&self, name: &str, description: &str) -> Result<Profile> {
        let path = self.agents_dir.path().join(format!("AGENTS.{name}.md"));
        fs::write(&path, format!("{description}\nprofile body\n"))?;
        Ok(Profile {
            name: name.to_string(),
            description: description.to_string(),
            path,
        })
    }
}

fn assert_symlink_target(link: &Path, expected_target: &Path) -> Result<()> {
    let target = fs::read_link(link)?;
    assert_eq!(target, expected_target);
    Ok(())
}

#[cfg(unix)]
fn symlink_file(target: &Path, link: &Path) -> Result<bool> {
    unix_fs::symlink(target, link)?;
    Ok(true)
}

#[cfg(windows)]
fn symlink_file(target: &Path, link: &Path) -> Result<bool> {
    match windows_fs::symlink_file(target, link) {
        Ok(()) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => Ok(false),
        Err(error) => Err(error.into()),
    }
}

fn apply_or_skip_symlink_permission(
    work_dir: &Path,
    agents_dir: &Path,
    profile: &Profile,
    confirmer: &mut impl whitman::agents_file::Confirm,
) -> Result<Option<ApplyOutcome>> {
    match apply_profile(work_dir, agents_dir, profile, confirmer) {
        Ok(outcome) => Ok(Some(outcome)),
        Err(error) if is_symlink_permission_error(&error) => Ok(None),
        Err(error) => Err(error),
    }
}

#[cfg(windows)]
fn is_symlink_permission_error(error: &anyhow::Error) -> bool {
    format!("{error:#}").contains("Developer Mode")
}

#[cfg(not(windows))]
fn is_symlink_permission_error(_error: &anyhow::Error) -> bool {
    false
}
