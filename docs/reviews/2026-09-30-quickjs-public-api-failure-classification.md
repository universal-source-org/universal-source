# QuickJS public-API failure-classification spike

## Outcome and scope

**Outcome B — the existing public API path does not close the integrated
resource/failure composition gate.** A custom allocator supplies a trusted
rejection reason, but cannot prevent the demonstrated source catch/finally
continuation. The engine heap ceiling can reject before that allocator observes
anything; source can catch it, perform a capability action and publish success.
The unchanged async patch propagates errors already marked uncatchable; it does
not classify or mark these ordinary OOM exceptions.

Baseline: `98c69242d35f79918688b11c3b144300ea121545`, `main`, clean tree, no active
plan. CURRENT_STATE's `222029b` snapshot preceded a documentation commit; this was
not a technical-state conflict. This follow-up extends only the existing
[integrated experiment](../../experiments/quickjs-integrated-resource-failure-spike/README.md).
The [predecessor review](2026-09-30-quickjs-integrated-resource-failure-spike.md),
[resource evidence](2026-09-29-quickjs-resource-spike.md) and
[async termination evidence](2026-09-29-quickjs-async-termination-fix-spike.md)
remain unchanged. RFC 0001 §§4–7/10 remain proposed and unchanged. No production
runtime, engine patch/pin, parser, engine selection, ADR or fork was changed.

## Public API audit

Local evidence uses rquickjs/core/sys 0.14.0 and QuickJS-NG 0.16.2 at the existing
pins. The build driver is unchanged: it verifies original `quickjs.c` SHA-256
`3a6b52a225c21709ef1fd314183efd3af862e867ce4ef8caabcae67922a745b3`, then applies only
the existing 12 async edits, producing
`9a926c4ed02517c84ecfbc9d925b6ceb782b3119f6d05a3d58280c66363e1f3f`.
Cargo metadata, compiled source and unique exported symbols verify the actual
Rust executable's engine. The new tests additionally assert that patched SHA.
No new upstream-version claim or network research is made.

The following are source-audit observations on that pin, corroborated by the
probes below, not calls to private functions. Names/line numbers refer to the
unmodified `quickjs.c` and public `quickjs.h` recorded by the resource review.

| Public control | Actual boundary and limitation |
| --- | --- |
| `JS_NewRuntime2` / `JSMallocFunctions` (header 469–475) | Callbacks receive opaque host state, sizes and pointers, not an OOM exception or failure-reason callback. Returning null permits a trusted Rust latch. No JS calls/reentry occur in the adapter. |
| `JS_SetMemoryLimit` | `js_calloc_rt`, `js_malloc_rt`, `js_realloc_rt` (1995, 2018, 2065) reject before arena/custom allocator entry. No public limit-hit hook or sticky reason is exposed. Memory usage reports current accounting, not a past failed request. |
| `JS_SetInterruptHandler` | A latched allocator refusal requests a stop at the next actual poll. This does not insert a checkpoint before catch/finally; the later-poll probe observes continuation before eventual interruption. |
| `JS_GetException`, `JS_IsError`, `JS_IsUncatchableError` | They inspect an available exception after control returns; a caught exception may never reach the host. Error class/text carries no authenticated OOM origin. |
| `JS_SetUncatchableError` (12049–12070) | Marks one existing Error object, not a runtime-wide future-error policy. It is a no-op on null. The host can mark an escaped OOM Error, but source catch code has already run. |

`JS_ThrowOutOfMemory` (8554) calls ordinary `JS_ThrowInternalError`; it does not
mark the result uncatchable. `JS_ThrowError2` (8435) falls back to `JS_NULL` if
the Error cannot be allocated. `JS_Throw` (7995) replaces the pending exception.
Thus preallocating/marking some other Error does not cause subsequent engine OOM
to use it. Calling JS from an allocator callback is not an established safe
embedding protocol and would not supply the engine's not-yet-created OOM value;
no reentry, private flag, longjmp or unwind-across-C workaround was attempted.
The public flag setter is useful at a host-controlled throw site, as the existing
admission gate proves. It is not an allocation-failure interception hook.

## Executable evidence

Seven new tests in
[failure_classification.rs](../../experiments/quickjs-integrated-resource-failure-spike/src/failure_classification.rs)
cover six integrated diagnostics and one isolated public exception API probe.
The integration uses the same hardened Rust invocation path.
The only invocation knob added is an optional engine heap ceiling at
call entry. Defaults are unchanged. The adapter always retains its finite 4 MiB
budget; the `0` engine-ceiling diagnostic does not allow unbounded allocation.
`note` is deliberately an ungated test witness of source execution; `action` is
the separately gated capability-effect counter. `Value(42)` represents candidate
publication in this evidence harness, not a production Source API envelope.

