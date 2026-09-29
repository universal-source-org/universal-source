# QuickJS-NG async/Promise uncatchable-stop termination fix and version review

## Outcome and scope

**Outcome B — a bounded engine change completely fixes the swallow for the RFC 0001
language profile, and no published upstream QuickJS-NG revision contains that fix.**

The prior [process-containment review](2026-09-29-process-containment-and-termination-review.md)
(Outcome C) named the remaining in-process termination gap — several Promise/async
internals convert an uncatchable host stop into an ordinary Promise rejection, so source
keeps running — and hypothesised it was one small, enumerable engine defect. This review
tests that hypothesis with an isolated throwaway engine experiment and settles the two
independent questions it raised:

1. **Is the defect completely fixable by a bounded engine change on the pinned revision?**
   Yes, for every route reachable under RFC 0001. Twelve edits across eleven functions,
   each reusing the engine's own `JS_IsUncatchableError` guard (the pattern
   `promise_reaction_job` already carries), make every uncatchable route terminate in
   process while ordinary exceptions stay catchable, with no leaks and no sanitizer errors.
2. **Does a published upstream QuickJS-NG revision already contain it?**
   No. Current upstream `master` (0.17.0) is byte-identical to the pin at every affected
   conversion site; it carries only the same partial coverage the pin already has.

This is a design and evidence review with a **throwaway, generated, gitignored** patched
engine used solely to gather evidence. Per the task it does not change production
`runtime/`, commit or patch the vendored dependency, update pins, update rquickjs, select
an engine, accept RFC 0001, create an ADR, implement process containment, modify the
portable contract, or authorise a fork. All prior blocker documents are unchanged and
both historical reproducers still reproduce (validated below).

Baseline: `304818de0619555e05b85184619509e3da38a734`
(`docs(engine): review process containment and async termination`), clean tree, no other
active plan. Phase 3 remains active. [RFC 0001](../../spec/rfcs/0001-javascript-execution-binding.md)
remains proposed and unedited; no engine or parser is selected; no ADR exists.

## Pins and provenance

rquickjs/core/sys **0.14.0** (`d7ef5eeae702fea24c03643064de454f1c1dd4b0`),
QuickJS-NG **0.16.2** (`0fdea21ff1090084e91dad812b343d92e79ba9d9`), unchanged.
`rquickjs-sys-0.14.0/quickjs/quickjs.c` has SHA-256
`3a6b52a225c21709ef1fd314183efd3af862e867ce4ef8caabcae67922a745b3` (64906 lines), equal to
the resource, admission and containment reviews; line numbers below refer to it. The
experiment driver re-verifies this SHA before every run and aborts on any mismatch.
Environment: macOS 27.0 (26A428), arm64, Apple clang 21.0.0, Rust/Cargo 1.96.0.

## On external access

The task permits public upstream research for the version question only (Part 7). All
engine-fix evidence (Parts 1–6) comes from the local pinned source and the local
throwaway build; only Part 7 fetched public upstream QuickJS-NG source and history. No
accounts, credentials, private infrastructure or third-party targets were touched. The
prior review recorded a broad Apple-platform claim about helper processes; that claim is
**not** relied on here — Apple provides system-managed extension/XPC and specialised
multi-process mechanisms (e.g. ExtensionFoundation, BrowserEngineKit), so the case for an
engine-level fix rests on its own merits (it runs in process on every platform), not on an
absolute containment-impossibility assertion. This task is not a platform review and does
not redesign around those mechanisms.

# Part 1 — Exhaustive conversion-site enumeration

## Method and completeness argument

