#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 3 ]]; then
  echo "usage: $0 <version> <sha256> <formula-path>" >&2
  exit 1
fi

version="$1"
sha256="$2"
formula_path="$3"

mkdir -p "$(dirname "$formula_path")"

cat >"$formula_path" <<FORMULA
class Whitman < Formula
  desc "Interactive profile picker for project AGENTS.md files"
  homepage "https://github.com/ProximalEnergy/whitman"
  url "https://github.com/ProximalEnergy/whitman/archive/refs/tags/v${version}.tar.gz"
  sha256 "${sha256}"
  license "Apache-2.0"
  head "https://github.com/ProximalEnergy/whitman.git", branch: "main"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/whitman --version")
  end
end
FORMULA
