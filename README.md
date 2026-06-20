# whitman

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
description, use `j`/`k` or arrow keys to move, press Enter to select a profile,
then confirm the selection.

If `.whitman/agents` has no profiles yet, `whitman` offers to create the first
one by prompting for a profile name and description. It writes
`AGENTS.<name>.md` in `.whitman/agents`, then continues to the profile picker.

`whitman` intentionally has no non-interactive apply mode. The active profile is
always chosen by a person in the terminal UI.

## Profile Files

Profiles are Markdown files named `AGENTS.<name>.md` under `.whitman/agents`.
The profile name is inferred from the file name.

Example:

```md
Default coding-agent instructions

# Instructions

...
```

Profile rules:

- File name format: `AGENTS.<name>.md`
- Name length: under 15 characters
- Name characters: ASCII letters, numbers, underscores, and hyphens
- Description source: first line, either plain text or formatted as
  `<!-- whitman: description -->`
- Description length: under 100 characters

## Safety Behavior

`whitman` always writes the destination `./AGENTS.md` in the current directory.

When `./AGENTS.md` does not exist, `whitman` creates a file symlink to the
selected profile.

When `./AGENTS.md` is already a symlink into `.whitman/agents`, `whitman`
updates the symlink to point at the selected profile.

When `./AGENTS.md` is a regular file, `whitman` converts it into
`.whitman/agents/AGENTS.old.md`, adding a Whitman description comment before the
original content so the converted file remains a valid profile. It then replaces
`./AGENTS.md` with the selected-profile symlink. If `AGENTS.old.md` already
exists, `whitman` asks before overwriting it.

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