An exception becomes an *ordinary* Promise rejection only by **materialising** the pending
exception with `JS_GetException` and handing it to a reject function or to
`fulfill_or_reject_promise`. No conversion path avoids `JS_GetException` (verified: every
other read of `rt->current_exception` in the file is infrastructure — the guard itself,
`build_backtrace` save/restore, `JS_SetPropertyValue` clearing, module sync-evaluation
propagation, or the interpreter's own unwind — none of which fabricates a rejection). The
audit therefore enumerates **all 36 `JS_GetException` call sites** and classifies each.
Because every rejection conversion must pass through one of these 36 sites, enumerating
them is exhaustive for the swallow class.

Independently, `JS_IsUncatchableError` is referenced in only **five** places in the whole
file — its definition (12049) and four guards: interpreter unwind (20901),
`js_async_function_resume` (21482, 21491), and `promise_reaction_job` (55584). Every other
conversion site omits the guard. That five-reference fact is the second, independent proof
that the guarded set is small and the unguarded set is exactly "all conversions minus these
four".

## Reachable swallow sites (fixed)

All reachable under RFC 0001 (ES2023 baseline; async generators and `for await` are
permitted; `.then`-handlers and async-function bodies are already guarded upstream).

| Function | Conv. line | Route | Guard omitted? | Uncatchable must propagate | Owned resources | Propagation |
|----------|-----------:|-------|:--------------:|:--------------------------:|-----------------|-------------|
| `js_promise_constructor` | 55916 | executor throw | yes | yes | `args[0]`,`args[1]`,`obj` | `goto fail` |
| `js_promise_resolve_function_call` | 55801 | thenable `then` getter | yes | yes | (none live at label) | `return JS_EXCEPTION` |
| `js_promise_resolve_thenable_job` | 55686 | thenable `then` call | yes | yes | `args[0]`,`args[1]` | skip convert; existing tail frees + returns `res` |
| `js_promise_all` (all/allSettled/any) | 56241 | iterator/element throw | yes | yes | funcs, iter, values, … | `goto fail`→`done` |
| `js_promise_race` | 56382 | iterator/element throw | yes | yes | funcs, iter, … | `goto fail`→`done` |
| `js_promise_try` | 56109 | callback throw | yes | yes | `resolving_funcs[0/1]`,`result_promise` | free then `return JS_EXCEPTION` |
| `js_async_generator_resume_next` | 21891 | body resume throw | yes | yes | (none; value not yet taken) | `return;` (void) |
| `js_async_generator_next` | 22008 (call) | caller of void helper | — | yes | `promise` | free then `return JS_EXCEPTION` |
| `js_async_generator_resolve_function` | ~21970 (call) | await-resume caller | — | yes | (none) | `return JS_EXCEPTION` |
| `js_async_generator_completed_return` | 21812 | `.return()` resolve | yes | yes | (`promise`==EXC) | `return -1` (mirrors existing error path) |
| `js_async_from_sync_iterator_next` | 56770 | `for await` sync next/getter | yes | yes | `resolving_funcs[0/1]`,`promise` | free then `return JS_EXCEPTION` |
| `js_async_from_sync_iterator_next` | 56744 | sync `IteratorClose` | yes | yes | `resolving_funcs[0/1]`,`promise` | free then `return JS_EXCEPTION` |

Ten conversion sites need a guard (across nine functions); two additional functions
(`js_async_generator_next`, `js_async_generator_resolve_function`) do not convert but must
**propagate** because the conversion helper they call is `void` — see Part 4. Total: **12
edits across 11 functions**.

## Already-correct guards (regression anchors, unchanged)

`promise_reaction_job` (55584), `js_async_function_resume` (21482, 21491) and the
interpreter unwind (20901). These carry the exact pattern the fix replicates; the
experiment proves the patch does not regress them.

## Out-of-profile conversion sites (documented, not patched)

Unreachable under RFC 0001, each fixable by the identical one-line guard if the profile
ever expands: the explicit-resource-management paths `js_async_iterator_proto_dispose`,
`js_dispose_resources`, `js_async_dispose_step`, `js_disposable_stack_dispose` and the
`OP_using`/`OP_await_using` interpreter handlers (`using`/`await using`/DisposableStack are
outside the §9 allowlist), and the dynamic-`import()` paths `js_load_module_fulfilled`,
`JS_LoadModuleInternal`, `js_dynamic_import_job`, `js_dynamic_import` (dynamic import,
`import.meta` and top-level `await` are forbidden syntax). The remaining `JS_GetException`
sites are not conversions (definition, backtrace, property-clear, a freshly-created
`TypeError`, module sync-eval propagation, callsite clearing).

# Part 2 — The isolated throwaway experiment

`experiments/quickjs-async-termination-fix-spike/` holds a driver (`build_and_run.py`) and
a pure-C harness (`harness.c`). The driver **never** touches the committed dependency: it
SHA-verifies the pinned source, copies it into gitignored `build/baseline/qjs` and
`build/patched/qjs`, applies the guard edits **only** to the patched copy (asserting the
exact replacement count per rule; 12 total), confirms the baseline copy is byte-identical
to the pin, and builds `harness.c` against each engine with `-fsanitize=address,undefined
-fno-sanitize-recover=all` plus QuickJS's own `JS_DUMP_LEAKS | JS_ABORT_ON_LEAKS` (a
portable object/atom/memory leak check; LeakSanitizer is unavailable on macOS/arm64).

