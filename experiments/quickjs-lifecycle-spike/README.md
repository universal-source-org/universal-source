# Non-production QuickJS Model B lifecycle spike

**Outcome A — lifecycle viable for the demonstrated Model B requirements through public APIs.** This selects no engine and accepts no RFC. The [review](../../docs/reviews/2026-09-28-quickjs-lifecycle-spike.md) maps every claim to evidence and records remaining boundaries. This crate implements no production loader, source dispatcher, service API or conformance corpus.

Each executing probe owns a separate rquickjs Runtime/Context pair. It stops single-job pumping at direct Promise settlement, copies a bounded structural candidate, releases every JavaScript root and drops context/runtime without executing remaining jobs. Native completion events carry only a generation and owned scalar; resolvers stay on the owner thread and are removed on revocation. Native drop counters and WeakRuntime observe retirement. Deliberate negative controls show why dropping only Context, sharing a runtime, or convenient property access are insufficient.

The 15 unit tests cover global/intrinsic/module/cache reset; roots and unrelated-runtime rejection; pending/chained/finally/nested-await work; queued and dormant graph destruction; species resolvers; stale completions; CPU cancellation/deadline; resource latching; setup/module failure and retry; host-only publication checkpoints; and hostile extraction. Three compile-fail doctests verify scoped-value escape and Runtime/Context thread restrictions. The old experiments remain unchanged and their counterexamples still pass.

## Scope and build

- Exact `rquickjs = 0.14.0`, default features disabled, only `std`; core/sys at the same version. Binding revision `d7ef5eeae702fea24c03643064de454f1c1dd4b0`.
- Vendored QuickJS-NG 0.16.2, revision `0fdea21ff1090084e91dad812b343d92e79ba9d9`. No patch, fork, private struct access or numeric internal class IDs.
- Independent lockfile reuses the prior QuickJS experiment's resolution, changing only the root package name. No production dependencies, workspace membership, new external package versions, futures, loader, parallel, bindgen or dynamic-library features.
- Rust/Cargo 1.96 and a native C toolchain. This machine compiles the vendored C engine statically. Tested only on macOS 27.0 build 26A428 arm64, rustc 1.96.0 (ac68faa20 2026-05-25), Cargo 1.96.0 (30a34c682 2026-05-25), Apple clang 21.0.0. No cross-platform evidence.
- Default probes cap engine allocation at 16 MiB and stack at 256 KiB; the custom allocator test instead rejects a single allocation above 4096 bytes after setup and records a host latch. These are test settings, not exact portable heap/RSS quotas.

`extraction.rs` is a small descriptor/class-gated copy probe, not a complete JSON converter. It captures ordinary object/array class IDs and original prototypes before source runs, rejects proxies/exotics before inspection, reads own descriptors without getters, and copies primitive/dense-array/plain-record values. Depth/node/string checks bound the tested shapes; complete Unicode validation, aggregate byte accounting, all native brands and schema validation remain outside this spike. No JSON serialization or source cleanup is used. Unsafe blocks only wrap public C APIs with live scoped values and balanced ownership, or delegate the public allocator trait to RustAllocator; no invalid reference or intentionally undefined behavior is exercised.

Modules use fixed in-memory text and post-evaluation export reads solely to prove module state/reset. This does **not** implement RFC capture-before-evaluation or its export replacement checks. That separate engine-selection evidence gap is explicitly the next task; the spike does not substitute post-evaluation capture as a production rule. Full contexts deliberately include globals disallowed by RFC 0001; no sandbox claim.

## Reproduce

From the repository root, compile first so the external timeout measures execution rather than a cold C build:

```sh
cargo test --manifest-path experiments/quickjs-lifecycle-spike/Cargo.toml --locked --no-run
python3 - <<'PY'
import os
import signal
import subprocess

process = subprocess.Popen([
    'cargo', 'test', '--manifest-path',
    'experiments/quickjs-lifecycle-spike/Cargo.toml', '--locked',
    '--', '--test-threads=1', '--nocapture',
], start_new_session=True)
try:
    code = process.wait(timeout=60)
except subprocess.TimeoutExpired:
    os.killpg(process.pid, signal.SIGKILL)
    process.wait()
    raise SystemExit('FAIL: lifecycle spike exceeded 60-second external backstop')
raise SystemExit(code)
PY
cargo fmt --manifest-path experiments/quickjs-lifecycle-spike/Cargo.toml --check
cargo clippy --manifest-path experiments/quickjs-lifecycle-spike/Cargo.toml --locked --all-targets -- -D warnings
```

The process-group backstop is a POSIX test safeguard, not a runtime deadline implementation. Intentional CPU loops must exit through the engine interrupt hook, never through that backstop. Compile-fail diagnostics printed with `--nocapture` are expected passing tests. Tests use a real monotonic 10 ms deadline armed by a test-only source-entry signal (so startup cannot masquerade as loop interruption), without a latency assertion, an external-thread cancellation flag, and a separately controlled fake-clock publication checkpoint. They do not promise hard real-time interruption.
