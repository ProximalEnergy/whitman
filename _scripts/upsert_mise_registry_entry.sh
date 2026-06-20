#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 /path/to/mise-repo" >&2
  exit 1
fi

mise_repo="$1"
registry_dir="$mise_repo/registry"
registry_file="$registry_dir/whitman.toml"

if [[ ! -d "$registry_dir" ]]; then
  echo "missing registry directory: $registry_dir" >&2
  exit 1
fi

cat >"$registry_file" <<'REGISTRY'
backends = ["cargo:whitman"]
description = "Interactive profile picker for project AGENTS.md files"
test = { cmd = "whitman --version", expected = "whitman {{version}}" }
REGISTRY
