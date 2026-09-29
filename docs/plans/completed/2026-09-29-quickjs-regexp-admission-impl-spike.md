# RegExp admission feasibility implementation spike

Status: completed — **Outcome B**. Baseline `c2b8079d7748c5ab19567b60a06046913411cb5e` (`docs(engine): review RegExp compilation admission remedy`); clean tree, no other active plans. Phase 3 active.

Scope: exactly one experiment-only spike, in a new `experiments/quickjs-regexp-admission-impl-spike` crate on pinned public APIs, testing whether the Outcome A pre-admission design from the [remedy review](../../reviews/2026-09-29-quickjs-regexp-admission-review.md#next-task) actually closes the RegExp compilation blocker while keeping RFC 0001 §§5/7/9/10 semantics and the in-process runtime. Resolve to Outcome A (design holds) or Outcome B (a route cannot be gated → escalate to process containment).

Deliverables:

1. A host-native RegExp facade over a host-only native reference, with the finite wrap set (constructor + `prototype.constructor` link, `String.prototype.match`/`matchAll`/`search`, `RegExp.prototype.compile`, `RegExp.prototype[@@split]`/`[@@matchAll]`), single ES2023-order coercion, one measurement, and a trusted deadline check at every gate.
2. A trusted Rust gate holding the length bound, deadline/cancel latch, native-compile counter, and reason; rejection is an uncatchable `RESOURCE_LIMIT` via one narrow audited public-API unsafe call (`JS_SetUncatchableError`), with a safe latch fallback recorded.
3. Preserved-prototype/`instanceof`/species/subclass identity; a reachability walk proving the native constructor is unrecoverable.
4. Exact-byte, fail-closed literal preflight (Boa visitor + Oxc token agreement) before `Module::declare`.
5. Routing every inventory row, including the 64,000-reference reproducer through each dynamic route, to an uncatchable `RESOURCE_LIMIT` with zero native compile and a healthy fresh realm.
6. Worst-case family/flag calibration and a matching checkpoint-interval audit.

Non-goals (unchanged): no production runtime change, RFC edit, new normative RegExp restriction, QuickJS patch, dependency upgrade, second engine, engine/parser selection, ADR, module/path completion, IPC/services, push/tag/release. Pins fixed. The external watchdog is only a safety net, never engine evidence. The resource Outcome B and its reproducer stay intact.

## Steps

1. Scaffold the crate (Cargo.toml + lockfile from the route-probe spike; `forbid(unsafe_code)` except one audited gate module).
2. Build the Rust gate and the JS facade bootstrap; install after the dynamic/global hardening.
3. Build literal preflight with two-parser agreement.
4. Write tests: identity/reachability, every route to uncatchable `RESOURCE_LIMIT` with zero compile, literal preflight, calibration, matching audit, fresh-realm health, uncatchable-through-catch/finally/await.
5. Validate: guarded run; fmt/strict Clippy; rerun the resource blocker and 11 resource tests; regression suite; doc/link checks; unsafe audit; protected paths/pins unchanged.
6. Review + move this plan to completed + minimal ROADMAP/ARCHITECTURE updates; commit locally; do not push.

## Findings during work

- Step 1 done: crate scaffolded; lockfile derived from the route-probe spike with only the root package renamed.
- Blocking finding (probe, pinned source): QuickJS-NG converts pending exceptions into promise rejections without checking `JS_IsUncatchableError` in `js_promise_constructor`, the resolve function's `then` lookup (`quickjs.c:55801`, reached by `await` through the internal `%Promise%` at 21516), `Promise.resolve`, `Promise.all` iteration and async generators. With a handler that interrupts on every poll, 50-iteration loops over each path completed normally after 50 interrupts; plain `try`/`catch`/`finally` stayed uncatchable. The `await` path uses intrinsics that a JS facade cannot wrap, so neither a gate rejection nor an engine deadline interrupt is guaranteed to stop source in-process. Expected consequence: Outcome B, with process containment as the escalation. The facade, preflight, calibration and matching audit are still built to record which parts hold.

- Steps 2–6 done. Every compile route was gated; the blocking finding was confirmed with 1,000-iteration loops over five deterministic shapes (about 1,002 callbacks each). The async generator shape was dropped from the asserted set because its body runs in a later job and the poll position was not deterministic.

## Result

**Outcome B — a gated stop is not guaranteed to stop source in-process; escalate to process-containment review.** See the [review](../../reviews/2026-09-29-quickjs-regexp-admission-impl-spike.md) and the [spike README](../../../experiments/quickjs-regexp-admission-impl-spike/README.md).

What holds:

- all 30 inventoried dynamic routes, with the 64,000-reference reproducer and an over-bound invalid pattern, give an uncatchable stop with latch `ResourceLimit`, zero native compiles and a healthy fresh realm;
- the native constructor and wrapped natives are unreachable (clean walk, 357 nodes);
- coercion is single and in ES2023 order;
- generic-receiver `@@split`/`@@matchAll` match the pinned native observably;
- exact-byte literal preflight measures before Boa validation, fails closed, and covers the static import graph;
- the worst compile at the candidate bound of 4,096 units is about 81–86 ms (debug);
- every matching case reaches a poll (longest gap about 43 ms).

What fails: pinned promise machinery converts uncatchable errors into rejections (`quickjs.c` 55916, 55801/55805, 21516, 56241, 21891). This absorbs both gate stops and engine deadline interrupts. `await` ignores the global `Promise`, so a JavaScript facade cannot close it. The finding qualifies in-process CPU enforcement generally on this pin.

Also recorded, not fixed: the resource spike evaluates the restricted-global hardener without calling it.

Remaining gaps:

- the swallow sites that were identified by reading only;
- the untested and unauthorized engine-level fix;
- release-build and cross-host calibration;
- the subject-length term in matching intervals;
- parser selection and complete module/path preflight.

Next: the process-containment review specified in the review.

## Validation

Host: macOS 27.0 (26A428), arm64, Rust/Cargo 1.96.0, debug test profile. The sandbox cannot build `rquickjs-sys`, so Cargo commands ran unsandboxed with `--offline`, crate-local target directories and no dependency changes. Experiment suites ran under a 60-second process-group guard (`perl -e 'setpgrp(0,0); alarm 60; exec @ARGV' cargo test …`); no guard fired.

| Exact command | Result |
| --- | --- |
| `cargo test --manifest-path experiments/quickjs-regexp-admission-impl-spike/Cargo.toml --offline --locked -- --test-threads=1 --nocapture` (guarded) | 16 pass (about 4.6 s); also stable in five parallel runs during development. |
| `cargo fmt --manifest-path experiments/quickjs-regexp-admission-impl-spike/Cargo.toml --check` | Pass. |
| `cargo clippy --manifest-path experiments/quickjs-regexp-admission-impl-spike/Cargo.toml --offline --locked --all-targets -- -D warnings` | Pass. |
| `cargo test --manifest-path experiments/quickjs-resource-spike/Cargo.toml --offline --locked -- --test-threads=1 --nocapture` (guarded) | 11 pass, unchanged. |
| `cargo test --manifest-path experiments/quickjs-regexp-admission-spike/Cargo.toml --offline --locked -- --test-threads=1 --nocapture` (guarded) | 8 pass, unchanged. |
| `python3 experiments/quickjs-resource-spike/probe_regexp.py` | **Exit 2** as preserved: 6 / 104 / 1,699 ms, zero callbacks; 64,000 references killed by its own five-second guard. Not engine evidence. |
| Ten prior Phase 3 experiment `cargo test … -- --test-threads=1 --nocapture` commands (guarded) | 151 pass: complete-value 20, class-bridge 10, value-boundary 8, global 19, dynamic 14, static-analysis 24, module-capture 14, lifecycle 18, engine 10, Boa reaction 14. |
| `cargo test --manifest-path runtime/Cargo.toml --locked --offline` | 53 pass. |
| `cargo test --manifest-path runtime/Cargo.toml --locked --offline --test declarative_conformance -- --nocapture` | 34/34 `PASS`. |
| `cargo build` / `cargo fmt --check` / `cargo clippy --all-targets -- -D warnings` on `runtime/Cargo.toml` | Pass. |
| `.venv/bin/python -m unittest discover -s conformance -p 'test_*.py'` | 6 pass. |

Captured summary: `/tmp/us-regexp-impl-regression.txt` (execution record only).

Development corrections:

- The first swallow probe armed the handler before evaluation and interrupted before the loop body ran; arming moved to a host function called inside the shape.
- `Module::declare` loads imports eagerly, so a rejected helper surfaces at the entry's declaration; `declare` returns a nested result.
- The async generator shape was non-deterministic and was removed from the asserted set; the `quiet_latch` test knob isolates the gate stop from latched polling.
- A wrong `@@split` expectation (a 4,096-slash pattern on `'//'`) was corrected to length 1.
- The capture-group family hit the 255-capture limit and full-length nesting failed early; non-capturing and depth-100 families were added, and both error rows are kept.
- A Clippy `neg_cmp_op_on_partial_ord` finding was fixed as `units.is_nan() || units > bound`.

## Audit and delivery

Two `#[allow(unsafe_code)]` functions in `ffi.rs` (public `JS_SetUncatchableError`, `JS_IsRegExp`), each with a SAFETY comment; the crate is otherwise `deny(unsafe_code)`. No private API, engine patch or dependency change. The new lockfile equals the route-probe lockfile after renaming the root package. Production `runtime/`, `spec/`, `conformance/`, `examples/`, `docs/decisions/` and every prior experiment are unchanged against the baseline. RFC 0001 remains proposed; no engine or parser is selected; no ADR exists.

Changed files: `docs/ARCHITECTURE.md`, `docs/ROADMAP.md`, the review, this plan, and `experiments/quickjs-regexp-admission-impl-spike/` (`.gitignore`, `Cargo.toml`, `Cargo.lock`, `README.md`, `src/ffi.rs`, `src/gate.js`, `src/lib.rs`, `src/preflight.rs`, `src/reach.js`, `src/tests.rs`). Local Conventional Commit only; no push, tag, release or settings change.
