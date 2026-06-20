# Whitman Plan

Existing: `_cli-tools/whitman` stub README only. New Rust crate there.
Unknowns: none.

## Task 1: Pin Rust toolchain
### Status: To Do
Add Rust 1.96.0 toolchain pin.

**File touched:**
- `_cli-tools/whitman/rust-toolchain.toml`

## Task 2: Define crate manifest
### Status: To Do
Create `whitman` package metadata, Apache-2.0 license, bin target,
dependencies: `ratatui`, `crossterm`, `clap`, `anyhow` or `thiserror`,
`directories`, test deps.

**File touched:**
- `_cli-tools/whitman/Cargo.toml`

## Task 3: Lock dependencies
### Status: To Do
Generate lockfile from manifest.

**File touched:**
- `_cli-tools/whitman/Cargo.lock`

## Task 4: CLI entrypoint
### Status: To Do
Wire clap command, no non-interactive mode, load profiles, run TUI, confirm,
apply selected profile.

**File touched:**
- `_cli-tools/whitman/src/main.rs`

## Task 5: Profile metadata
### Status: To Do
List `~/.whitman/profiles`, parse `agents.<name>.md`, validate name and
description limits, infer description from first whitman comment line.

**File touched:**
- `_cli-tools/whitman/src/profile.rs`

## Task 6: AGENTS safety
### Status: To Do
Handle `./AGENTS.md`: detect whitman symlink, refuse external symlink,
convert non-whitman file to `agents.old.md`, confirm before overwrite.

**File touched:**
- `_cli-tools/whitman/src/agents_file.rs`

## Task 7: Cross-platform symlink
### Status: To Do
Create/update file symlink on Unix and Windows. On Windows permission failure,
show setup instructions, no copy fallback.

**File touched:**
- `_cli-tools/whitman/src/platform.rs`

## Task 8: Terminal UI
### Status: To Do
Build ratatui/crossterm profile picker: list, search filter, `j`/`k`
navigation, confirmation screen.

**File touched:**
- `_cli-tools/whitman/src/tui.rs`

## Task 9: Profile tests
### Status: To Do
Test metadata inference, invalid names, missing/long descriptions, profile
search matching.

**File touched:**
- `_cli-tools/whitman/tests/profile_tests.rs`

## Task 10: AGENTS safety tests
### Status: To Do
Test symlink refusal, conversion to `agents.old.md`, overwrite confirmation
paths, Windows symlink error behavior where portable.

**File touched:**
- `_cli-tools/whitman/tests/agents_file_tests.rs`

## Task 11: README
### Status: To Do
Document install, usage, profile config, search/navigation, safety behavior,
Windows symlink setup, crates.io publishing.

**File touched:**
- `_cli-tools/whitman/README.md`

## Task 12: License
### Status: To Do
Add Apache-2.0 license text.

**File touched:**
- `_cli-tools/whitman/LICENSE`

## Task 13: CI
### Status: To Do
Add cross-platform GitHub Actions for format, clippy, tests on Linux, macOS,
Windows.

**File touched:**
- `.github/workflows/whitman-pr-checks.yml`

## Unresolved Questions
- None
