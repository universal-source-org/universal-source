# QuickJS value-boundary prerequisite spike

Non-production Phase 3 evidence for RFC 0001 §6. **Outcome B — blocker demonstrated** under this unit's prohibition on unsafe Rust: pinned rquickjs lacks a safe positive ordinary-object class query. Its property-value iterators also invoke getters. This experiment stops at that prerequisite; it does not implement or claim a complete converter.

Pins: rquickjs/core/sys `0.14.0`, revision `d7ef5eeae702fea24c03643064de454f1c1dd4b0`; QuickJS-NG `0.16.2`, revision `0fdea21ff1090084e91dad812b343d92e79ba9d9`. Default features off, `std`/`loader` on. The lockfile differs from the restricted-global spike only in the root package name.

The [review](../../docs/reviews/2026-09-28-quickjs-value-boundary-spike.md) records the counterexample, public API audit, unproven §6 requirements and narrow next step. The [completed plan](../../docs/plans/completed/2026-09-28-quickjs-value-boundary-spike.md) records all validation commands/results.

## Reproduce

Use Rust/Cargo 1.96 or newer and the native C toolchain used by the earlier spikes. From the repository root:

```sh
cargo test --manifest-path experiments/quickjs-value-boundary-spike/Cargo.toml --locked --no-run
python3 - <<'PY'
import os, signal, subprocess
command = ['cargo', 'test', '--manifest-path',
           'experiments/quickjs-value-boundary-spike/Cargo.toml', '--locked',
           '--', '--test-threads=1', '--nocapture']
process = subprocess.Popen(command, start_new_session=True)
try:
    result = process.wait(timeout=60)
except subprocess.TimeoutExpired:
    os.killpg(process.pid, signal.SIGKILL)
    process.wait()
    raise SystemExit('external 60-second timeout')
raise SystemExit(result)
PY
cargo fmt --manifest-path experiments/quickjs-value-boundary-spike/Cargo.toml --check
cargo clippy --manifest-path experiments/quickjs-value-boundary-spike/Cargo.toml --locked --all-targets -- -D warnings
```

Six behavior tests and two expected E0133 compile-fail doc tests pass. `#![forbid(unsafe_code)]` applies to this crate. No raw C call executes; the compile-fail examples demonstrate why the available public C class/descriptor APIs cannot be used directly within this constraint. Dependencies contain their existing internal unsafe implementations, unchanged.

## What runs

`src/tests.rs` includes the existing dynamic and global hardening scripts and policy snapshots unchanged. Each test creates a fresh runtime/context. It captures any trusted prototype/native test handle before hardening, applies both bootstraps, then evaluates fixed diagnostic fixtures. A retained Proxy constructor is used only by the host to create a hostile test candidate; it is never restored to the global. Integer-only native callbacks count source hook execution.

`record_gate` inspects only direct value predicates. A Proxy is rejected without traps. Other candidates that need positive ordinary-class proof yield `BlockedOrdinaryClassProof`, including valid records. That is an embedding blocker, not an RFC `INVALID_RESULT` result or a working conversion. Arrays are excluded from this **record prerequisite** only; no array transfer policy is implemented.

The separate negative controls demonstrate prototype-only admission of a mutated Map, getters invoked by `Object::get` and `props`, a source tag getter invoked by captured native `Object.prototype.toString`, and a Proxy trap invoked by own-key enumeration. Known fixture arrays/objects are inspected only to establish these counterexamples, never as a substitute for classifying arbitrary source candidates. No hostile hook runs in the gate itself.

Test safeguards are 16 MiB engine memory, 256 KiB engine stack and a 60-second external process-group timeout. These are not transfer limits or normative constants. Depth/node/string/key/expanded-copy accounting and incoming conversion remain unproved because the class prerequisite blocks this unit. No production runtime, source rewriting, RFC amendment or engine/parser selection is included.