Each edit reuses `if (unlikely(JS_IsUncatchableError(ctx->rt->current_exception)))` *before*
`JS_GetException` and routes the latched uncatchable back through each site's existing
cleanup — a shared `fail`/`done` label where one exists, otherwise the site's own owned
values. This is the smallest pattern consistent with `promise_reaction_job`.

The harness models the host stop **both** ways the real system produces it: `stop()` — a
native throw marked uncatchable, exactly the RegExp pre-admission gate's shape
(`JS_ThrowInternalError` + `JS_SetUncatchableError`); and `expire(); for(;;){}` — a latched
deadline interrupt (`JS_ThrowInterrupted`), the shape for Timeout/Cancel/ResourceLimit.

# Part 3 — Regression matrix (baseline vs patched)

Every route is run in a fresh runtime in two classes: an **ordinary** exception (must stay
catchable / reject normally) and an **uncatchable** host stop (must never become an
ordinary rejection). `SWALLOWED` = source continued (a `.catch`/`.finally`/next statement
ran); `TERMINATED` = source did not continue **and** control returned to the host with the
uncatchable latched; `CAUGHT` = ordinary rejection delivered. No outer watchdog is used.

Result — `RESULT: PASS`:

- **Native-gate mechanism** (`stop()`): executor, resolve-thenable-getter,
  thenable-then-call, `Promise.all`/`race` iterator, `Promise.try`, async-generator resume,
  async-generator await-resume, `finally`, and the RegExp-admission-gate shape all flip
  **SWALLOWED → TERMINATED**.
- **Latched-deadline mechanism** (`expire(); for(;;){}`, the preserved reproducer's exact
  five shapes): executor, resolve-getter, `Promise.resolve` getter and `Promise.all`
  iterator flip **SWALLOWED → TERMINATED**; the `await` getter already terminated in both
  (it exits through the pre-existing `js_async_function_resume` guard under the single-raise
  model).
- **Already-guarded anchors** (`.then` handler throw, async-function body throw,
  `await` thenable getter, `for await` sync iterator): **TERMINATED in both** — unregressed.
- **Every ordinary route**: **CAUGHT in both** — ordinary semantics preserved.
- **Fresh independent runtime**: healthy after all stop cases (`HEALTH ok`).

For each uncatchable case the harness proves the required properties: `cont()` is never
called (source after the stop does not execute; a rejection handler never receives the host
stop; a `finally` cannot turn it into success; no later success candidate resolves);
control returns to the C host with the uncatchable observable via `JS_IsUncatchableError`;
the runtime is then destroyed cleanly (Model B retirement); and a subsequent fresh runtime
evaluates normally.

# Part 4 — Cleanup correctness (mandatory)

For each modified site the patched propagation path was compared to that site's existing
normal-rejection cleanup, and ownership was preserved exactly:

- **Shared-label sites** (`js_promise_constructor`, `js_promise_all`, `js_promise_race`)
  jump to the function's own `fail:`/`done:` label, which already frees every owned value
  and returns `JS_EXCEPTION`. The materialised exception value (`ret`/`error`) is
  `JS_EXCEPTION` on these paths, so nothing extra needs freeing.
- **Tail-reuse site** (`js_promise_resolve_thenable_job`) skips only the reject conversion
  and falls through to the pre-existing `JS_FreeValue(args[0]); JS_FreeValue(args[1]); return
  res;` tail (with `res == JS_EXCEPTION`) — identical cleanup, one branch shorter.
- **Explicit-free sites** (`js_promise_try`, `js_async_from_sync_iterator_next` ×2) free
  exactly the values that site owns at the propagation point (`resolving_funcs[0/1]`, the
  result/capability promise) and no more; `method`/`err` are either `JS_EXCEPTION` or
  already freed on every path reaching the label.
