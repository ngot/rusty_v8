# 🧬 rusty_v8 with coroutine-local V8

### ⚠️ TL;DR

Branch `coroutine` is upstream rusty_v8 at a release tag plus one V8 patch that lets stackful coroutines share one isolate, and nothing else.
A `v<version>-coroutine.<n>` tag builds, tests and publishes the patched archive for 4 targets.
An embedder does not build V8: rusty_v8's own `build.rs` downloads the archive from this repository's releases once `RUSTY_V8_MIRROR` and `RUSTY_V8_MIRROR_TAG` point here.
`main` stays a plain mirror of upstream.

## 🧩 What the branch adds

| Path                               | What                                                                                                                                                                        |
| ---------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `coroutine/coroutine-local.patch`  | The V8 change: coroutine-local `thread_local`s, interrupts kept across a switch, the CPU sampler's thread check, object shapes, trace events, eval and script cache entries |
| `coroutine/thread_locals.txt`      | Every V8 `thread_local`, marked `patched` or `ignored: <reason>`                                                                                                            |
| `coroutine/list-thread-locals.sh`  | Lists V8's current `thread_local`s                                                                                                                                          |
| `coroutine/check-thread-locals.sh` | Fails when the list and V8 disagree                                                                                                                                         |
| `coroutine/apply.sh`               | Applies the patch to the `v8` submodule, once                                                                                                                               |
| `coroutine/UPGRADE.md`             | Rebasing onto a new rusty_v8, repairing the patch, classifying a `thread_local`                                                                                             |
| `coroutine/LICENSE`                | The patch's license (BSD-3-Clause, as V8)                                                                                                                                   |
| `.gn`                              | `v8_typed_array_max_size_in_heap = 64` (upstream: 0), the reason beside it                                                                                                  |
| `tests/test_coroutine_patch.rs`    | Every export the patch adds links, and the archive reports the patch's version                                                                                              |
| `.github/workflows/coroutine.yml`  | Build, test and release; upstream's `ci.yml` is deleted on this branch so a tag does not start its matrix                                                                   |

## 🏷️ Releasing

| Step | What                                                                                                        |
| ---- | ----------------------------------------------------------------------------------------------------------- |
| 1️⃣   | Commit to `coroutine`; `coroutine.yml` builds and tests all 4 targets on every push                         |
| 2️⃣   | When the push is green, tag it `v<rusty_v8 version>-coroutine.<n+1>` and push the tag                       |
| 3️⃣   | The tag's run publishes the release only when all 4 targets passed                                          |
| 4️⃣   | Bump `RUSTY_V8_MIRROR_TAG` in [v8_coroutine_rust](https://github.com/ngot/v8_coroutine_rust) and release it |

| Target                      | Runner                | Tested on                                     |
| --------------------------- | --------------------- | --------------------------------------------- |
| `x86_64-unknown-linux-gnu`  | `ubuntu-22.04`        | the build runner                              |
| `aarch64-unknown-linux-gnu` | `ubuntu-22.04`, cross | `ubuntu-22.04-arm`, against the built archive |
| `aarch64-apple-darwin`      | `macos-15`            | the build runner                              |
| `x86_64-pc-windows-msvc`    | `windows-2022`        | the build runner                              |

Each target ships 4 assets under rusty_v8's own names, so its `build.rs` finds them under the tag:

| Asset                                                                              | What                                              |
| ---------------------------------------------------------------------------------- | ------------------------------------------------- |
| `librusty_v8_release_<target>.a.gz`, `rusty_v8_release_<target>.lib.gz` on Windows | The static library                                |
| `src_binding_release_<target>.rs`                                                  | The generated Rust binding                        |
| `licenses_<target>.tar.gz`                                                         | V8's license and every bundled third-party notice |
| `SHA256SUMS_<target>`                                                              | Checksums of the other 3                          |

A released tag is final: the release job refuses to replace one, so fix forward with `<n+1>`.

## 🛠️ Building locally

```sh
git submodule update --init --recursive --depth 1
bash coroutine/apply.sh
bash coroutine/check-thread-locals.sh
V8_FROM_SOURCE=1 cargo build --release
```

The archive is `target/release/gn_out/obj/librusty_v8.a`, and the binding is `target/release/gn_out/src_binding.rs`.
Point an embedder at them with `RUSTY_V8_ARCHIVE` and `RUSTY_V8_SRC_BINDING_PATH` to test a patch change before tagging it.

## ⚖️ Licenses

| Component   | License      | Use                                                                                       |
| ----------- | ------------ | ----------------------------------------------------------------------------------------- |
| ⚙️ V8       | BSD-3-Clause | The patch modifies V8 files and adds `src/base/coroutine-local.h`; see [LICENSE](LICENSE) |
| 📦 rusty_v8 | MIT          | Upstream, unchanged apart from `.gn`, the test and the workflow                           |
