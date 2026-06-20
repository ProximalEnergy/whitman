use std::fs;
use std::path::Path;

use anyhow::Result;
use tempfile::tempdir;
use whitman::profile::{
    filter_profiles, list_profiles, parse_description, parse_profile_file, profile_file_name,
};

#[test]
fn infers_profile_metadata_from_file_name_and_first_line() -> Result<()> {
    let directory = tempdir()?;
    let path = write_profile(
        directory.path(),
        "agents.work.md",
        "<!-- whitman: Work projects -->\ncontent\n",
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
    write_profile(
        directory.path(),
        "agents.zeta.md",
        "<!-- whitman: Last -->\n",
    )?;
    write_profile(
        directory.path(),
        "agents.alpha.md",
        "<!-- whitman: First -->\n",
    )?;
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
fn rejects_invalid_profile_names() -> Result<()> {
    let directory = tempdir()?;
    let long_name = "abcdefghijklmnx";
    let path = write_profile(
        directory.path(),
        &format!("agents.{long_name}.md"),
        "<!-- whitman: Too long -->\n",
    )?;

    let error = format!("{:#}", parse_profile_file(&path).unwrap_err());
    assert!(error.contains("under 15 characters"));

    let path = write_profile(
        directory.path(),
        "agents.bad.name.md",
        "<!-- whitman: Bad chars -->\n",
    )?;
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
    write_profile(
        directory.path(),
        "agents.work.md",
        "<!-- whitman: Client projects -->\n",
    )?;
    write_profile(
        directory.path(),
        "agents.home.md",
        "<!-- whitman: Personal tasks -->\n",
    )?;
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
    assert_eq!(profile_file_name("client_1")?, "agents.client_1.md");
    assert!(profile_file_name("bad.name").is_err());
    Ok(())
}

fn write_profile(directory: &Path, file_name: &str, contents: &str) -> Result<std::path::PathBuf> {
    let path = directory.join(file_name);
    fs::write(&path, contents)?;
    Ok(path)
}
