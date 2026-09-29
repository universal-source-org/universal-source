# QuickJS resource-enforcement feasibility

## Outcome and scope

**Outcome B — blocker demonstrated.** The pinned embedding interrupts ordinary JavaScript and tested expensive RegExp matching. It does not poll the interrupt callback during a costly RegExp compilation path. With the callback armed to stop at its first invocation, a local 80,007-byte pattern compiles successfully for approximately 1.64 seconds with zero callback invocations; a 320,007-byte pattern stays inside the engine past a five-second external safety guard, also without a callback. An 8 MiB engine heap limit and 128 KiB stack setting do not stop this computation. This fails the requested credible control of expensive native RegExp work. The outer process kill does not prove engine interruption, classification or safe in-process retirement.

Baseline: `be9120b85f7a1a7664bacb71c2efd240355f8f10`, `test(engine): prove complete value boundary`. Starting tree was clean; no pre-existing active plan. Phase 3 remains active. [RFC 0001](../../spec/rfcs/0001-javascript-execution-binding.md) remains proposed and unchanged. The [experiment](../../experiments/quickjs-resource-spike/README.md) contains eleven bounded tests and a separately contained compiler reproducer. This is not production execution, an engine/parser selection, an ADR or a JavaScript conformance suite.

The task explicitly requires stopping scope expansion on Outcome B. The sections below distinguish tested controls, source-audit findings, preserved lifecycle evidence and unfinished parts of the larger Outcome A matrix. Previous lifecycle, capture, static-analysis/import-attribute, dynamic-code, global, class and complete-value evidence is unchanged.

## Pins, local provenance and platform

rquickjs/core/sys **0.14.0**, binding revision **`d7ef5eeae702fea24c03643064de454f1c1dd4b0`**; vendored QuickJS-NG **0.16.2**, revision **`0fdea21ff1090084e91dad812b343d92e79ba9d9`**. These are the baseline's previously audited pins. The new lockfile is byte-identical to the class-bridge lockfile after replacing only the root package name. Registry VCS metadata confirms the binding revision; the engine header/version declarations confirm 0.16.2. No dependency source was edited, updated or replaced.

This audit reads the installed registry sources. Source references below identify the pinned upstream files for readers; no remote service is involved in the executable probes. The existing provenance reviews remain the record of earlier upstream byte comparisons; this unit does not claim a new remote provenance check.

Local source SHA-256 fingerprints:

| Registry-relative source | SHA-256 |
| --- | --- |
| `rquickjs-sys-0.14.0/quickjs/quickjs.h` | `979127138da79cad5ddc7effa1afb13e8b02f744da2aa36844d4cb9d8cb92de9` |
| `rquickjs-sys-0.14.0/quickjs/quickjs.c` | `3a6b52a225c21709ef1fd314183efd3af862e867ce4ef8caabcae67922a745b3` |
| `rquickjs-sys-0.14.0/quickjs/libregexp.h` | `53ff95a038f7001b752c455d195d998e2955f0ae4ed62a002c13eca798db3ceb` |
| `rquickjs-sys-0.14.0/quickjs/libregexp.c` | `af13e996abb1767fe0cdfda123c45b811d53688d403a2ac47142e97bf79f1fae` |
| `rquickjs-core-0.14.0/src/runtime/base.rs` | `1e5c5e85453096e647867d3978a03298614fa609c243f90270a339c000ae399c` |
| `rquickjs-core-0.14.0/src/runtime/raw.rs` | `e7f70ab2752c2c91fcd36474d326fc24b3e4fd7c346dcc52a01673abb3eb36eb` |
| `rquickjs-core-0.14.0/src/allocator.rs` | `301baa03e63c247b710e3359bfed959195e0649214d0f650b6901d7bc83f88c0` |

Environment: macOS 27.0 build 26A428, arm64, Rust/Cargo 1.96.0; rustc `ac68faa20c58cbccd01ee7208bf3b6e93a7d7f96`, target `aarch64-apple-darwin`. Cargo's development/test native build only. No release-performance, mobile, other architecture, process-RSS, hard real-time or cross-platform guarantee follows. Pinned engine source disables the stack check on WASI; the local stack result must not be generalized to that target.

## Smallest reproducer and compiler diagnosis

The [fixed fixture](../../experiments/quickjs-resource-spike/src/regexp_compile.js) is:

```js
references => new RegExp("\\k<a>".repeat(references) + "(?<a>x)")
```

