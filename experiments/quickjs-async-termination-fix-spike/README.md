# QuickJS-NG async/Promise uncatchable-stop termination fix spike

**Outcome B — a bounded engine change completely fixes the swallow for the RFC 0001
language profile, and no published upstream QuickJS-NG revision contains it.**

This isolated, throwaway experiment answers two independent questions for Phase 3:

1. Is the Promise/async uncatchable-stop *swallowing* defect **completely fixable by a
   bounded engine change** on the pinned QuickJS-NG revision, reusing the engine's own
   `JS_IsUncatchableError` guard (the pattern `promise_reaction_job` already carries)?
   **Yes**, for every route reachable under RFC 0001.
2. Does any **published upstream** QuickJS-NG revision after the pin already contain an
   equivalent fix? **No** — current master (0.17.0) is byte-identical to the pin at every
   affected conversion site.

Baseline commit: `304818de0619555e05b85184619509e3da38a734`. Phase 3 remains active. This
experiment does **not** change the production engine, patch/commit the vendored dependency,
update pins, update rquickjs, select an engine, accept RFC 0001, or amend resource
semantics. See the [review](../../docs/reviews/2026-09-29-quickjs-async-termination-fix-spike.md)
and the [completed plan](../../docs/plans/completed/2026-09-29-quickjs-async-termination-fix-spike.md).

## What this is

- `build_and_run.py` — the driver. It **never** touches the committed dependency:
  1. Locates the pinned QuickJS-NG source and SHA-verifies `quickjs.c`
     (`3a6b52a2…a745b3`, revision `0fdea21f`, 0.16.2).
  2. Copies it into `build/baseline/qjs` and `build/patched/qjs` (both **gitignored**).
  3. Applies the guard edits **only** to the patched copy, asserting the exact
     replacement count per rule (12 replacements across 11 functions).
  4. Confirms the baseline copy is byte-identical to the pinned source.
  5. Builds `harness.c` against each engine with `-fsanitize=address,undefined`,
     `-fno-sanitize-recover=all`, and QuickJS's own `JS_DUMP_LEAKS | JS_ABORT_ON_LEAKS`
     (a portable leak check; LeakSanitizer is unavailable on macOS/arm64).
  6. Runs both and diffs the verdicts.
- `harness.c` — a pure-C regression matrix. Each uncatchable route is exercised in a
  fresh runtime with both host-stop mechanisms:
  - `stop()` — a native throw marked uncatchable (the shape the RegExp **pre-admission
    gate** raises: `JS_ThrowInternalError` + `JS_SetUncatchableError`).
  - `expire(); for(;;){}` — a **latched deadline** interrupt (the shape the host uses for
    Timeout/Cancel/ResourceLimit via `JS_ThrowInterrupted`).
- `build/` — generated, gitignored baseline & patched engine copies and binaries.

No outer watchdog decides success. Termination is proven **in-process**: the engine
returns control to the C host with the uncatchable still latched and with source JS never
continuing (`cont()` is never called). A 120 s outer timeout exists only to fail a genuine
hang, and a timeout is reported as FAIL.

## The fix (experiment-local, generated, never committed)

Every edit reuses the engine's existing
`if (unlikely(JS_IsUncatchableError(ctx->rt->current_exception)))` check *before*
`JS_GetException`, and routes the latched uncatchable back to the host through each
site's **existing** cleanup (a shared `fail`/`done` label where one exists, else the
site's own owned values). No conversion path calls `JS_GetException` on an uncatchable.

| # | Function | Route | Propagation |
|---|----------|-------|-------------|
| 1 | `js_promise_constructor` | executor throw | `goto fail` |
| 2 | `js_promise_resolve_function_call` | thenable `then` getter | `return JS_EXCEPTION` |
| 3 | `js_promise_resolve_thenable_job` | thenable `then` call | skip convert; existing tail frees args, returns `res` |
| 4 | `js_promise_all` (all/allSettled/any) | iterator/resolve throw | `goto fail` |
| 5 | `js_promise_race` | iterator/resolve throw | `goto fail` |
| 6 | `js_promise_try` | callback throw | free funcs + result promise, `return JS_EXCEPTION` |
| 7 | `js_async_generator_resume_next` | body resume throw | `return;` (void) |
| 7b | `js_async_generator_next` | caller of the void helper | free promise, `return JS_EXCEPTION` |
| 7c | `js_async_generator_resolve_function` | await-resume caller | `return JS_EXCEPTION` (then `promise_reaction_job`'s existing guard carries it) |
| 8 | `js_async_generator_completed_return` | `.return()` promise resolve | `return -1` (mirrors existing error path) |
| 9a | `js_async_from_sync_iterator_next` | `for await` sync next/getter | free funcs + promise, `return JS_EXCEPTION` |
| 9b | `js_async_from_sync_iterator_next` | sync `IteratorClose` | free funcs + promise, `return JS_EXCEPTION` |

**Key finding:** the async-generator route (7/7b/7c) is *not* fixable by the one-line
guard alone. `js_async_generator_resume_next` is a `void` helper whose two callers
(`js_async_generator_next`, `js_async_generator_resolve_function`) discard a pending
exception, so they must cooperate to propagate. The harness detected this directly: with
only the site-7 guard, `async_gen_resume__uncatchable` went to `UNRESOLVED` (source did
not continue, but control did not return to the host). This matches the prediction in
[bellard/quickjs#341](https://github.com/bellard/quickjs/issues/341) that the void
helper's callers "should change too."

## Reproduce

From the repository root (no network needed):

```sh
python3 experiments/quickjs-async-termination-fix-spike/build_and_run.py
```

Expected: `RESULT: PASS`. Every uncatchable/deadline route flips baseline `SWALLOWED`
→ patched `TERMINATED` (or stays `TERMINATED` for the two routes already guarded upstream);
every ordinary exception stays `CAUGHT` in both; a fresh runtime is healthy; no sanitizer
or leak abort fires.

## Environment

Local evidence only: macOS 27.0 (26A428), arm64, Apple clang 21.0.0. Other platforms and
build profiles are untested. The committed project dependency is unchanged.
