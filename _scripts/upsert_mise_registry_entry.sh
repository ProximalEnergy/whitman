#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 /path/to/mise/registry.toml" >&2
  exit 1
fi

registry_file="$1"
entry_file="$(mktemp)"

cleanup() {
  rm -f "$entry_file" "${registry_file}.tmp"
}
trap cleanup EXIT

cat >"$entry_file" <<'REGISTRY'
[tools.whitman]
backends = ["cargo:whitman"]
description = "Interactive profile picker for project AGENTS.md files"
test = ["whitman --version", "whitman {{version}}"]
REGISTRY

if [[ ! -f "$registry_file" ]]; then
  echo "missing registry file: $registry_file" >&2
  exit 1
fi

if rg -q '^\[tools\.whitman\]' "$registry_file"; then
  awk -v entry_file="$entry_file" '
    BEGIN { skip = 0; inserted = 0 }
    /^\[tools\.whitman\]/ {
      while ((getline line < entry_file) > 0) print line
      close(entry_file)
      inserted = 1
      skip = 1
      next
    }
    /^\[tools\./ && skip {
      skip = 0
    }
    !skip { print }
    END {
      if (!inserted) {
        print ""
        while ((getline line < entry_file) > 0) print line
        close(entry_file)
      }
    }
  ' "$registry_file" >"${registry_file}.tmp"
else
  awk -v entry_file="$entry_file" '
    function print_entry() {
      print ""
      while ((getline line < entry_file) > 0) print line
      close(entry_file)
      inserted = 1
    }
    BEGIN { inserted = 0 }
    /^\[tools\./ {
      name = $0
      sub(/^\[tools\./, "", name)
      sub(/\].*$/, "", name)
      if (!inserted && name > "whitman") print_entry()
    }
    { print }
    END {
      if (!inserted) print_entry()
    }
  ' "$registry_file" >"${registry_file}.tmp"
fi

mv "${registry_file}.tmp" "$registry_file"
