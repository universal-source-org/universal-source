# RegExp admission implementation spike

## Outcome and scope

**Outcome B — a gated stop is not guaranteed to stop source in-process; escalate to process-containment review.**

The pre-admission design from the [remedy review](2026-09-29-quickjs-regexp-admission-review.md) was implemented as an experiment, and its bounding parts hold:

- every inventoried dynamic route, including the preserved 64,000-reference reproducer, reaches the trusted gate and is refused with zero native compiles;
- the native constructor and wrapped natives are unreachable from source;
- coercion is single and in ES2023 order;
- generic-receiver `@@split`/`@@matchAll` match the pinned native observably;
- literals are measured from exact bytes and bounded before Boa or QuickJS sees them, across the static import graph;
- the worst measured compile at the candidate bound of 4,096 UTF-16 units is about 81–86 ms (debug build).

The design fails at its last step: the refusal must be a stop that source cannot survive. The gate throws an Error marked with the public `JS_SetUncatchableError`. Pinned QuickJS-NG honours that mark in the interpreter's unwind, async functions and reaction jobs. It does not honour it where promise machinery converts a pending exception into a rejection:

- the `Promise` executor;
- the resolve function's `then` lookup, which `Promise.resolve` and every `await` reach through the internal `%Promise%`;
- `Promise.all` iteration;
- async generator resumption.

The engine's own deadline interrupt uses the same mark, so it is converted the same way. Source that re-enters such a context after a latched stop keeps running. The test shows 1,000 loop iterations completing after about 1,002 interrupt callbacks; nothing but the test's own loop count bounds it.

The `await` path ignores the global `Promise` binding, so no JavaScript-level facade can close it. Forbidding `await` or promises would be a new normative restriction, which RFC 0001 does not permit. Checking the mark at those engine sites would be a QuickJS patch or upgrade, which this task does not authorize.

RFC 0001 §5 requires "eventual interruption under that policy" and §10 "a credible bound for source-created allocations and CPU work". Neither holds for this source in-process. By the task's rule, a route that cannot be gated means Outcome B, with escalation to a process-containment review.

This finding is not specific to RegExp. It limits in-process deadline and cancellation enforcement on the pinned engine for any source that uses these promise paths. It is recorded here as new evidence. The [resource review](2026-09-29-quickjs-resource-spike.md) is not rewritten: its CPU tests covered synchronous `catch`/`finally`, nested catches and queued markers, which do honour the mark, and those results stand. The original resource **Outcome B stands unchanged**. The reproducer still exits 2, and the native compiler still does not poll.

Baseline: `c2b8079d7748c5ab19567b60a06046913411cb5e` (`docs(engine): review RegExp compilation admission remedy`), clean tree, no other active plan. Phase 3 remains active. [RFC 0001](../../spec/rfcs/0001-javascript-execution-binding.md) remains proposed and unedited; no engine or parser is selected; no ADR exists. Evidence comes from the sixteen-test [implementation spike](../../experiments/quickjs-regexp-admission-impl-spike/README.md) and a pinned-source audit. This task accessed no external system; ES2023 steps are cited from the specification text as known to the reviewer.

## Pins and local provenance

rquickjs/core/sys **0.14.0** (`d7ef5eeae702fea24c03643064de454f1c1dd4b0`), QuickJS-NG **0.16.2** (`0fdea21ff1090084e91dad812b343d92e79ba9d9`), Boa parser/AST/interner **0.22.0**, Oxc **0.152.0**. Unchanged; the spike's lockfile equals the route-probe lockfile after renaming only the root package. `rquickjs-sys-0.14.0/quickjs/quickjs.c` has SHA-256 `3a6b52a225c21709ef1fd314183efd3af862e867ce4ef8caabcae67922a745b3`, equal to both prior reviews; line numbers below refer to it. Environment: macOS 27.0 (26A428), arm64, Rust/Cargo 1.96.0, debug test profile.

## What was built

- **Trusted gate (Rust).** The bound, a first-wins latch (`Cancelled`, `Timeout`, `ResourceLimit`), a deadline/cancellation check at every gate entry and admission, counters for admitted, readmitted and rejected patterns and for native compiles, and the interrupt handler. Source cannot reach this state.
- **Facade (trusted bootstrap `gate.js`).** Installed after the unchanged dynamic-code and restricted-global hardening, before any package code. It captures the native constructor, `compile`, `@@split`, `@@matchAll` and `String.prototype.match`/`matchAll`/`search` in a closure. It replaces the global `RegExp`, `RegExp.prototype.constructor` and those methods with wrappers that keep the original prototype, names, lengths and descriptors. The global constructor is a Proxy over the native with a frozen null-prototype handler, so `typeof`, `name`, `length`, species and subclassing behave natively while apply/construct go through the gate.
- **Two audited unsafe calls** in `ffi.rs`: `JS_SetUncatchableError` marks the host stop object, and `JS_IsRegExp` distinguishes genuine from generic receivers without source-observable reads. Both are public C APIs; the crate is otherwise `deny(unsafe_code)`.
- **Literal preflight (Rust).** A 1 MiB module byte bound, then Oxc raw-mode tokens (no pattern validation) to measure each literal's UTF-16 length, then the bound, then Boa's validating parse and visitor, then exact agreement of body and flags. Any error or disagreement rejects. A `Loader` runs the same preflight on every imported module before declaring it.