One thousand references already demonstrate compilation without any interrupt callback. The larger 64,000-reference case is the smallest size in this four-point sweep that exceeds the chosen five-second guard; no globally minimal byte count is claimed. A standalone equivalent is `new RegExp("\\k<a>".repeat(64000) + "(?<a>x)")`. It uses ordinary permitted string operations and named forward references. There is no match operation, service, dynamic JavaScript compilation, import, source rewriting or injected checkpoint.

The host builds a fresh hardened realm, installs the public interrupt handler, compiles the fixed function while unarmed, prints/flushed `ENTER`, arms a host-only flag, and calls the function. Any armed callback increments a host counter, latches Timeout, prints `INTERRUPT_CALLBACK` and returns true. The returned temporary RegExp is never published through a Source API result. The diagnostic intentionally enters with a request already armed to establish whether the engine polls. A production host would check before entry as well; this probe does not allege a before-entry policy bypass. Its absence of compiler polling also applies to a cancellation/deadline becoming pending during those native scans.

Final local run:

| Forward references | Pattern bytes | Observation |
| --- | --- | --- |
| 1,000 | 5,007 | Returned successfully in 18 ms; 0 callbacks. |
| 4,000 | 20,007 | Returned successfully in 116 ms; 0 callbacks. |
| 16,000 | 80,007 | Returned successfully in 1,638 ms; 0 callbacks. |
| 64,000 | 320,007 | ENTER recorded; no callback/RETURN; external five-second kill, runner exit 2. |

Timing varies with build/hardware/load. The claim is a demonstrated long polling gap, not mathematical nontermination or a universal five-second requirement in the RFC. Completed cases independently prove the missing polling path. The unsuccessful large case strengthens the local resource-control counterexample; it is not counted as a passing engine test. Post-return memory usage printed by the binary is after temporary values are dropped and does not measure peak usage.

