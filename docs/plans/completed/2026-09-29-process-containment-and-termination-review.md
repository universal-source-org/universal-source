# Process-containment and async-termination review

Status: completed — **Outcome C**. Baseline `2b05e0b77e2a4a123c8c9bb8df48cb080841edc2` (`test(engine): record RegExp admission spike Outcome B`); clean tree, no other active plan. Phase 3 active.

Scope: one design-level review. Decide whether disposable helper-process containment could restore RFC 0001's termination/resource guarantees without changing the portable contract, and whether the Promise/async stop-swallowing defect recorded by the admission spike has a small credible engine-level remedy. No implementation of either remedy.

Deliverables:

1. Process-boundary model comparison (per-invocation, reusable-runtime, worker pool, per-logical-instance): isolation, startup, memory, teardown, crash containment, state leakage, Model B compatibility.
2. Trusted parent-side stop classification and ordering; §6 value transfer over IPC; host-service invocation over IPC; Model B lifecycle mapping; RegExp and Promise-swallow behaviour under containment; resource accounting by layer.
3. Platform viability matrix (Linux, macOS, Windows, Android, iOS, iPadOS, tvOS), with Apple mobile/TV application restrictions on helper processes.
4. Cross-platform architectural consequence (Models P, H, E, S) against charter/principles/roadmap.
5. Root-cause review of the pinned QuickJS Promise/async exception-to-rejection conversion sites; minimal-patch hypothesis; regression matrix; upstream/version evidence under the local-only constraint.
6. Option comparison table; one design outcome (A/B/C/D); the smallest next task.

Non-goals (unchanged): no IPC/process implementation, no production helper, no QuickJS patch/upgrade, no RFC edit, no production runtime change, no final service signatures, no engine/parser selection, no ADR, no module/path completion, no platform application code, no push/tag/release. Pins fixed. The external watchdog is only a safety net. Prior Outcome B documents stay intact.

Network: the task's top-level constraint forbids external/remote access ("Do not access external services, accounts, credentials or third-party systems"; "Do not interact with any remote service or system"). This is treated as binding over the conditional research invitations in Parts 2 and 4.3. Platform restrictions are recorded as established reviewer knowledge to be confirmed against current vendor documentation before any implementation; a concrete upstream QuickJS-NG fix determination is deferred to a remote-permitted follow-up.

## Steps

1. Read charter, principles, ADRs, spec (source/host/lifecycle), RFC 0001, and the Phase 3 reviews; inspect the pinned engine source for the conversion sites.
2. Write the review: process models, trusted termination, IPC value/service boundary, lifecycle mapping, platform matrix, Apple limits, root cause, minimal-patch hypothesis, regression matrix, upstream evidence, option table, outcome, next task.
3. Rerun the Promise swallow reproducer and the RegExp compiler blocker; rerun the resource tests; doc/link/whitespace checks; confirm pins/deps/RFC/ADR unchanged.
4. Minimal ROADMAP/ARCHITECTURE updates; move this plan to completed; local commit; do not push.

## Findings during work

- The engine defines `JS_IsUncatchableError` once and checks it in exactly three places: the interpreter unwind (`quickjs.c:20901`), async-function resume (21482, 21491) and `promise_reaction_job` (55584). Every other exception-to-rejection conversion omits it.
- `promise_reaction_job` (55558) already contains the exact guard the conversion sites lack: `if (unlikely(JS_IsUncatchableError(ctx->rt->current_exception))) return JS_EXCEPTION;` before `res = JS_GetException(ctx);`. The fix pattern therefore already exists in the same file.
- Expected outcome: process containment cannot serve the charter-required Apple mobile/TV scope, while the Promise-swallow defect is plausibly one bounded engine fix (the reaction-job guard replicated or centralised) and is the only remaining in-process termination blocker after pre-admission. Leaning Outcome C, next task a narrowly scoped engine-fix/version feasibility experiment.

## Result

**Outcome C — a bounded engine-level fix is the more plausible portable path.** See the [review](../../reviews/2026-09-29-process-containment-and-termination-review.md).

- Process containment restores hard termination and maps cleanly onto Model B, §6 transfer, host-service injection over IPC, and trusted parent-side kill-reason classification. The safe default is one fresh child per invocation; a pool is a happy-path latency optimisation, but any call the parent had to kill/interrupt cannot reuse its worker (the resource review's "cannot destroy a runtime while its thread is still executing it" rule).
- Platform matrix: straightforward on Linux/macOS/Windows; achievable on Android via `isolatedProcess` services; **generally unavailable on iOS/iPadOS/tvOS** App Store apps (no spawnable helper processes). So containment cannot be the universal portable answer, and Outcome A is excluded.
- The remaining in-process blocker (Promise/async stop-swallowing) is plausibly one small enumerable engine defect: `JS_IsUncatchableError` is referenced in only four places, and `promise_reaction_job` (`quickjs.c:55584`) already carries the exact guard the sibling exception-to-rejection conversion sites (executor 55916, thenable-resolve 55805 reached by `await`/`Promise.resolve`, `Promise.all` 56241, async generator 21891, and `Promise.race`/`Promise.try`/completed-return by reading) omit. With the already-working in-process RegExp pre-admission, a bounded fix would restore §§5/7/10 termination in process on every platform, including where containment is impossible.
- Therefore the next task is a narrowly scoped engine-fix/version feasibility experiment, not a containment build. Outcome D is not reached (a credible bounded remedy exists); Outcome B is rejected because conceding Apple mobile/TV scope is a larger compromise than first testing that fix.

Remaining gaps: exhaustive site enumeration; whether any other non-polling native path exists; upstream availability of the fix (undetermined under the local-only constraint); vendor confirmation of the platform matrix; and containment IPC latency/pool-reuse quantification. External access was not used; platform facts are reviewer knowledge to confirm, and the upstream-fix question is deferred to a remote-permitted follow-up.

## Validation

Host: macOS 27.0 (26A428), arm64, Rust/Cargo 1.96.0, debug test profile. Documentation-only change: no crate created or modified, so no new fmt/Clippy surface. Experiment suites ran under a process-group guard; no guard fired.

| Command | Result |
| --- | --- |
| `python3 experiments/quickjs-resource-spike/probe_regexp.py` | **Exit 2** as preserved (1,000 / 4,000 / 16,000 refs, zero callbacks; 64,000 refs killed by the external five-second guard). Both blockers still reproduce. |
| `cargo test --manifest-path experiments/quickjs-resource-spike/Cargo.toml --offline --locked -- --test-threads=1` (guarded) | 11 pass, unchanged. |
| `cargo test --manifest-path experiments/quickjs-regexp-admission-impl-spike/Cargo.toml --offline --locked -- --test-threads=1 --nocapture promise_machinery await_resolution` | 2 pass: swallow reproducer (1,000 iterations complete, ~1,002 callbacks per shape) and `await` ignores the global `Promise`. |
| Link/anchor and whitespace check on changed Markdown; `git diff --check` | Pass. |
| Pins, `runtime/`, `spec/`, `conformance/`, `examples/`, `docs/decisions/`, experiments unchanged | Confirmed by diff against baseline. |

## Audit and delivery

No production runtime, dependency, RFC, ADR or engine/parser selection change. No engine patch or upgrade. Pins fixed. RFC 0001 remains proposed. Changed files: `docs/ARCHITECTURE.md`, `docs/ROADMAP.md`, the review, and this plan. Local Conventional Commit only; no push, tag, release or settings change.
