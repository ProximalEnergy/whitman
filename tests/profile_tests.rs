use std::fs;
use std::path::Path;

use anyhow::Result;
use tempfile::tempdir;
use whitman::profile::{
    create_profile_file, filter_profiles, list_profiles, parse_description, parse_profile_file,
    profile_file_name, repository_agents_dir,
};

#[test]
fn infers_profile_metadata_from_file_name_and_first_line() -> Result<()> {
    let directory = tempdir()?;
    let path = write_profile(
        directory.path(),
        "AGENTS.work.md",
        "Work projects\ncontent\n",
    )?;

    let profile = parse_profile_file(&path)?;

    assert_eq!(profile.name, "work");
    assert_eq!(profile.description, "Work projects");
    assert_eq!(profile.path, path);
    Ok(())
}

#[test]
fn lists_profiles_sorted_by_name() -> Result<()> {
    let directory = tempdir()?;
    write_profile(directory.path(), "AGENTS.zeta.md", "Last\n")?;
    write_profile(directory.path(), "AGENTS.alpha.md", "First\n")?;
    write_profile(directory.path(), "notes.md", "<!-- whitman: Ignore -->\n")?;

    let profiles = list_profiles(directory.path())?;
    let names: Vec<_> = profiles
        .iter()
        .map(|profile| profile.name.as_str())
        .collect();

    assert_eq!(names, ["alpha", "zeta"]);
    Ok(())
}

#[test]
fn lists_profiles_from_repository_agents_dir() -> Result<()> {
    let repository = tempdir()?;
    let repository_agents = repository.path().join(".whitman").join("agents");
    fs::create_dir_all(&repository_agents)?;

    write_profile(&repository_agents, "AGENTS.repo.md", "Repository profile\n")?;

    let profiles = list_profiles(&repository_agents)?;
    let names: Vec<_> = profiles
        .iter()
        .map(|profile| profile.name.as_str())
        .collect();

    assert_eq!(names, ["repo"]);
    Ok(())
}

#[test]
fn creates_starter_profile_file() -> Result<()> {
    let directory = tempdir()?;

    let profile = create_profile_file(directory.path(), "work", "Work projects")?;

    assert_eq!(profile.name, "work");
    assert_eq!(profile.description, "Work projects");
    assert_eq!(profile.path, directory.path().join("AGENTS.work.md"));
    assert_eq!(
        fs::read_to_string(&profile.path)?,
        "Work projects\n\n# Instructions\n"
    );
    Ok(())
}

#[test]
fn repository_agents_dir_uses_nearest_git_ancestor() -> Result<()> {
    let repository = tempdir()?;
    fs::create_dir(repository.path().join(".git"))?;
    let nested = repository.path().join("packages").join("app");
    fs::create_dir_all(&nested)?;

    assert_eq!(
        repository_agents_dir(&nested),
        repository.path().join(".whitman").join("agents")
    );
    Ok(())
}

#[test]
fn rejects_invalid_profile_names() -> Result<()> {
    let directory = tempdir()?;
    let long_name = "abcdefghijklmnx";
    let path = write_profile(
        directory.path(),
        &format!("AGENTS.{long_name}.md"),
        "Too long\n",
    )?;

    let error = format!("{:#}", parse_profile_file(&path).unwrap_err());
    assert!(error.contains("under 15 characters"));

    let path = write_profile(directory.path(), "AGENTS.bad.name.md", "Bad chars\n")?;
    let error = format!("{:#}", parse_profile_file(&path).unwrap_err());
    assert!(error.contains("ASCII letters"));
    Ok(())
}

#[test]
fn rejects_missing_empty_and_long_descriptions() {
    let missing = parse_description("not a whitman comment")
        .unwrap_err()
        .to_string();
    assert!(missing.contains("first line"));

    let empty = parse_description("<!-- whitman: -->")
        .unwrap_err()
        .to_string();
    assert!(empty.contains("cannot be empty"));

    let long = format!("<!-- whitman: {} -->", "a".repeat(100));
    let error = parse_description(&long).unwrap_err().to_string();
    assert!(error.contains("under 100 characters"));
}

#[test]
fn filters_profiles_by_name_or_description_case_insensitively() -> Result<()> {
    let directory = tempdir()?;
    write_profile(directory.path(), "AGENTS.work.md", "Client projects\n")?;
    write_profile(directory.path(), "AGENTS.home.md", "Personal tasks\n")?;
    let profiles = list_profiles(directory.path())?;

    let by_name = filter_profiles(&profiles, "HOME");
    assert_eq!(by_name[0].name, "home");

    let by_description = filter_profiles(&profiles, "client");
    assert_eq!(by_description[0].name, "work");

    let all = filter_profiles(&profiles, " ");
    assert_eq!(all.len(), 2);
    Ok(())
}

#[test]
fn formats_profile_file_names() -> Result<()> {
    assert_eq!(profile_file_name("client_1")?, "AGENTS.client_1.md");
    assert!(profile_file_name("bad.name").is_err());
    Ok(())
}

fn write_profile(directory: &Path, file_name: &str, contents: &str) -> Result<std::path::PathBuf> {
    let path = directory.join(file_name);
    fs::write(&path, contents)?;
    Ok(path)
}
