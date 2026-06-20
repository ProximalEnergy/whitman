#!/usr/bin/env bash
set -euo pipefail

cargo pkgid | sed 's/.*#//'
