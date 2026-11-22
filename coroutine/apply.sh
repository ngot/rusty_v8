#!/usr/bin/env bash
# Applies coroutine/coroutine-local.patch to the v8 submodule, once.
# Run from anywhere; a tree that already carries the patch is left alone.
set -euo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
patch=$root/coroutine/coroutine-local.patch
if git -C "$root/v8" apply --reverse --check "$patch" 2>/dev/null; then
  echo "coroutine-local.patch already applied"
  exit 0
fi
git -C "$root/v8" apply "$patch"
echo "applied coroutine-local.patch"