## Results that hold

| Obligation | Result |
| --- | --- |
| Every inventoried dynamic route gated | 30 routes × 2 patterns (reproducer and an over-bound invalid pattern) inside `try`/`catch`/`finally`: uncatchable stop, latch `ResourceLimit`, zero native compiles, no source marker, under 1 s, healthy fresh realm. Routes: `new`/call forms, flags, object and RegExp-like patterns, call-form `constructor: null`, alias, `call`/`apply`/`bind`, bound `new`, `Reflect.construct` with and without `newTarget`, subclass, `(/a/).constructor`, `prototype.constructor`, String `match` (three forms), `matchAll`, `search`, `compile` (three forms), and generic `@@split`/`@@matchAll`, directly and through `String.prototype.split`/`matchAll`. |
| Non-compiling routes untouched | String `replace`/`replaceAll`/`split`/`includes`, `exec`, `@@replace` pass unrefused. |
| Genuine representation | Escaped `source` can exceed the admitted length (4,096 slashes), so genuine clones and species constructs reuse internal source through a native clone. They are counted as **readmitted**: deadline check, no re-measurement. Worst-flag calibration covers the flag change. |
| Bound edges | Exactly 4,096 admitted; 4,097 refused. |
| Native unreachable | Negative control finds `native RegExp` from `root.RegExp`. After installation the facade walk is clean (357 nodes, 1,457 edges, depth 3) with thrown errors as roots; the unchanged global-surface walk stays clean (368 nodes, 1,482 edges). |
| ES2023 single coercion | Facade `match, source, flags, prototype, toString:P, toString:F`. The pinned native reads `prototype` last. |
| Generic receivers | Facade `@@split`/`@@matchAll` on generic receivers produce the same logged reads, results and iterator brand as the pinned native. |
| Identity and semantics | Name, length, prototype, `instanceof`, constructor link, species, call-form identity, clones, subclassing, `Reflect.construct` `newTarget`, descriptors, errors, `compile` brand check, `matchAll` non-global `TypeError`, ES2023 `RegExpCreate` stringification. |
| Deadline density | A deadline becoming pending before the fourth construction stops after three compiles; pending cancellation stops before any. |
| Uncatchable where honoured | `try`/`catch`/`finally`, async function `try`, promise reaction `.catch`/`.finally`, generator `finally`, `forEach`, `for-of` iterator `return`: no source runs after the stop. Backtrace hooks that could run source are inert after hardening. |
| Literal preflight | 64,000-reference literal (320,007 units) rejected in about 8 ms by raw measurement, before Boa validation. UTF-16 counting (astral characters count two). Seven agreement fixtures cover division/literal ambiguity, escapes, flags, comments and templates. Synthetic disagreements, a Boa syntax error for `/(/` and an Oxc lexing error reject. An over-bound literal in an imported helper rejects the graph at declaration with no evaluation. |

Three deliberate differences from the pinned native follow ES2023, the RFC baseline:

- the `newTarget.prototype` read position;
- `RegExpCreate` stringification in String methods;
- `GetMethod(@@match)` on non-nullish primitives, where the pinned native checks Objects only (a later edition's behaviour).

The `v` flag (ES2024) is accepted by the native compiler and left to it. That is a baseline conformance gap outside this task.

## Blocking finding: promise machinery absorbs uncatchable stops

Uncatchability is a flag on the Error object (`JS_IsUncatchableError` 12049, `JS_SetUncatchableError` 12068). It takes effect only where code checks it:

- the interpreter's exception unwind skips handlers (20901);
- async function resumption does not deliver it into the function (21482, 21491);
- promise reaction jobs propagate it (55584).

The sites below instead take the pending exception with `JS_GetException` and settle a promise with it. Source continues.

| Site | Lines | Deterministic tested shape |
| --- | --- | --- |
| `js_promise_constructor` executor call | 55900; rejection at 55916 | `new Promise(() => { STOP; })` |
| `js_promise_resolve_function_call` `then` lookup | 55766; `then` read 55801, `fail_reject` 55805 | `new Promise(r => r({ get then() { STOP; } }))`; `Promise.resolve(thenable)` (56032) reaches the same resolve function |
| `await` | `js_async_function_resume` calls `js_promise_resolve(ctx, ctx->promise_ctor, …)` at 21516, reaching the same resolve function | `(async () => { await { get then() { STOP; } }; })()` |
| `js_promise_all` iteration | 56218; `fail_reject` 56241 | `Promise.all({ [Symbol.iterator]() { STOP; } })` |
| Async generator resumption | 21891 rejects the request | Observed, but its body is deferred to a job, so the poll position was not deterministic; not in the asserted set |

The same pattern appears, by reading only, in `js_async_generator_completed_return` (21812), `js_promise_try` (56109) and `js_promise_race` (56382).

Two variants are asserted for each tested shape:

1. **Gate stop alone** (test-only `quiet_latch`, so latched polls do not also interrupt). `new RegExp(P)` with the reproducer inside the shape gives zero compiles and latch `ResourceLimit`, but evaluation completes normally, the next statement runs, and a later `.catch` receives the host stop object (`host stop`).
2. **Production configuration**, where the latched handler interrupts at every poll. The shape's body is `expire(); for (;;) {}`, and a host function sets a pending `Timeout`. The engine interrupt (`JS_ThrowInterrupted` 8642, an InternalError with the same mark) fires inside the context and is converted. The poll counter resets to 10,000 operations at each poll (556, 8651), so the next poll again fires inside the next iteration's context. A 1,000-iteration outer loop completes with latch `Timeout` after 1,002 callbacks. Nothing in the engine bounds the iteration count.

Why the facade cannot close it:

- The shapes need no RegExp and no gate. Any host stop, including cancellation and deadlines, is affected.
- Wrapping the global `Promise` does not cover `await`: with `globalThis.Promise = undefined`, `await` still reads a thenable's `then` synchronously, because it uses `ctx->promise_ctor`.
- Removing promises or `await` would contradict RFC 0001's asynchronous exports and add a normative restriction; neither is permitted.
- The host's latch still prevents effects, since services and publication check it and no late result is accepted. But the host thread is inside the engine. It cannot retire the runtime while the runtime executes (see the resource review's retirement section), so the CPU work is not bounded.
- A local patch checking `JS_IsUncatchableError` at these sites, or an upstream change, may well fix this. Both are outside this task's authority; no upstream remedy was researched.

## Calibration at the candidate bound

Worst flag set per family, in ms, debug build; `!` means the compile did its work and then threw a SyntaxError. Flags tried: none, `i`, `u`, `iu`, `v`, `iv`.

| Family | At 4,096 | At 8,192 | Worst flags |
| --- | --- | --- | --- |
| Property classes `\p{L}` repeated | 81.2–81.5 ! (14 without `i`) | 81.6–81.9 ! | `iu`, `iv` |
| Set operations `[\p{L}--\p{N}]` | 45.3 | 91.0 | `iv` (invalid without `v`) |
| Case-folded ranges `[\u0000-\uffff]` | 19.6–22.2 | 39.5–42.4 | any with `i` |
| References + 3-byte padding | 5.7–6.0 | 23.0–23.2 | flag-independent |
| Forward references (reproducer family) | 4.3–4.6 (6–14 on cold runs) | 17.3–18.6 | flag-independent |
| Lookbehind reversal | 0.5 | 1.8 | flag-independent |
| Alternation | 0.35 | 1.2 | flag-independent |
| Nested quantifiers, depth 100 | 0.07 | 0.13 | flag-independent |
| Quantified groups `(?:a)*` | 0.05 | 0.08 | flag-independent |
| Capture groups `(a)*` | 0.03 ! (255-capture limit) | 0.04 ! | — |
| Nested quantifiers, full length | 0.03 ! (early nesting error) | 0.02 ! | — |

Across runs, the worst case at 4,096 units was 81–86 ms, always property classes with case folding and Unicode flags. Those compiles hit an engine limit and error at the same cost at both lengths, so they are capped, not growing. Forward references, padding, lookbehind and alternation grow about 3.5–4× when length doubles, consistent with the audited quadratic paths. Set operations and case folding grow about 2×. The per-compile non-polling slice is therefore finite and small at this bound. This part of the design holds for the pin.

## Matching checkpoint-interval audit

A recording handler that never stops execution measures the wall-clock gap between callbacks (64 MiB heap for this audit only).

