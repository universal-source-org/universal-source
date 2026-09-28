# QuickJS restricted-global surface spike

Non-production Phase 3 feasibility evidence for RFC 0001 §9. **Outcome A — restricted-global surface viable** for the tested pinned surface. This is not a complete sandbox, engine selection, JavaScript conformance claim or production execution.

Pins: rquickjs/core/sys `0.14.0`, revision `d7ef5eeae702fea24c03643064de454f1c1dd4b0`; QuickJS-NG `0.16.2`, revision `0fdea21ff1090084e91dad812b343d92e79ba9d9`. Default features off, `std`/`loader` on. Lock resolution equals the dynamic-code spike after changing only the root package name. Production dependencies are untouched.

The [review](../../docs/reviews/2026-09-28-quickjs-global-surface-spike.md) explains the policy, source evidence, tests, limitations and next gap. The [completed plan](../../docs/plans/completed/2026-09-28-quickjs-global-surface-spike.md) records full repository validation.

## Reproduce

Use Rust/Cargo 1.96 or newer and the native C toolchain used by the earlier QuickJS spikes. Build the test executable first, then run it under an external process-group timeout:

```sh
cargo test --manifest-path experiments/quickjs-global-surface-spike/Cargo.toml --locked --no-run
python3 - <<'PY'
import os, signal, subprocess
command = ['cargo', 'test', '--manifest-path',
           'experiments/quickjs-global-surface-spike/Cargo.toml', '--locked',
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
cargo fmt --manifest-path experiments/quickjs-global-surface-spike/Cargo.toml --check
cargo clippy --manifest-path experiments/quickjs-global-surface-spike/Cargo.toml --locked --all-targets -- -D warnings
```

All 19 tests pass. The negative controls deliberately observe real clock/random/stack behavior; their text is environment-specific and is not a conformance expectation. The suite also passes when the wrapper's child environment specifies `TZ=UTC` or `TZ=Asia/Shanghai`; epoch Date output changes in the controls while hardened restrictions remain identical. No engine timer or memory setting replaces the external timeout.

## Files and trust boundary

- `src/pristine-inventory.json`: observed descriptors of 71 global own keys and 128 additional intrinsic/root objects. Each entry is `[name, value type or accessor, writable or null, enumerable, configurable]`. Symbol keys use descriptive strings solely for fixture reporting. This is an engine drift snapshot, not normative design.
- `src/allowed-globals.json`: the exact 38 names from RFC §9; no bookkeeping globals or symbols permitted.
- `src/allowed-intrinsics.json`: explicit ES2023 keys for 72 retained intrinsic objects, including iterator/function-family prototypes and array unscopables. BigInt gets the required own throwing locale method missing in the pristine engine.
- `src/inventory.js`: trusted descriptor inventory; no package executes before it passes the snapshot gate.
- `src/harden.js`: trusted fixed bootstrap, invoked after including the earlier dynamic-code bootstrap **unchanged**. It removes extensions and ambient objects, supplies the native-bound Symbol facade, removes Error stack access, and locks throwing ambient-method guards. Only guards/necessary links are locked; the realm stays mutable.
- `src/probes.js`, `src/entry.js`: fixed diagnostic package bytes, never rewritten. The latter uses a static helper, captured async operation, and a test-private injection argument.
- `src/binding.js`: creates frozen null-prototype test wrappers around an authority-free Rust integer callback. It specifies no service profile.
- `src/traverse.js`: test-private forbidden-identity witness, descriptor/prototype traversal with depth 16/node 4096 caps; exceeding either fails. Original targets retained for comparison never reach source.
- `src/tests.rs`: public API setup, full reused 80-route compiler corpus, controls, descriptor/mutation/reset tests, host callback/loader/capture integration and bounded job pumping for the allowed async operation.

The source can create a property named Date, Proxy or Math.random containing its own code/data. That does not recover a native clock, proxy constructor or RNG. Any late intrinsic installation or host injection that returns a removed original violates this strategy's setup discipline. Snapshot/table drift requires review; never regenerate expectations merely to make an upgraded engine pass.
