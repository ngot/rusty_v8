#!/usr/bin/env bash
# Fails when V8 declares a thread_local that coroutine/thread_locals.txt does not classify,
# or when the list names one V8 no longer has. Reads the pinned tree (git HEAD of v8),
# so it works before or after coroutine/apply.sh.
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
listed=$(grep -v '^#' "$here/thread_locals.txt" | cut -f1 | sort -u)
current=$("$here/list-thread-locals.sh" "$here/../v8")
if ! diff <(echo "$listed") <(echo "$current"); then
  echo "thread_local drift: classify each '>' line in coroutine/thread_locals.txt" \
    "(and add a layer 2 test for each one marked patched); drop each '<' line." >&2
  exit 1
fi
echo "every V8 thread_local is classified"