- **Return-code site** (`js_async_generator_completed_return`) returns `-1` with
  `promise == JS_EXCEPTION`, byte-for-byte the same state as the existing `return -1` a few
  lines below, so no new ownership is introduced.
- **Void-helper + callers** (`js_async_generator_resume_next` and its two callers): the
  helper returns before taking the exception; the callers free the capability `promise` and
  return `JS_EXCEPTION`; the enqueued request remains owned by the generator and is freed by
  its finalizer.

Crucially, **no propagation path calls `JS_GetException` on the uncatchable** — the pending
exception is preserved for the host, satisfying the "no premature consume / no stale
exception after the host regains control" requirement.

This is not asserted from "tests did not crash." Every one of the 30+ fresh runtimes was
torn down under AddressSanitizer + UndefinedBehaviorSanitizer with
`-fno-sanitize-recover=all` **and** QuickJS `JS_DUMP_LEAKS | JS_ABORT_ON_LEAKS`; the driver
additionally requires a `FREED <case>` line after each verdict, so any leak-triggered abort
during `JS_FreeRuntime` is attributed to its case. The full run is clean: no use-after-free,
no undefined behaviour, no object/atom/memory leak, across both engines.

# Part 5 — Completeness search

Beyond the source enumeration (Part 1), the harness itself is a completeness instrument: a
route that stops swallowing but does **not** return control to the host surfaces as
`UNRESOLVED`. This is exactly how the async-generator gap was found — with only the site-7
guard, `async_gen_resume__uncatchable` reported `UNRESOLVED`, revealing that the void
helper's callers discard the pending exception. After adding the two caller-propagation
edits (7b, 7c), **zero** `UNRESOLVED` verdicts remain. Combined with the exhaustive
`JS_GetException` enumeration and the five-reference `JS_IsUncatchableError` fact, the
search concludes **Complete** for the RFC 0001 language profile: no other reachable
swallowing route exists. The only remaining conversion sites are the enumerated
out-of-profile ones, which are unreachable and individually documented.

# Part 6 — Preserved resource evidence (re-run)

Both historical blocker reproducers still run against the **pinned** engine and are
unchanged (the fix lives only in the throwaway build; re-pointing the committed dependency
is disallowed, and the isolated harness is the sanctioned patched-engine evidence):

- **Promise-swallow reproducer** (`experiments/quickjs-regexp-admission-impl-spike`,
  `promise_machinery_swallows_gate_and_deadline_stops_in_process`, N=1000): **passes**, i.e.
  it still demonstrates the swallow on the pinned engine for all five shapes
  (`gate and deadline stops swallowed 1000 times`). The isolated harness shows those same
  shapes **TERMINATED** under the patch.
- **RegExp-compile reproducer** (`experiments/quickjs-resource-spike`, `probe_regexp.py`):
  exits **2** with `callback_count=0` at 320 007 bytes — still **non-polling**. As expected,
  the patch does **not** touch RegExp compilation: the architecture keeps RegExp bounded via
  pre-admission and general stops correct via the corrected uncatchable propagation. These
  are orthogonal; this experiment does not regress the RegExp story.

# Part 7 — Upstream QuickJS-NG version research

Public upstream source and history were inspected directly. Current `master`
(`QJS_VERSION` 0.17.0, ahead of the pinned 0.16.2) contains `JS_IsUncatchableError` in the
**same five places** as the pin: definition (12054), interpreter unwind (20929),
`js_async_function_resume` (21510, 21519), `promise_reaction_job` (55629). Every conversion
site this review fixes is **still unguarded upstream** and byte-identical to the pin:

- executor conversion (master 55959–55962): naked `JS_GetException` → reject;
- `js_promise_all`/`race` `fail_reject` (master 56285–56287, 56428): naked;
- `js_promise_resolve_thenable_job` (master 55732): naked;
- `js_async_generator_resume_next` (master 21918–21921): naked.

