use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn help_documents_interactive_profile_picker() {
    let mut command = Command::cargo_bin("whitman").expect("binary exists");

    command
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Choose a global profile and link it to ./AGENTS.md",
        ));
}

#[test]
fn version_uses_whitman_package_name() {
    let mut command = Command::cargo_bin("whitman").expect("binary exists");

    command
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("whitman "));
}
