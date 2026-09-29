# QuickJS RegExp admission implementation spike

Evidence for the [RegExp admission implementation review](../../docs/reviews/2026-09-29-quickjs-regexp-admission-impl-spike.md). It implements the pre-admission design from the [remedy review](../../docs/reviews/2026-09-29-quickjs-regexp-admission-review.md#next-task) as an experiment: a host-native RegExp facade with the finite wrap set, a trusted Rust gate, and exact-byte literal preflight before module declaration. It then tests whether a gate rejection actually stops source in-process. It is **not** production code, a policy value, a parser or engine selection, or an RFC change. The preserved blocker lives unchanged in [quickjs-resource-spike](../quickjs-resource-spike/README.md).

Baseline: `c2b8079d7748c5ab19567b60a06046913411cb5e`. Phase 3 remains active; RFC 0001 remains proposed. Result: **Outcome B**. Every compile route is gated, but pinned promise machinery converts uncatchable stops, including engine deadline interrupts, into ordinary rejections.

## Pins and boundaries

- rquickjs/core/sys `0.14.0` (`d7ef5eeae702fea24c03643064de454f1c1dd4b0`); QuickJS-NG `0.16.2` (`0fdea21ff1090084e91dad812b343d92e79ba9d9`). Defaults disabled; `std`/`loader` only.
- Boa parser/AST/interner `0.22.0` and Oxc `0.152.0`, as in the route probe. The lockfile equals the route-probe lockfile after renaming only the root package. No new dependency.
- Each realm is a fresh Runtime/Context with an 8 MiB heap limit and a 128 KiB stack setting. It is hardened by the unchanged dynamic-code and restricted-global bootstraps (the latter called with its allowlists), then the facade is installed. Hardening reuses the unchanged `inventory.js`, `pristine-inventory.json`, allowlists and `traverse.js` from [quickjs-global-surface-spike](../quickjs-global-surface-spike/README.md).
- `deny(unsafe_code)` everywhere except [ffi.rs](src/ffi.rs), which holds exactly two audited public-API calls: `JS_SetUncatchableError` (mark the host stop) and `JS_IsRegExp` (brand check invisible to source). No private API, patch, network, credentials or source rewriting.
- `L_MAX` = 4,096 UTF-16 code units is a candidate host bound for this experiment, not a proposed portable value. `MODULE_BYTES` = 1 MiB is defense in depth for the preflight.

## Structure

| File | Role |
| --- | --- |
| [lib.rs](src/lib.rs) | `Gate` (bound, trusted latch, deadline/cancellation check, compile/admit/readmit/reject counters, interrupt handler), realm construction, hardening and facade installation. |
| [gate.js](src/gate.js) | Trusted bootstrap. Captures the native constructor and wrapped natives in a closure, then installs the facade constructor, `compile`, `@@split`, `@@matchAll` and `String.prototype.match`/`matchAll`/`search` wrappers with ES2023-order single coercion. |
| [preflight.rs](src/preflight.rs) | Module byte bound, Oxc raw-token literal measurement, bound check **before** Boa validation, Boa visitor list, exact agreement, and a `Loader` that preflights every module in the static graph before declaration. |
| [reach.js](src/reach.js) | Bounded reachability walk from source-visible roots (including thrown errors) for the native constructor and wrapped natives. |
| [tests.rs](src/tests.rs) | Sixteen tests. |

## Run

Every test is bounded in-process. The preserved 320,007-unit reproducer is never handed to the native compiler. The established guard is still used:

```sh
cargo test --manifest-path experiments/quickjs-regexp-admission-impl-spike/Cargo.toml --locked --no-run
python3 - <<'PY'
import os, signal, subprocess
command = ['cargo', 'test', '--manifest-path',
           'experiments/quickjs-regexp-admission-impl-spike/Cargo.toml', '--locked',
           '--', '--test-threads=1', '--nocapture']
process = subprocess.Popen(command, start_new_session=True)
try:
    result = process.wait(timeout=60)
except subprocess.TimeoutExpired:
    os.killpg(process.pid, signal.SIGKILL)
    process.wait()
    raise SystemExit('FAIL: external 60-second timeout')
raise SystemExit(result)
PY
cargo fmt --manifest-path experiments/quickjs-regexp-admission-impl-spike/Cargo.toml --check
cargo clippy --manifest-path experiments/quickjs-regexp-admission-impl-spike/Cargo.toml --locked --all-targets -- -D warnings
```

## Sixteen tests

| Test | Observation |
| --- | --- |
| `every_dynamic_route_rejects_before_native_compile_with_trusted_latch` | 30 construction, alias, species, subclass, String-method, `compile` and generic-receiver `@@split`/`@@matchAll` routes, each run with the 64,000-reference reproducer and with an over-bound invalid pattern inside `try`/`catch`/`finally`. Each gives an uncatchable stop, latch `ResourceLimit`, zero native compiles, no source marker, under 1 s, and a healthy fresh realm. |
| `non_compiling_routes_and_genuine_internal_source_are_not_rejected` | String `replace`/`replaceAll`/`split`/`includes`, `exec`, `@@replace` and a genuine RegExp with an overridden `source` getter are not rejected; clones of genuine internal source are counted as readmitted. |
| `bound_edges_and_genuine_representation` | Exactly `L_MAX` is admitted, `L_MAX + 1` is rejected. The escaped `source` of an admitted pattern can exceed the bound, so genuine clones reuse internal source rather than re-measuring. |
| `facade_preserves_identity_shape_and_es2023_semantics` | `name`, `length`, prototype, `instanceof`, constructor link, species, call-form identity, clones, subclassing and descriptors. |
| `facade_coerces_once_in_es2023_order_unlike_pinned_native` | Facade order `match, source, flags, prototype, toString:P, toString:F`; the pinned native reads `prototype` last. |
| `generic_split_and_match_all_are_observably_faithful_to_native` | Logged observable reads and results on generic receivers equal the pinned native for bounded patterns. |
| `native_constructor_and_wrapped_natives_are_unreachable` | Negative control finds the native constructor; the facade walk is clean (357 nodes, 1,457 edges); the unchanged global-surface walk stays clean. |
| `stop_is_uncatchable_through_catch_finally_async_and_reactions` | Seven shapes: `try`/`catch`/`finally`, async function, promise reaction, generator `finally`, backtrace hooks, `forEach`, `for-of` iterator close. No source runs after the stop. |
| `promise_machinery_swallows_gate_and_deadline_stops_in_process` | **Outcome-deciding.** Five shapes (Promise executor, resolve-function thenable, `Promise.resolve` thenable, `await` thenable, `Promise.all` iterator). The gate stop becomes a rejection and source continues; a reaction receives the stop object. With the production handler, a latched deadline interrupt is swallowed the same way: 1,000 loop iterations complete after about 1,002 callbacks. |
| `await_resolution_ignores_the_global_promise_binding` | With global `Promise` set to `undefined`, `await` still reads a thenable synchronously, so a JavaScript-level wrapper cannot close that path. |
| `gate_reads_deadline_and_cancellation_before_every_admitted_compile` | A deadline becoming pending before the fourth construction stops after three compiles; pending cancellation stops before any. |
| `literal_preflight_bounds_before_boa_and_quickjs` | Bound edges in UTF-16 units (astral characters count two). The 64,000-reference literal is rejected by raw measurement in about 8 ms, before Boa validation. Oversized modules are rejected. |
| `literal_preflight_agreement_is_exact_and_fail_closed` | Division/literal ambiguity, escapes, flags, comments and templates; synthetic disagreements, a Boa syntax error and an Oxc lexing error all reject. |
| `every_module_in_the_static_graph_is_preflighted_before_declaration` | A helper imported by the entry is preflighted in the loader; an over-bound helper literal rejects the graph at declaration with no evaluation. |
| `calibrate_worst_case_families_and_flags_at_candidate_bound` | Eleven families by six flag sets at `L_MAX` and `2 * L_MAX`; worst at `L_MAX` about 81 ms. |
| `matching_checkpoint_interval_audit` | Five matching cases with a recording handler; every case reaches a poll; longest observed gap about 43 ms. |

Timings are local debug-profile observations (macOS 27.0 arm64, Rust 1.96.0), not portable constants.
