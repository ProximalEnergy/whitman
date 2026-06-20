# Whitman Plan

Existing: standalone Rust crate at repository root.
Unknowns: none.

## Task 1: Pin Rust toolchain
### Status: Done
Add Rust 1.96.0 toolchain pin.

**File touched:**
- `rust-toolchain.toml`

## Task 2: Define crate manifest
### Status: Done
Create `whitman` package metadata, Apache-2.0 license, bin target,
dependencies: `ratatui`, `crossterm`, `clap`, `anyhow` or `thiserror`,
`directories`, test deps.

**File touched:**
- `Cargo.toml`

## Task 3: Lock dependencies
### Status: Done
Generate lockfile from manifest.

**File touched:**
- `Cargo.lock`

## Task 4: CLI entrypoint
### Status: Done
Wire clap command, no non-interactive mode, load profiles, run TUI, confirm,
apply selected profile.

**File touched:**
- `src/main.rs`

## Task 5: Profile metadata
### Status: Done
List `~/.whitman/profiles`, parse `agents.<name>.md`, validate name and
description limits, infer description from first whitman comment line.

**File touched:**
- `src/profile.rs`

## Task 6: AGENTS safety
### Status: Done
Handle `./AGENTS.md`: detect whitman symlink, refuse external symlink,
convert non-whitman file to `agents.old.md`, confirm before overwrite.

**File touched:**
- `src/agents_file.rs`

## Task 7: Cross-platform symlink
### Status: Done
Create/update file symlink on Unix and Windows. On Windows permission failure,
show setup instructions, no copy fallback.

**File touched:**
- `src/platform.rs`

## Task 8: Terminal UI
### Status: Done
Build ratatui/crossterm profile picker: list, search filter, `j`/`k`
navigation, confirmation screen.

**File touched:**
- `src/tui.rs`

## Task 9: Profile tests
### Status: Done
Test metadata inference, invalid names, missing/long descriptions, profile
search matching.

**File touched:**
- `tests/profile_tests.rs`

## Task 10: AGENTS safety tests
### Status: Done
Test symlink refusal, conversion to `agents.old.md`, overwrite confirmation
paths, Windows symlink error behavior where portable.

**File touched:**
- `tests/agents_file_tests.rs`

## Task 11: README
### Status: Done
Document install, usage, profile config, search/navigation, safety behavior,
Windows symlink setup, crates.io publishing.

**File touched:**
- `README.md`

## Task 12: License
### Status: Done
Add Apache-2.0 license text.

**File touched:**
- `LICENSE`

## Task 13: CI
### Status: Done
Add cross-platform GitHub Actions for format, clippy, tests on Linux, macOS,
Windows.

**File touched:**
- `.github/workflows/whitman-pr-checks.yml`

## Task 14: crates.io publication
### Status: Pending User Confirmation
Publish the `whitman` crate to crates.io. Local readiness is verified with
`cargo publish --dry-run --allow-dirty`; actual upload is externally visible and
requires explicit confirmation.

**File touched:**
- None

## Unresolved Questions
- Confirm whether to publish `whitman` v0.1.0 to crates.io now.

## Verification
- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-targets --all-features`
- PTY CLI smoke tests for default selection, search filtering, `j` navigation,
  and conversion of an existing `AGENTS.md` to a valid `agents.old.md` profile
  with preserved original content
- `cargo package --allow-dirty`
- `cargo publish --dry-run --allow-dirty`
- `cargo search whitman --limit 10` returned no matching package output
