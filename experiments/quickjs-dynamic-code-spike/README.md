# QuickJS dynamic compilation suppression spike

**Non-production; Outcome A — dynamic compilation suppression viable.** Fourteen tests demonstrate the pinned public-API strategy against 80 dynamic compilation routes, mutation, initialization ordering and Model B reset. This is not a complete sandbox or engine selection. See the [review](../../docs/reviews/2026-09-28-quickjs-dynamic-code-suppression-spike.md) and [completed plan](../../docs/plans/completed/2026-09-28-quickjs-dynamic-code-suppression-spike.md).

## Mechanism and boundaries

The fixed trusted [bootstrap](src/harden.js) runs through host `Ctx::eval` on a pristine context before any package code or binding installation. It wraps native eval and the four function-family constructors in standard Proxy call/construct guards that throw a captured native EvalError. It locks global eval/Function and the four prototype constructor links, and redirects specialized constructors' parent links to guarded Function. A hidden frozen null-prototype handler cannot acquire traps from later prototype mutation. No untrusted text is compiled by a guard.

Function prototypes, constructor-family relationships, ordinary calls/construction, names and lengths are preserved in tested controls. No unrelated object/prototype is frozen. Guard proxies use the engine's standard object model; source does not receive their targets or handlers. Deleting global Proxy after bootstrap does not disable them. This is not an assertion that the default context satisfies the RFC global allowlist: the full context deliberately leaves unrelated globals/extensions intact for this focused probe.

[probes.js](src/probes.js) is a fixed diagnostic module compiled unchanged. It captures only already-guarded aliases. Eighty attempts include direct/indirect eval, four constructor families, prototypes, descriptors, meta-constructor chains, Reflect, call/apply/bind, bound construction and malformed text. Error classification compares against the saved intrinsic EvalError prototype, not a mutable global name or message. Payload markers remain absent. [tests.rs](src/tests.rs) also demonstrates original payload execution without hardening, leaks from incomplete hardening/late installation, host compile/link/capture order, and fresh-realm reset.

The module fixture's extra exports/test globals are observation tools, not an RFC source API or production dispatcher. The tiny capture fixture reuses the prior public native-hook ordering without implementing path containment or exposing compilation services. Compilation stays enabled for host-owned package bytes. Host code must never publish original compiler handles, proxy handlers, an eval bridge, or add compiler-bearing intrinsics after hardening.

## Pins and run

- rquickjs/core/sys **0.14.0**, revision `d7ef5eeae702fea24c03643064de454f1c1dd4b0`.
- QuickJS-NG **0.16.2**, revision `0fdea21ff1090084e91dad812b343d92e79ba9d9`.
- Only direct dependency: rquickjs `=0.14.0`, default features off, `std` and `loader` enabled. The lockfile is identical to the module-capture crate's except for the root package name; no version changed.
- Local evidence: Rust/Cargo 1.96.0, macOS 27.0 (26A428), arm64. Vendored QuickJS C build requirements are unchanged; no cross-platform proof follows.

From repository root, prebuild then run with an external timeout:

```sh
cargo test --manifest-path experiments/quickjs-dynamic-code-spike/Cargo.toml --locked --no-run
python3 - <<'PY'
import os, signal, subprocess
command = ['cargo', 'test', '--manifest-path',
           'experiments/quickjs-dynamic-code-spike/Cargo.toml',
           '--locked', '--', '--test-threads=1', '--nocapture']
p = subprocess.Popen(command, start_new_session=True)
try:
    raise SystemExit(p.wait(timeout=60))
except subprocess.TimeoutExpired:
    os.killpg(p.pid, signal.SIGKILL)
    p.wait()
    raise SystemExit('FAIL: external 60-second timeout')
PY
cargo fmt --manifest-path experiments/quickjs-dynamic-code-spike/Cargo.toml --check
cargo clippy --manifest-path experiments/quickjs-dynamic-code-spike/Cargo.toml --locked --all-targets -- -D warnings
```

Observed: **14 pass, 0 failed/ignored, 0 doc tests**. Negative controls intentionally execute harmless marker payloads; guarded paths never do. No tests access network/filesystem services. Prior experiments, RFC, production runtime and dependencies are unchanged. Next is only the broader restricted-global/ambient-authority feasibility gap, not production execution.