| Case | Callbacks | Longest gap | Total |
| --- | --- | --- | --- |
| 200,000 short global matches (`replace`) | 1 | 4.6 ms | 4.6 ms |
| JavaScript loop of 200,000 `test` calls | 81 | 1.4 ms | 108 ms |
| Catastrophic backtracking `/(a+)+b/` on 24 characters | 6,711 | 0.2 ms | 0.99 s |
| Back-reference compares `/^(a*)\1*b/` on 10,000 characters | 22 | 20.6 ms | 78 ms |
| 4,095-character literal run + `b` over 60,000 characters | 13 | 43.4 ms | 0.50 s |

Every case reaches a poll. The matcher counts only backtrack, goto and loop operations. A counted step can therefore carry up to one pattern-length run of character comparisons (bounded by the admission bound), or one back-reference comparison whose length is bounded by the subject string (heap- and string-limited, not by the admission bound). A published matching interval would need a subject-length term. Under Outcome B this does not change the result: an interrupt that does fire can still be absorbed as above.

## Resource-spike hardening defect (recorded, not fixed)

`experiments/quickjs-resource-spike/src/lib.rs` (lines 35–38) evaluates the restricted-global `harden.js`, which is an arrow function expression, **without calling it**. Its realms therefore had dynamic-code hardening only, not the 38-name global surface. The class-bridge, value-boundary and complete-value spikes, and this spike, call it with the allowlists. The resource observations concern CPU, heap, stack and compiler paths, and the native compile needs no global, so the reproducer and its conclusion are unaffected. The resource review's "fresh hardened realm" wording is imprecise for the global surface. Prior experiments and reviews are left unchanged, as this task's scope requires.

## Consequences

- **Pre-admission is a sound bounding technique** for the pinned compiler: finite wrap set, unreachable native, exact-byte literal preflight and a calibrated per-compile slice. It remains useful as defense in depth inside any containment design.
- **In-process stop delivery is the blocker.** On the pinned engine, neither host gate stops nor engine interrupts are guaranteed to end source that uses promise machinery. This also qualifies every earlier in-process CPU enforcement claim for such source.
- The remedy review's ranking already names process containment as the fallback. Its costs remain: IPC for every §6 transfer and service call, per-invocation process or pool lifecycle, and unavailability on iOS, iPadOS and tvOS-class hosts. §10 permits but does not require it ("A realm need not be an OS process").

## Remaining uncertainty

- The swallow-site list comes from reading one revision. Three sites are asserted deterministically in five shapes; the others are by reading only.
- A local engine patch at these sites is plausible and small, but untested and unauthorized. An upstream fix is unresearched.
- Calibration is one debug build on one host. Release timings, other hosts and families outside the audited set are unmeasured.
- The matching interval has an unresolved subject-length term.
- Parser selection and complete module/path preflight remain open; this preflight is an experiment.

## Next task

**Process-containment review** (design level, no implementation): decide whether a per-domain child process that the host kills on deadline, cancellation or resource stop can satisfy RFC 0001 §§4, 5, 6, 7 and 10 without contract changes. It should cover:

- classification from the host's own kill reason;
- mapping lifecycle Model B retirement, late-delivery rejection and scoped resolvers onto process or pool lifecycle;
- the IPC boundary for §6 values and host services, and its trusted-state placement;
- latency and resource costs;
- platforms where helper processes are unavailable (iOS, iPadOS, tvOS), and what a host there could claim;
- whether in-process pre-admission remains required as defense in depth.

It should compare, without authorizing them, the engine-level alternatives: a local check of `JS_IsUncatchableError` at the conversion sites, an upstream change, or another engine. It must not select an engine, patch or upgrade QuickJS, edit the RFC, or implement IPC.

## Validation and audit

The new spike passes **16** tests under the 60-second process-group guard (about 4.6 s; guard not fired); fmt and strict Clippy pass. The blocker runner `python3 experiments/quickjs-resource-spike/probe_regexp.py` exits **2** as preserved: 1,000 / 4,000 / 16,000 references in 6 / 104 / 1,699 ms with zero callbacks, and 64,000 references stopped only by the external five-second safety kill, which is not engine evidence. The 11 resource tests and 8 route-probe tests pass unchanged. All 151 prior Phase 3 tests, 53 runtime tests, 2 declarative harness tests with 34/34 cases, and 6 Python tests pass; runtime build, fmt and strict Clippy pass. Exact commands are in the [completed plan](../plans/completed/2026-09-29-quickjs-regexp-admission-impl-spike.md).

**Unsafe audit:** exactly two `#[allow(unsafe_code)]` functions in `ffi.rs`, each one public C API call with a SAFETY comment; the crate is otherwise `deny(unsafe_code)`. No private API, engine patch or dependency change. Production `runtime/`, `spec/`, `conformance/`, `examples/`, `docs/decisions/` and all prior experiments are unchanged. Nothing was pushed or published.
