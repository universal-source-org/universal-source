# Non-production JavaScript engine spike

This isolated, unpublished Rust crate probes QuickJS-NG primitives through rquickjs. It is **not Universal Source JavaScript execution**, a conformance suite, a hardened realm, a converter, or a selected engine dependency. The [evaluation](../../docs/reviews/2026-09-28-javascript-engine-evaluation.md) records exact revisions, candidate comparisons, results and limits. **No engine was selected.** RFC 0001 remains proposed.

Ten tests use fixed in-memory source text to exercise context isolation/reuse/destruction, Rust-to-JS calls, synchronous results, direct Promise observation, explicit single-job execution, host resolution, loop/regexp interruption, disposal, descriptors/accessors, Proxy inspection, twelve dynamic-constructor denial paths, and absent ambient APIs. Two passing tests deliberately reproduce old reactions/await continuations running after their synthetic invocation ended. Passing this suite therefore does not mean the RFC can be implemented by these APIs.

No source package, manifest, module graph, Source API operation, host service or production abstraction is loaded. Tests use full contexts with globals beyond the RFC allowlist so that the missing hardening remains visible. The local constructor bootstrap is an experiment, not a security boundary. The small unsafe descriptor helper is limited to known ordinary objects; it is not a general structural copier.

## Dependency and build scope

- Rust/Cargo 1.96+ (aligned with the existing runtime), a working native C toolchain.
- Exact `rquickjs = 0.14.0`, default features disabled, only `std`; committed independent Cargo.lock. This vendors QuickJS-NG 0.16.2 at `0fdea21ff1090084e91dad812b343d92e79ba9d9`.
- No loader, futures, dynamic-library, parallel or bindgen feature is enabled. On the tested macOS arm64 target the supplied bindings and `cc` compile/link the vendored C engine statically; no separate engine binary download occurs.
- Active external build dependencies: rquickjs/core/sys 0.14.0, hashbrown 0.17.1, allocator-api2 0.2.21, equivalent 1.0.2, foldhash 0.2.0, cc 1.5.1, find-msvc-tools 0.1.14, shlex 2.0.1. The lockfile also contains optional inactive dependencies selected by Cargo resolution; this is not a claim that all locked packages compile.
- `runtime/Cargo.toml` and `runtime/Cargo.lock` are untouched. This crate is not a workspace member or a runtime dependency. Its `target/` is ignored.

## Run from the repository root

The tests include intentional nontermination. Use an external process-group timeout as a backstop if interruption regresses; the test's intended success is an engine-controlled exception, not the timeout killing it. On the evaluated macOS/Linux-style host:

```sh
python3 - <<'PY'
import os
import signal
import subprocess

command = [
    'cargo', 'test', '--manifest-path',
    'experiments/javascript-engine-spike/Cargo.toml', '--locked',
    '--', '--test-threads=1', '--nocapture',
]
process = subprocess.Popen(command, start_new_session=True)
try:
    code = process.wait(timeout=60)
except subprocess.TimeoutExpired:
    os.killpg(process.pid, signal.SIGKILL)
    process.wait()
    raise SystemExit('FAIL: spike exceeded 60-second external backstop')
raise SystemExit(code)
PY
cargo fmt --manifest-path experiments/javascript-engine-spike/Cargo.toml --check
cargo clippy --manifest-path experiments/javascript-engine-spike/Cargo.toml --locked --all-targets -- -D warnings
```

The 60 seconds includes compilation; use `cargo test --manifest-path experiments/javascript-engine-spike/Cargo.toml --locked --no-run` first on a cold/slower build. The process-group wrapper itself is not a Windows portability claim.

Local result on macOS 27.0 arm64, Rust 1.96.0, Apple clang 21.0.0: **10 passed, 0 failed, 0 ignored; 0 doc tests**. Format and strict Clippy passed. Each runtime configures 16 MiB engine allocation and 256 KiB stack ceilings, without testing exhaustion or claiming exact portable/process quotas. Only this host was tested; see the evaluation for remaining platform and RFC blockers.
