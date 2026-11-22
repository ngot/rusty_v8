# 🔁 Upgrading V8 (rusty_v8 bump)

### ⚠️ TL;DR

Every rusty_v8 bump (about every 4 weeks) needs: rebase this branch, re-apply the patch, **re-classify every new `thread_local`**, and a green `coroutine.yml` run on all 4 targets.
The dangerous step is classification: a variable wrongly marked `ignored` corrupts memory silently, and the first audit of V8 15.2 got one wrong (`thread_reserved_stack_limit`).
Classify by **readers**, not by declarations.
Paths below are relative to the root of this repository; the bindings and the coroutine tests live in [ngot/v8_coroutine_rust](https://github.com/ngot/v8_coroutine_rust), under `v8-coroutine-sys/` and `v8-coroutine/`.

## 🧭 Steps

| #   | Step                                                                             | Command / file                                                                                                    |
| --- | -------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------- |
| 1️⃣  | Rebase onto the new tag                                                          | `git rebase --onto v<new> v<old> coroutine`; resolve `.gn` and the deleted `ci.yml` by keeping this branch's side |
| 2️⃣  | Apply the patch and run the drift guard                                          | `git submodule update --init --recursive`, `bash coroutine/apply.sh`, `bash coroutine/check-thread-locals.sh`     |
| 3️⃣  | If the patch fails to apply: repair it                                           | See "Repairing the patch"                                                                                         |
| 4️⃣  | If the drift guard fails: classify                                               | See "Classifying a thread_local"                                                                                  |
| 5️⃣  | Re-check the readers of every `patched` and every reader-sensitive `ignored` row | See "Reader checks"                                                                                               |
| 6️⃣  | Build and test until green                                                       | `V8_FROM_SOURCE=1 cargo nextest run --release`, then push and let `coroutine.yml` run all 4 targets               |
| 7️⃣  | Run the publishing checks                                                        | See "Before publishing"                                                                                           |

`coroutine.yml` order: apply the patch, drift guard, full build, rusty_v8's suite plus `tests/test_coroutine_patch.rs`.
It fails fast on drift, before the long build.
`coroutine/thread_locals.txt` is the record of every classification; keep each reason specific enough that the next bump can re-check it.

## 🩹 Repairing the patch

The patch touches 19 files in V8's `src/` and adds `src/base/coroutine-local.h`.
The `src/execution/v8threads.*` and `src/profiler/cpu-profiler.cc` hunks are not `thread_local`s either: `ThreadManager` records which V8 thread identity has its state restored under the lock, and the CPU sampler checks that identity instead of the one that started the profiler, so samples from every coroutine count; `v8-coroutine/tests/cpu_profile.rs` fails when they do not.
The `src/objects/compilation-cache-table.cc` and `src/codegen/compilation-cache.cc` hunks are not `thread_local`s either: they restore the eval cache's hash placeholder, which V8 13.7 dropped, so a source evaluated once is not kept until a memory-pressure GC, and the script cache, with `src/objects/compilation-cache-table-inl.h`, keeps a script's top-level function only from a second compile while its Script is alive, so a burst of distinct scripts does not fill the heap before their bytecode is old enough to flush; `v8-coroutine-sys/tests/eval_cache.rs` and `v8-coroutine-sys/tests/script_cache.rs` fail when they no longer do.
The `src/execution/stack-guard.cc` hunks are not `thread_local`s: they keep pending interrupt flags across `ArchiveStackGuard`, `RestoreStackGuard` and `InitThread`, and `v8-coroutine/tests/interrupts.rs` fails when they no longer do.

| Symptom                                                                                        | Fix                                                                                                                                                                                                     |
| ---------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| ❌ A hunk is rejected                                                                          | Re-apply by hand in `<rusty_v8>/v8`: replace the `thread_local` declaration with `constinit v8_coroutine::CoroutineLocal<T> name(v8_coroutine::k...)`, keep the include of `src/base/coroutine-local.h` |
| ❌ "no viable conversion" from `v8_coroutine::CoroutineLocal<T>`                               | A use site needs two user-defined conversions (for example to `StackSlot`); call `.Get()` there                                                                                                         |
| ❌ Member access or a reference through the variable (`x.contains_all(...)`, `&x`)             | Use `x.Get()` to read and `x.Set(v)` or `x = v` to write; `CoroutineLocal` never hands out a reference                                                                                                  |
| ❌ `static_assert(kInitialValue.ToIntegral() == v8_coroutine::kInitialPerThreadAsserts)` fails | `PerThreadAssertType` changed; recompute the bits and update `kInitialPerThreadAsserts` in `coroutine-local.h`                                                                                          |

Regenerate the patch from the working copy (the header must be tracked as intent-to-add so it appears in the diff):

```sh
cd <rusty_v8>/v8
git add -N src/base/coroutine-local.h
git diff > ../coroutine/coroutine-local.patch
```

Compile only the touched files first to save a full build per mistake (paths from `ninja -t targets all`):

```sh
cd target/release/gn_out
DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer ../ninja_gn_binaries/ninja/ninja \
  obj/v8/v8_libbase/platform.o obj/v8/v8_base_without_compiler/{isolate,thread-id,local-heap,assert-scope}.o
```

## 🔎 Classifying a `thread_local`

`coroutine/check-thread-locals.sh` prints new declarations as `>` lines.
Add each to `coroutine/thread_locals.txt` as `<file>: <declaration><TAB>patched: <why>` or `<TAB>ignored: <why>`.

A coroutine only switches inside `v8_coroutine::suspend()`, after `Isolate::Exit` and `v8::Unlocker`, and resumes before `Locker` and `Isolate::Enter`.
Patch a variable if its value can differ between two coroutines at that point.

| Question                                                                                              | Patch                               | Ignore |
| ----------------------------------------------------------------------------------------------------- | ----------------------------------- | ------ |
| 🪪 Keys V8's per-thread state (Locker owner, archived `ThreadLocalTop`, handle scopes, stack guard)   | ✅                                  |        |
| 📏 Describes the current stack (start, limit, bounds)                                                 | ✅                                  |        |
| 🧷 Read in release builds by a `CHECK`, and written by scopes that can stay open across a suspend     | ✅                                  |        |
| 🔁 Rewritten by `Isolate::Enter` / `Exit` (`SetIsolateThreadLocals`) on every switch                  | Only if in the minimum identity set | ✅     |
| 🏗️ Set and cleared inside one compile job, GC step, or layout change with no call into JS             |                                     | ✅     |
| 🚫 Not compiled into this build (DEBUG-only, slow DCHECKs, sandbox, multi-cage, Windows, d8, testing) |                                     | ✅     |

For a new `patched` variable:

1. Add an entry to `enum CoroutineSlot` in `coroutine-local.h` and, if it does not start at zero, its initial value to `v8_coroutine_slots_init` and to `thread_table` (both in `platform.cc`).
2. Convert the declaration (and any `extern` in a header) to `v8_coroutine::CoroutineLocal<T>`.
3. Bump `V8_COROUTINE_PATCH_VERSION` in `coroutine-local.h` **and** `PATCH_VERSION` in `v8-coroutine-sys/lib.rs` together; `tests/test_coroutine_patch.rs` reads the patch's value itself.
4. Add a layer 2 test in `v8-coroutine/tests/per_variable.rs` that shows the variable stays separate per coroutine through behaviour V8 exposes.
5. Check that the negative control still fails every scenario.

⚠️ Slots are one `uintptr_t` each and narrower values sit in the low bytes, so only little-endian targets are supported; `T` must be trivially copyable.

## 🔬 Reader checks

The first audit marked `thread_reserved_stack_limit` "no callers" from a grep that missed `isolate.cc`; in V8 15.2 every GC reads it through `IsOnCentralStack()`.
Re-run reader searches on every bump, for the accessors as well as the variables:

```sh
cd v8
rg -n 'GetStackStart|GetReservedStackLimit|ObtainCurrentThreadStack' src
rg -n 'IsOnCentralStack|SetStackStart|stack_guard\(\)->InitThread' src
rg -n 'SetIsolateThreadLocals' -A25 src/execution/isolate.cc   # what Enter/Exit rewrites
```

Re-read the `ignored` reasons that depend on V8 behaviour rather than on build flags: rows saying "rewritten by Isolate::Enter", "never calls into JS", or "cleared on every call out of Wasm".

## 🍎 Build gotchas (macOS)

| Symptom                                                                                          | Fix                                                                                                                                                                                         |
| ------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `xcodebuild requires Xcode, but active developer directory ... is a command line tools instance` | Install Xcode and export `DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer`                                                                                                         |
| `ld64.lld: could not load TAPI file ... libSystem.tbd: unknown target`                           | The SDK is newer than Chromium's bundled lld; symlink an older SDK (for example `MacOSX26.5.sdk`) into `gn_out/sdk/xcode_links/` and pass `mac_sdk_path="//<that link>"` in `EXTRA_GN_ARGS` |

## ✅ Before publishing

Run v8_coroutine_rust's checks with the new archive (a draft tag, or `RUSTY_V8_ARCHIVE` and `RUSTY_V8_SRC_BINDING_PATH` pointed at this build).

| Check          | Command                                                                         | Pass                                         |
| -------------- | ------------------------------------------------------------------------------- | -------------------------------------------- |
| 🧪 Patch suite | `coroutine.yml` on this branch                                                  | All 4 targets green                          |
| 🧹 Lint        | `cargo clippy -p v8-coroutine-sys -p v8-coroutine --all-targets -- -D warnings` | No warnings                                  |
| 📊 Cost        | `cargo test --release -p v8-coroutine --test measure -- --ignored --nocapture`  | Within the README "Cost" table, or update it |
| 🌐 Embedders   | The end-to-end tests of whatever embeds this crate                              | Green on the new archive                     |

📝 Publishing: see [README.md](README.md#-releasing).
