# QuickJS public-API class bridge spike

**Outcome A — audited public-API class bridge viable.** This non-production experiment closes only the positive engine-class inspection prerequisite. It is not the complete RFC §6 converter, a sandbox or an engine selection.

Baseline: `d1d94be87a4c43b003a4568d8f4176034b9b3382`. Pins remain rquickjs/core/sys `0.14.0` at `d7ef5eeae702fea24c03643064de454f1c1dd4b0`, QuickJS-NG `0.16.2` at `0fdea21ff1090084e91dad812b343d92e79ba9d9`. Features: defaults off, `std`/`loader` on. Lock resolution is identical to the previous value-boundary spike except the root name.

The [review](../../docs/reviews/2026-09-28-quickjs-class-bridge-spike.md) records public API provenance, every safety invariant, the matrix and limits. The [completed plan](../../docs/plans/completed/2026-09-28-quickjs-class-bridge-spike.md) records complete validation.

## Tiny safe API

`ClassBridge::new(&Ctx)` captures opaque class IDs from public native `Object::new` and `Array::new` allocations. `classify(&Value)` returns `NonObject`, `OrdinaryObject`, `Array` or `OtherObject`; it returns `ForeignContext` for a mismatched value context. Only exact equality to the two captured classes can admit an object to later inspection. No candidate properties, prototype, constructor, iterator or JS helper are read or called.

[bridge.rs](src/bridge.rs) contains exactly **one unsafe block**, calling the public `JS_GetClassID` via existing sys bindings and `Value::as_raw`. No unsafe functions/impls, raw pointer exports, manual extern declarations, private IDs, engine layouts, patches or ownership conversions. The crate denies unsafe code except that one private function; tests forbid unsafe code.

Classification is not transfer acceptance: ordinary objects with accessors/custom prototypes and sparse arrays still get their engine class. A plain host-created service wrapper has no distinct engine brand and needs a separate host identity registry or branded representation in a future converter. The test-private opaque Rust callback is a registered native class and is rejected even after prototype changes. No full helper registry or service profile is designed here.

## Tests and ordering

Ten tests cover ordinary/null/class-created records, arrays and impostors, 26 standard native/function/iterator fixtures under three prototype replacements, 20 test-private excluded/native fixtures, active/revoked proxies, hostile getters/coercion/iterator/species/then/constructor hooks, poisoned builtins, an opaque host closure, mismatched contexts and fresh realm setup. Source hooks/traps never execute during classification. Labeled negative controls deliberately exercise unsafe alternatives. The prior Map blocker suite is unchanged and rerun.

Each test creates a fresh runtime/context, captures class metadata and any private fixtures while pristine, runs the earlier dynamic/global hardening scripts unchanged, then evaluates fixed diagnostic source. Removed constructors are never restored to globals. Private Date/buffer/GC/stack fixtures are host-retained test values, not production capabilities. No new module loading, operation capture, converter or job pumping is introduced. The existing capture/lifecycle suites retain that evidence.

## Reproduce

Use Rust/Cargo 1.96 or newer and the existing native C toolchain. From the repository root, precompile then use the established 60-second external process-group timeout:

```sh
cargo test --manifest-path experiments/quickjs-class-bridge-spike/Cargo.toml --locked --no-run
python3 - <<'PY'
import os, signal, subprocess
command = ['cargo', 'test', '--manifest-path',
           'experiments/quickjs-class-bridge-spike/Cargo.toml', '--locked',
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
cargo fmt --manifest-path experiments/quickjs-class-bridge-spike/Cargo.toml --check
cargo clippy --manifest-path experiments/quickjs-class-bridge-spike/Cargo.toml --locked --all-targets -- -D warnings
```

All 10 tests pass; fmt and strict Clippy pass. Runtime safeguards are 16 MiB memory and 256 KiB stack, not portable resource enforcement or transfer budgets. Miri was not run: the central proof crosses into native C, which ordinary Miri execution does not validate. No cross-platform claim follows from the macOS arm64 run.

New native class IDs fail closed without adding a blacklist entry. This assumes the engine preserves its public class-identity semantics; a future engine redesign reusing the ordinary class for a new exotic needs renewed audit. Numeric IDs are neither hard-coded nor persisted. Next: resume the full §6 boundary proof using this safe gate, separately establishing descriptor safety, Unicode, helper identities, recursion/bounds and incoming construction.
