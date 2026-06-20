# whitman

"""
Do I contradict myself?
Very well then I contradict myself,
(I am large, I contain multitudes.)
"""

`whitman` is an interactive Rust CLI for choosing a repository AGENTS.md profile
and linking it into the current project as `./AGENTS.md`.



Profiles live in `.whitman/agents` where Whitman is initialized. When run from a
subdirectory, Whitman uses the nearest ancestor containing `.whitman`; otherwise
it uses the nearest Git repository root.

## Install

From crates.io:

```sh
cargo install whitman
```

From this repository:

```sh
cargo install --path .
```

The crate is pinned to Rust 1.96.0 with `rust-toolchain.toml`.

## Usage

Run `whitman` from the project directory that should receive `AGENTS.md`:

```sh
whitman
```

The terminal UI lists available profiles. Type to search by profile name or
description, use arrow keys to move, press Enter to select a profile, then
confirm the selection. Press `+` to create a new profile from inside the UI.

When creating a profile, `whitman` prompts for a profile name and description in
the terminal UI. It writes `AGENTS.<name>.md` and updates
`.whitman/agents/descriptions.toml`, then selects the new profile.

`whitman` intentionally has no non-interactive apply mode. The active profile is
always chosen by a person in the terminal UI.

## Profile Files

Profiles are Markdown files named `AGENTS.<name>.md` under `.whitman/agents`.
The profile name is inferred from the file name, and the profile description is
stored in `.whitman/agents/descriptions.toml`.

Example:

`.whitman/agents/descriptions.toml`

```toml
default = "Default coding-agent instructions"
```

`.whitman/agents/AGENTS.default.md`

```md
# Instructions

...
```

Profile rules:

- File name format: `AGENTS.<name>.md`
- Description file format: `.whitman/agents/descriptions.toml`
- Description entry format: `<name> = "<description>"`
- Name length: under 15 characters
- Name characters: ASCII letters, numbers, underscores, and hyphens
- Description source: the value for the profile name in the description file
- Description length: under 100 characters

## Safety Behavior

`whitman` always writes the destination `./AGENTS.md` in the current directory.

When `./AGENTS.md` does not exist, `whitman` creates a file symlink to the
selected profile.

When `./AGENTS.md` is already a symlink into `.whitman/agents`, `whitman`
updates the symlink to point at the selected profile.

When `./AGENTS.md` is a regular file, `whitman` converts it into
`.whitman/agents/AGENTS.old.md` and adds `old = "Converted from AGENTS.md"` to
`.whitman/agents/descriptions.toml`. It then replaces `./AGENTS.md` with the
selected-profile symlink. If the old profile already exists, `whitman` asks
before overwriting it.

When `./AGENTS.md` is a symlink outside `.whitman/agents`, `whitman` refuses to
overwrite it and prints an error.

## Windows Symlinks

On Windows, `whitman` uses `std::os::windows::fs::symlink_file` to create a file
symlink. If Windows denies symlink creation, enable Developer Mode in
Settings > System > For developers, or run `whitman` from an Administrator
terminal.

`whitman` does not copy profile files as a fallback.

## Development

```sh
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

CI runs format, clippy, and tests on Linux, macOS, and Windows.

## Publishing

The crates.io package name is `whitman`. Before publishing:

```sh
cargo package
cargo publish
```

## License

Apache-2.0. See `LICENSE`.
