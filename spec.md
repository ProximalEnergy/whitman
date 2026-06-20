# Requirements
- Rust CLI utility named `whitman`
- Use Rust 1.96.0, latest stable as of 2026-06-19
- Manage symlink for `AGENTS.md`
- User chooses active profile
- Each profile has:
  - name under 15 characters
  - description under 100 characters
  - associated `agents.profile_name.md` file
- Profile location: `~/.whitman/profiles`
- Windows profile location: `%USERPROFILE%\.whitman\profiles`
- Profiles are global
- Profile metadata inferred from files
- Profile name inferred from `agents.profile_name.md`
- Profile description inferred from first line:
  - `<!-- whitman: description -->`
- Destination always `./AGENTS.md`
- CLI displays profile list
- CLI supports search input
- Search automatically filters available profiles
- CLI supports keyboard navigation:
  - `j` moves down
  - `k` moves up
- User confirms selected profile
- CLI creates or updates symlink to chosen profile
- CLI overwrites existing `AGENTS.md` with symlink
- If existing `AGENTS.md` is not a whitman profile:
  - convert it into profile file `agents.old.md`
  - then replace `AGENTS.md` with symlink
- If existing profile would be overwritten:
  - confirm with user first
- Existing symlink outside `~/.whitman/profiles`:
  - refuse
  - show error
  - no overwrite
- No non-interactive mode
- Windows supported
- Windows symlink behavior:
  - use file symlink
  - if permission unavailable, fail with setup instructions
  - do not copy as fallback
- Open source repo ready
- License: `Apache-2.0`
- Published to crates.io
- crates.io package name `whitman` available as of 2026-06-19
- README documents install, usage, profile config, safety behavior
- License included
- CI runs format, clippy, tests

# Infrastructure
- No backend infrastructure
- GitHub repository
- GitHub Actions for CI
- crates.io package publishing
- Optional release automation for tagged releases
- Cross-platform CI for Linux, macOS, Windows

# Libraries Used
- New libraries:
  - `ratatui` for terminal UI
  - `crossterm` for terminal events/backend
  - `clap` for CLI args
  - `anyhow` or `thiserror` for errors
  - `dirs` or `directories` for config paths
- Existing libraries:
  - Rust standard library filesystem APIs for symlink operations

# Unknowns
- None

# Discrepancies
- None found