The defect is publicly documented — [bellard/quickjs#341](https://github.com/bellard/quickjs/issues/341)
("Uncaughtable error seems to be caughtable when it occurs inside a Promise") describes the
exact `JS_ExecutePendingJob` swallow and proposes returning an error instead of calling the
resolve funcs when `JS_IsUncatchableError`; a commenter explicitly notes the void helper's
caller (`js_async_function_resolve_call`) "should change too", independently confirming this
review's async-generator finding. QuickJS-NG's own `api-test.c` `async_call` test asserts
the **async-function** path terminates (the guard it already ships), but there is no test or
guard for the Promise-combinator, executor, thenable-job or async-generator paths.
[PR #745](https://github.com/quickjs-ng/quickjs/pull/745) (merged 2024-12) only *exposed*
the `JS_IsUncatchableError` API; it added no guards.

**Conclusion (stated explicitly, not speculated): no published upstream QuickJS-NG revision
after `0fdea21f` — through current `master` / 0.17.0 — contains an equivalent fix for the
sites this review identifies.** Upgrading the engine would not remedy the defect.

# Part 8 — Candidate version evaluation

Not applicable: Part 7 found no upstream fix, so there is no candidate version to adopt for
this defect. For completeness: the version distance is one minor (0.16.2 → 0.17.0); adopting
it is out of scope (pins are frozen) and, critically, would **not** fix the swallow, so a
version bump is not a remedy here.

# Part 9 — Maintained-fork surface (no upstream fix exists)

Recorded for the next consolidated review; **the fork is not authorised**.

- **Changed sites:** 12 edits across 11 functions, all in the Promise/async-generator
  internals of a single translation unit (`quickjs.c`).
- **Patch size:** ~40 added lines; each edit is a small guard reusing an existing helper
  and existing cleanup — no new functions, no signature changes, no data-structure changes.
- **Clean apply:** yes — every anchor matched its exact expected occurrence count against
  the pinned SHA.
- **Merge-conflict exposure:** low. The affected functions are byte-identical between the
  pin (0.16.2) and current `master` (0.17.0), so these regions have not churned; a rebase
  onto a future release would likely apply with minimal conflict. Risk concentrates in the
  async-generator resume/caller trio, which is the most intricate region.
- **Regression suite:** the throwaway harness (this experiment) plus the two preserved
  reproducers give a ready fork-validation battery.
- **Design-principle conflict:** maintaining a patched engine fork is in tension with the
  project's preference for an unmodified, auditable upstream dependency and with the
  no-vendored-patch constraint; it adds a permanent maintenance and supply-chain surface.
  This must be weighed against the alternatives (process containment where deployable; a
  different engine) in the consolidated engine-feasibility review, not decided here.

# Decision

**Outcome B.** The Promise/async uncatchable-stop swallow is completely fixable for the
RFC 0001 language profile by a bounded, pattern-consistent engine change (12 edits / 11
functions, ASan+UBSan+leak-clean, ordinary semantics preserved, fresh runtime healthy), and
no published upstream QuickJS-NG revision contains that fix. The fix is *bounded* but not
uniformly one-line: the async-generator route requires the conversion guard **plus**
cooperation from the void helper's two callers, a nuance now proven and independently
corroborated upstream.

Per Outcome B, the next coherent task is a **consolidated engine-feasibility review** that
compares, as portable termination strategies: (a) a maintained QuickJS-NG fork carrying
this patch, (b) process/extension containment where deployable, and (c) an alternative
engine — and that weighs each against the design principles and the charter's platform
list. No engine is selected, no fork is authorised, and no RFC/ADR is created here.

## Validation

- `experiments/quickjs-async-termination-fix-spike/build_and_run.py`: `RESULT: PASS`;
  baseline SHA re-verified; 12 replacements applied to the patched copy only; baseline copy
  byte-identical to the pin; ASan+UBSan+`JS_ABORT_ON_LEAKS` clean on both engines; fresh
  runtime healthy; no outer-watchdog kill involved.
- Preserved Promise-swallow reproducer: passes (defect still reproduces on the pinned
  engine, all five shapes).
- Preserved RegExp-compile reproducer: exits 2, `callback_count=0` (still non-polling).
- Upstream determination made from direct inspection of public `master` source and history.
- Pins, RFC 0001 (proposed, unedited), absence of any ADR and of any engine/parser
  selection: all unchanged. The committed vendored engine is untouched; the patched copy is
  generated and gitignored.