| Probe | Trusted allocator rejection | Engine heap rejection |
| --- | --- | --- |
| Same 1 MiB `String.repeat`, allocator single-request cap 256 KiB; engine ceiling 8 MiB vs 512 KiB | Rejection count positive; catch marker `77`; action 0; host `Stop(ResourceLimit)` | Rejection count 0; catch marker `77`; action 1; host `Value(42)` |
| 32 MiB request, engine ceiling 0 vs 8 MiB | With engine cap disabled, finite allocator cap catches request; catch still runs, action/publication suppressed | With engine cap enabled, request bypasses allocator, catch returns 42 and action executes |
| Direct, Promise executor, reaction, await, thenable job, async generator | Same-job catch `77` and finally `78` run; executor also reaches `79`; later host boundary discards candidate | Same markers run, action executes and 42 publishes through every route |
| Uncaught OOM in executor/reaction/async body, followed by a separate `.catch` job | Latch prevents pumping the next catch job; no marker/action/publication | Ordinary rejection reaches `.catch`; marker/action/42 all observed |
| Allocator catch followed by `for(;;)` | Marker 77 runs first; existing interrupt callback eventually returns control; no marker 99, action or publication | Not needed for the allocator checkpoint question |

The same 1 MiB request without either constraining override returns 0 with no
catch/rejection, providing a success control. Requests are refused by controlled
limits, not by inducing system OOM. Refusal counts and host-owned latch state,
not exception wording, decide the allocator column.

Ordinary controls throw null, Error with OOM-like text, a forged
`{name, message, code}` object, explicit RangeError and real recursive stack
overflow. Direct catch, Promise rejection handling and await catches retain
ordinary semantics: marker/action/42, zero allocator rejections, no logical
disposal. Uncaught versions remain SourceError. These controls prohibit treating
all errors, all nulls or all RangeErrors as a trusted resource stop.

The public marking test catches a real heap OOM, writes a source marker and
rethrows it. At host return the marker is already 77; the Error's uncatchable bit
changes false→true only when explicitly marked. An ordinary source Error can be
marked the same way. A real engine OOM under a one-byte ceiling produces null,
which remains uncatchable=false even after the public setter. This demonstrates
both the timing limit and the allocation-failure fallback, without source
property inspection as a classifier.

Every integrated probe closes delivery, rejects late completions, drops its
runtime with zero outstanding tracked allocation bytes and verifies destruction.
Every case runs 42 in an independent fresh generation and rejects the stale
token. Trusted allocator outcomes set the experiment's logical-disposal flag;
unclassified caught engine OOM does not, which is part of the blocker. Runtime
retirement itself still occurs for every outcome. Queued jobs get no final drain.

## Validation and limits

- Patched integrated suite: **22 tests passed** (7 new, 15 preserved), with
  `cargo fmt --check`, clippy `-D warnings`, source/linkage provenance and QuickJS
  object/atom leak-abort checks passing.
- Unmodified integrated baseline: its original negative control passes and
  records the swallowed marker 1; patched run records no marker.
- Original admission Promise/async negative control: all five shapes still
  swallow gate and deadline stops 1,000 times on the unmodified pin.
- Unchanged C async-fix harness: **RESULT: PASS**, ordinary exceptions preserved,
  fresh runtimes healthy, ASan/UBSan and QuickJS leak-abort checks clean.
- Unchanged resource suite: **11 tests passed**. Original RegExp compiler
  reproducer: 1,000/4,000/16,000 references return with zero callbacks;
  64,000 references enter then hit the five-second external safety kill (exit 2),
  still with no callback. That expected negative evidence is not engine control,
  teardown success or a passing engine-stop test.
- Local macOS arm64 debug evidence only. No new Rust sanitizer claim: the prior
  integrated Rust sanitizer-link limitation is unchanged and was not retried.
  Full Phase 3, declarative Rust/Python and platform suites were not rerun:
  no shared patch/hardener/value/lifecycle semantics or contracts changed, and
  this is a blocked focused follow-up, not a closure/consolidated review.
- An initial new-test run failed on a mistyped expected SHA literal; corrected
  against the verified generated source, then all tests passed. This was a test
  provenance assertion defect, not an engine result.
- Scoped Markdown structure, 168 relative links and 12 anchors passed, as did
  whitespace checks and the complete diff review. No JSON or contract changed.

## Decision and only next task

1. **Allocator:** trusted host classification and publication suppression yes;
   trusted terminal **with no continuation no** through this API path.
2. **Engine heap:** **no** reliable entry into the same trusted class; its
   refusal precedes the allocator hook and is catchable.
3. **Source continuation:** **yes**, including same-job catch/finally and
   Promise/async paths after allocator rejection. Separate queued jobs can be
   abandoned once the host observes its latch; that does not undo earlier code.
4. **Ordinary semantics:** **preserved**, including actual unclassified stack
   overflow and forged failures.
5. **Further engine work:** **required if pursuing this strict in-process
   no-continuation property on the current candidate path**. The missing boundary
   is authenticated allocation/engine-limit signalling and terminal propagation
   before source catch dispatch, including failure to allocate the Error itself.
   Public post-return marking, heap-cap tuning and polling do not close it.

This is a pinned-path conclusion, not a theorem about every possible embedding.
The integrated resource/failure composition gate remains open; QuickJS remains
a candidate, not a selected engine. No new substantive QuickJS modification was
made or authorized, and no broader resource gate is declared closed.

**Only next task:** a decision-only review of whether to authorize a separately
scoped engine-level allocation/heap terminal-failure experiment, with its minimum
scope and acceptance criteria. Do not implement a patch during that review or
start another selection gate, alternative-engine evaluation, helper-process/IPC
design, production integration or patch-ownership work. This task stops at the
decision point.