The pinned [RegExp source](https://github.com/quickjs-ng/quickjs/blob/0fdea21ff1090084e91dad812b343d92e79ba9d9/libregexp.c) explains it:

- `re_parse_term`, in the named-backreference branch, cannot find the later capture in the groups parsed so far. It calls `re_parse_captures` to locate it and again to emit its group index.
- `re_parse_captures` (local line 1729) scans from the beginning to the end of the pattern. Repeating a forward reference produces repeated full-pattern scans: quadratic work for this family with linear input/bytecode storage.
- `lre_compile` (2577) and this scan have no timeout/interrupt polling. Stack checks on parser recursion do not constrain this flat pattern's repeated scans.
- Matching is different: `lre_poll_timeout` (2798) checks every 10,000 relevant backtracking/loop checkpoints. Its `lre_check_timeout` bridge in `quickjs.c:49605` invokes the runtime handler. `LRE_RET_TIMEOUT` becomes `JS_ThrowInterrupted` in the JS wrapper. That working matching path says nothing about compilation.

## Public capabilities and missing control

All directly used controls are safe rquickjs methods over functions in the [public QuickJS header](https://github.com/quickjs-ng/quickjs/blob/0fdea21ff1090084e91dad812b343d92e79ba9d9/quickjs.h). The [binding runtime API](https://github.com/DelSkayn/rquickjs/blob/d7ef5eeae702fea24c03643064de454f1c1dd4b0/core/src/runtime/base.rs) documents the closures and limits. Internal C functions are read to explain observations; the experiment never calls them or accesses their structs.

| Public API | Use and limitation |
| --- | --- |
| `Runtime::new` / `Context::full` → `JS_NewRuntime`, `JS_NewContext` | Dedicated runtime/queue domain per realm; all intended intrinsics precede unchanged hardening. |
| `Runtime::set_interrupt_handler` → `JS_SetInterruptHandler` | Installs a boxed owner-thread closure. Returning true requests an uncatchable engine interruption at actual polling sites; does not create polling sites in compiler/native code. |
| `Runtime::set_memory_limit` → `JS_SetMemoryLimit` | Runtime-wide accounted allocation ceiling; 8 MiB here, zero would disable it. No public limit-hit notification/latch is installed by this API. |
| `Runtime::set_max_stack_size` → `JS_SetMaxStackSize` | 128 KiB relative stack setting. rquickjs updates the stack top through public `JS_UpdateStackTop` on context entry. No public stack-limit-hit reason callback. |
| `Runtime::memory_usage` → `JS_ComputeMemoryUsage` | Reads accounted statistics after returning to the owner; does not identify a past rejected allocation or peak/RSS. |
| `Module::declare` / `Module::eval`, `Ctx::eval`, `Function::call` → `JS_Eval`, `JS_EvalFunction`, `JS_Call` | Fixed synthetic code construction, initialization and execution. No package loader/capture implementation is added. |
| `Ctx::execute_pending_job` → `JS_ExecutePendingJob` | Explicit bounded job stepping. This convenience binding clears failed-job exception state; the trusted stop latch, not its bool or exception text, decides interruption. |
| `Promise::state` / `Promise::result` → `JS_PromiseState`, `JS_PromiseResult` | Direct settlement inspection; no `.then` observer or extra turn. |
| `Ctx::catch` → `JS_GetException`; scoped Value release → `JS_FreeValue` | Release pending exception handles without formatting or inspecting source properties. No uncatchability reset or resumed source execution. |
| Context/Runtime destruction → `JS_FreeContext`, `JS_FreeRuntime` | Drop every scoped/rooted owner; final runtime destruction discards queued arguments and runs internal GC without executing jobs. `WeakRuntime::try_ref` verifies destruction in selected probes. |

`Function::new` and safe global property installation create only test markers; the unchanged hardeners use ordinary public object operations. There is no source-visible clock, interrupt-control object or new service. `Runtime::run_gc` / `JS_RunGC` and `set_gc_threshold` / `JS_SetGCThreshold` were reviewed but not explicitly called or tuned. Normal engine GC and the GC inside teardown remain enabled. `JS_NewRuntime2`/`JSMallocFunctions` and rquickjs's allocator trait were reviewed, not newly implemented. No custom allocator feature is enabled; the binding's caveat about custom allocators therefore does not affect the demonstrated default-limit path.

No second documented compiler-cancellation API was found in the pinned public header. Public allocator callbacks can refuse allocations and retain trusted rejection state, as the earlier lifecycle spike demonstrates. They are allocation callbacks, not callbacks for each compiler scan. Geometrically grown buffers and repeated scans can execute without a new allocation. A particular allocator/pattern-size policy might reduce that interval, but no such complete policy or bound has been demonstrated; it cannot be silently counted as an already proven solution. Heap and stack caps alone demonstrably do not make this compiler path honor the installed stop hook. Calling GC/destruction concurrently with an active context, private flags, signal tricks and engine patches are not valid uses of the audited interface.

Closing this direct engine-hook gap would require compiler polling (and safe error propagation), likely through an upstream engine change/version or patch; neither was attempted. Another containment or explicitly bounded RegExp policy would require a separately scoped design/evidence review. This is not a proof that every conceivable restrictive allocator policy is impossible. It is a concrete blocker to claiming the requested resource boundary from the tested pinned public controls. The RFC need not be weakened: the candidate may remain unsuitable. If a future proposal instead relaxes stop behavior or changes allowed RegExp behavior, it must explicitly reconsider the RFC rather than treating this experiment as acceptance.

## CPU, trusted reasons and initialization

Eleven unit tests are in [tests.rs](../../experiments/quickjs-resource-spike/src/tests.rs). CPU tests arm only after compiling the fixture and stop on the **second** engine callback. The callback increments an owner-thread counter, latches exactly one of `Cancelled`, `Timeout`, or `ResourceLimit`, and keeps returning true once latched. These are deterministic simulated host decisions, not instruction counts or elapsed deadline measurements. Host monotonic clocks/atomic cancellation state could be read at the same callback, as already exercised in the preserved lifecycle suite. Source cannot access this Rust state.

`while(true)`, infinite `for`, a billion-iteration finite loop, repeatedly evaluated depth-12 Fibonacci recursion, and arithmetic/string/object work all return an engine error after two callbacks for each host reason. A catch that would return success, a finally marker, a queued Promise marker, and nested catches attempting to continue forever never execute after CPU interruption. The source implementation sets an uncatchable error; tests establish actual return and zero effects. A forged thrown `{code, message, name}` leaves the latch unset. Engine message/class text is never used to classify it.

Pinned bytecode sites decrement an internal 10,000-count poll counter. This is an implementation detail, not a public promise of one callback per 10,000 instructions or a wall-time bound. Native work, parsing and GC cannot be assumed to have the same coverage. The callback must be small, non-panicking and avoid JS reentry. Tests keep `Runtime`, `Context`, closures and handles on the owner thread with `Rc<Cell<_>>`; real cross-thread signals must use host atomics/messages, not move these non-Send handles. No futures/parallel integration is tested.

A compiled module with an infinite initialization loop is evaluated in separate load-validation and ready-call domains. The host checks trusted state before readiness/dispatch: it produces the test-private loading-interruption diagnostic in the former and Timeout in the latter. The operation's native marker remains zero and neither path becomes ready/called. Realm destruction precedes a fresh health probe. These are host-phase primitives, not new portable loading representations or production logical-instance implementation. Complete export capture remains covered by its unchanged earlier suite.

## Memory and stack findings

Retained ordinary objects, arrays, a 32 MiB string request, nested records, closures, a retained Promise chain, Map and Set reach the 8 MiB configured limit and return an error to Rust. Each run asserts that a separate 100-callback CPU safety guard did not fire, reads finite accounted usage, releases values, verifies the weak runtime is dead, and runs 42 in a new hardened runtime. The slack in the usage assertion is test tolerance, not a promised quota-overhead constant. No system OOM is induced.

The pinned `js_malloc_rt`, `js_calloc_rt` and `js_realloc_rt` check runtime-wide accounted bytes before calling the allocator. Relevant engine objects, strings, property tables, function/Promise records and RegExp bytecode/backtracking buffers go through this accounting. Actual usable-size rounding and overhead are added after successful allocation, so this is not exact byte-for-byte reserved memory. Arena backing storage/unused slots and allocator metadata can differ from live accounting. Initial runtime allocation, native/Rust call stacks, Rust allocations for source buffers/handles/host copying, libc internals and unrelated process allocations are not comprehensively bounded by this setting. It is not per-context and not RSS. A conservative engine cap plus process headroom is useful demonstrated containment, not arbitrary process allocator failure recovery.

Automatic GC affects when otherwise unreachable allocations disappear; exact iteration-to-OOM is not asserted. An early draft fixture retained only the end of an unresolved Promise chain and did not reliably grow live memory. It reached external guards at 25 seconds and, during isolation, 8 seconds. That was a bad heap-growth fixture, not evidence that the heap ceiling failed: unrooted graphs may disappear. The final fixture explicitly retains every Promise, includes a CPU guard, and passes. These development timeouts are reported rather than hidden.

A source catch can convert the deliberately oversized string allocation failure into `42` with no host interrupt latch. Default `JS_SetMemoryLimit` and post-return memory statistics therefore do **not** alone provide trustworthy exhaustion classification. The earlier public custom-allocator rejection latch remains relevant; full runtime-wide OOM classification/latched stop behavior was not completed after Outcome B. Error allocation itself can fail; neither an InternalError string nor null exception is a reliable host reason. Actual process allocator failure/abort is outside the recovery promise and was not tested.

Unbounded recursion and a 1,000-function synchronous call chain return errors under the 128 KiB stack setting; independent fresh runtimes then run normally. A caught real recursive overflow and an explicitly thrown `RangeError` both satisfy the same `instanceof RangeError` check. `JS_ThrowStackOverflow` creates an ordinary RangeError; no public hit latch or authenticated subtype is exposed by the examined controls. This establishes bounded local stack growth, **not trustworthy RESOURCE_LIMIT classification**. RFC §7's unclassified `SOURCE_ERROR` fallback remains intact; it does not permit classifying all RangeErrors as resource exhaustion. The full initialization/job/getter recursion matrix is unrun in this stopped unit.

## Allocation before §6 checks

Source inspection confirms that `JS_GetOwnPropertyNames` allocates its property-enumeration array through `js_malloc`; `JS_ToCStringLenUTF16` may materialize a wide buffer through `js_alloc_string`. Those native buffers use the runtime allocation path, before the converter can inspect the resulting length. A descriptor value is ordinarily duplicated, while atom/key creation and host collection may allocate. Module parsing/compilation also uses engine allocations, but runtime limits do not measure all binding/Rust input buffers or impose compiler CPU checkpoints.

This is a source-audited backstop for those engine allocations, not the requested complete adversarial conversion/OOM proof. The many-own-key, dense-array, huge-key and string-extraction exhaustion matrix was not added after the blocker. The existing complete-value tests pass unchanged; their structural budgets still do not bound pre-enumeration native work or process memory. RFC §6 and its prior proof are not redesigned or weakened.

## Promise pressure, retirement and publication

Long loops inside a Promise reaction and after an await trigger the same trusted resource latch while explicitly stepping jobs. The operation does not resolve successfully. Pending-empty-queue, recursively scheduled reactions and async recursion remain Pending after at most 32 steps; the host abandons the domain without settling them or draining remaining jobs. The retained Promise graph independently exhausts the engine heap. A job count is insufficient for a single running infinite reaction, and an empty queue is not successful completion. No generic Promise cancellation is introduced.

The common retirement discipline is the existing Model B: stop job pumping and native delivery, revoke scope, release scoped values/resolvers/errors/module handles, drop Context and Runtime owners, then decide/publish only host-owned data. The new CPU probes show queued/finally/catch markers remain zero across destruction. Heap/stack probes show recoverable return/destruction and fresh health; they do not claim authenticated failure reasons or source cleanup for every possible native object. Source-visible finalization facilities are absent after the unchanged hardening.

The candidate test constructs either a fixed synchronous record or an already settled native Promise, extracts only the known fixture's boolean, observes host timeout/cancellation during the cleanup interval, destroys the realm, and selects the stop reason instead of success. Its ordinary property read is explicitly not a §6 converter. It demonstrates candidate precedence, not every conversion error or a new D1 total ordering. The prior lifecycle suite preserves the stronger root/callback/drop and publication-boundary evidence, including logical disposal after execution-side termination.

The unchanged `late_completions_and_old_resolvers_cannot_enter_next_generation` test reruns for success/cancel/timeout/resource labels: closing delivery removes the resolver; stale generation messages cannot settle old or next-realm Promises, and a matching new generation still works. Actual CPU interruption, explicit allocator rejection and late delivery remain separately tested primitives. No new integrated provider-after-every-failure matrix is claimed. The compiler case never returns before the watchdog kill, so its in-process revocation/retirement and subsequent same-process health are **not established**. One cannot safely destroy a runtime while its owning thread is still executing it.

## Comparison controls and remaining limits

| Insufficient mechanism | Evidence or limit |
| --- | --- |
| Deadline checked only before/after eval/call | The compiler remains inside the call with no callback; a later check can reject publication but cannot create an in-run checkpoint. The candidate test still requires the final check. |
| External timeout alone | Compiler runner exits 2 after killing the process; no engine return, delivered result or in-process cleanup is credited. |
| Promise/job count alone | A single reaction/await continuation can loop forever until the interrupt hook fires. Empty-queue pending returns need a host deadline/cancel policy. |
| Structural value budgets alone | Heap fixtures can exhaust memory before any candidate exists; native enumeration/extraction can allocate before budget inspection. Full boundary OOM follow-up remains open. |
| Exception name/text/code alone | Forged interruption record leaves trusted state unset; caught OOM can return 42; ordinary RangeError matches real overflow's ordinary class. |
| System OOM | Deliberately not induced. Engine cap is set far below normal process headroom; neither Rust abort recovery nor machine-wide OOM is proven. |
| RegExp uses bytecode checkpoints | Matching has a separate hook, compilation has none in the reproduced path. Matching success does not establish compiler control. |

Outcome A is not justified. Unfinished portions include complete allocator rejection/termination classification, native conversion exhaustion, module allocation/stack variants, all job allocation/stack variants, complete code-parser/native arithmetic checkpoint coverage, teardown latency, and integrated delivery after each failure type. No claim covers private API changes, source instrumentation, release timing or other platforms. The finite example is not asserted to run forever; the missing usable checkpoint is the demonstrated problem.

**Unsafe audit:** zero new unsafe blocks, zero unsafe functions and zero unsafe impls. Both the library and standalone binary use `forbid(unsafe_code)`; tests inherit it. No manual C call, private structure/global, numeric class constant, bytecode patch, allocator override or signal against engine internals exists. POSIX SIGKILL applies only to the contained local test process group as the external safety net. Prior experiments' separately audited unsafe code is unchanged. Miri is not used to claim validation of native C.

## Validation and next unit

Eleven new unit tests pass. All **151** prior Phase 3 tests/checks pass unchanged, including 20 complete-value, 10 class-bridge, five compile-fail checks and the expected-panic Boa control. Runtime: **53** tests; declarative rerun: **2** harness tests and all **34** cases; Python: **6** tests. Runtime build/fmt/strict Clippy and new-crate fmt/strict Clippy pass. The compiler probe is deliberately separate and reports an external timeout, never an engine success. Exact commands, development/final guard results and documentation/preservation checks are in the [completed plan](../plans/completed/2026-09-29-quickjs-resource-spike.md).

Smallest justified next task: **review a specific remedy for RegExp compilation interruption/containment against this reproducer and unchanged RFC**, before reopening the complete resource matrix. That review must distinguish an actual documented mechanism from a proposed upstream fix, version change or different containment architecture. This result authorizes none of those implementations or an engine selection. Module/path containment and final parser/preflight strategy remain later gaps; a consolidated selection review is premature. No production dependency, RFC, ADR or prior evidence changed, and nothing was pushed or published.
