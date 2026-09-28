# Non-production QuickJS module capture spike

**Outcome A — pre-evaluation capture viable.** A private host-native dependency callback captures linked entry functions before any package module body executes. This uses pinned public APIs, preserves package source text and selects no engine. The [review](../../docs/reviews/2026-09-28-quickjs-module-capture-spike.md) explains the phase distinction, syntax-analysis requirement, negative controls and limits.

This independent unpublished crate contains 14 focused tests, no production loader, manifest integration, dispatcher, parser, services or JavaScript conformance implementation. Prior lifecycle experiments remain unchanged. A separate crate keeps the added `loader` feature out of their original configuration.

## Strategy and boundaries

The host compiles the fixed entry/dependencies, declares a native capture module, then compiles a private root containing two ordered imports: capture module first, entry second. Evaluating this root links the entire graph, invokes the native callback, then evaluates the unmodified package graph. The callback reads actual linked namespace values into Rust-held Function references; it does not call source functions. It is removed from private runtime userdata before returning. The root has no executable statements beyond imports. No source-visible initialization export or callback is introduced.

There is no standalone public link-only call: `JS_ResolveModule` resolves dependencies, and `Module::eval` combines linking/evaluation. The native hook provides the required host intervention point inside that call. Only the host root/native module has started evaluation when capture runs; **no entry/helper module has started evaluation or run its body**. Compilation alone is not enough to safely inspect a namespace in this pinned engine. Tests never probe unlinked namespace access or use private structs/raw-pointer tricks.

The small helper checks export names and obtains functions. It deliberately does **not** validate direct-declaration syntax: generators, aliases and re-exports can pass that check. Those passing negative controls are evidence that pre-execution source syntax analysis is required, not permission to accept the forms. No parser is selected or added. Host-known operation names stand in for validated manifest declarations; source functions are always obtained from the engine, never fabricated from fixture knowledge.

The fixture resolver routes only fixed local text and denies package imports of private host names. It is not a containment/snapshot implementation. The `mark` global is solely a host-observable test counter, not proposed source authority. Full contexts are intentionally unhardened. Fresh Runtime/Context retirement and absence of job pumping reuse the established lifecycle approach; there is no async initialization protocol.

## Pinned dependencies and local evidence

- rquickjs/core/sys **0.14.0**, binding revision `d7ef5eeae702fea24c03643064de454f1c1dd4b0`.
- Vendored QuickJS-NG **0.16.2**, revision `0fdea21ff1090084e91dad812b343d92e79ba9d9`; no engine changes or private API.
- Exact dependency, default features disabled, `std` and `loader` only. `loader` activates **relative-path 2.0.1**, already in the prior lockfile. Independent lockfile differs from the lifecycle lockfile only in root package name; no external package version changed. Production Cargo files are untouched.
- Same Rust/Cargo 1.96 baseline and native C build. Tested on macOS 27.0 build 26A428 arm64, target `aarch64-apple-darwin`, rustc 1.96.0 (`ac68faa20`, 2026-05-25), Cargo 1.96.0 (`30a34c682`, 2026-05-25), Apple clang 21.0.0. No cross-platform claim.
- Probe limits are 16 MiB engine allocations and 256 KiB stack, not portable heap guarantees. No performance or resource-policy study.

## Reproduce

From the repository root, compile first, then guard execution with the same external process-group backstop as the lifecycle spike:

```sh
cargo test --manifest-path experiments/quickjs-module-capture-spike/Cargo.toml --locked --no-run
python3 - <<'PY'
import os
import signal
import subprocess

process = subprocess.Popen([
    'cargo', 'test', '--manifest-path',
    'experiments/quickjs-module-capture-spike/Cargo.toml', '--locked',
    '--', '--test-threads=1', '--nocapture',
], start_new_session=True)
try:
    code = process.wait(timeout=60)
except subprocess.TimeoutExpired:
    os.killpg(process.pid, signal.SIGKILL)
    process.wait()
    raise SystemExit('FAIL: module capture spike exceeded 60-second external backstop')
raise SystemExit(code)
PY
cargo fmt --manifest-path experiments/quickjs-module-capture-spike/Cargo.toml --check
cargo clippy --manifest-path experiments/quickjs-module-capture-spike/Cargo.toml --locked --all-targets -- -D warnings
```

Result: **14 passed, 0 failed, 0 ignored; 0 doc tests**. Strict Clippy and formatting pass. No external timeout fired. See the [completed plan](../../docs/plans/completed/2026-09-28-quickjs-module-capture-spike.md) for exact baseline and prior-experiment validation. Next is a narrow source-static-analysis compatibility proof, not production execution or engine selection.
