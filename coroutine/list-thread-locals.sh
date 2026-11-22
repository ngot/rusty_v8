#!/usr/bin/env bash
# Prints every `thread_local` declaration in V8 src/ at the checked-out commit (ignoring
# working-tree edits such as the applied patch), one "<file>: <declaration>" per line.
# Usage: list-thread-locals.sh <path to rusty_v8>/v8
set -euo pipefail
git -C "$1" grep -n -w thread_local HEAD -- src |
  sed -E 's/^HEAD:([^:]+):[0-9]+:/\1: /; s|//.*$||; s/[[:space:]]+/ /g; s/ $//' |
  grep -w thread_local |
  sort -u
