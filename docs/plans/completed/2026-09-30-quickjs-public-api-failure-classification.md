# Public-API failure-classification spike

Baseline: `98c69242d35f79918688b11c3b144300ea121545`, `main`, clean tree;
no prior active plan. CURRENT_STATE's older pre-documentation snapshot is not
a pin; the intervening commits are documentation/ignore changes, not a technical
conflict.

## Scope

Answer whether the existing experimental QuickJS-NG pin, unchanged async patch
and Rust public API can make allocator and engine-heap failures trusted terminal
conditions with no source continuation, preserving ordinary exception semantics.
Use the integrated crate, adding only diagnostic knobs and focused tests. Read
the routed integrated/resource/async evidence and RFC §§4–7/10. No contract,
production, engine patch, selection, other gate or publication changes.

## Work and checks

1. Audit public allocator, heap, interrupt and exception APIs and their pinned
   implementations; identify the earliest trusted observation point.
2. Add paired allocator/engine-ceiling probes, catch/Promise/async witnesses,
   ordinary/forged/stack controls, public exception-marking limits, and retirement,
   capability/publication and fresh-runtime assertions.
3. Run the new tests and complete integrated suite; rerun original async swallow,
   patched C termination regression, and non-polling RegExp reproducer. Preserve
   historical files. Run format/clippy and documentation/link/whitespace checks.
4. Record Outcome and minimal blocker/next decision, update current context only,
   review the full diff, move this plan to completed and commit locally.

## Results

Completed — **Outcome B**, documented in the
[review](../../reviews/2026-09-30-quickjs-public-api-failure-classification.md).

- Added seven diagnostics in the existing integrated crate and one optional
  call-entry public heap-limit override; defaults and original tests unchanged.
  The only new unsafe block wraps the public Error flag query/set/query at host
  control boundaries with live borrowed values; no private layout or reentry.
- Trusted allocator refusal blocks action/publication but not source catch/finally.
  Engine heap rejection bypasses the latch and can allow action/42 publication.
  Removing its ceiling still leaves allocator catch continuation. Public marking
  after catch is too late and cannot mark the real OOM null fallback. Ordinary,
  forged and actual stack exceptions retain ordinary semantics.
- Final patched integrated run: 22/22; baseline control: 1/1; fmt and clippy
  `-D warnings` passed. Initial new-test SHA assertion typo was corrected before
  these final passes. Build source/metadata/linkage provenance remains verified.
- Resource suite: 11/11. Original Promise/async negative control: 1/1, five
  shapes each swallow 1,000 stops. Unchanged C async-fix ASan/UBSan/leak-abort
  harness: PASS. Original RegExp compiler: zero callbacks for returned cases,
  large case external kill/exit 2 as expected (not an engine-control success).
- Updated experiment guide, current-state index, roadmap and architecture only
  for these findings. Historical reviews/reproducers, engine patch, lockfiles,
  RFCs, production and other gates remain unchanged.
- Full diff and scoped Markdown structure/relative-link/anchor/whitespace checks
  completed. No JSON or shared contracts changed. Full Phase 3/declarative and
  cross-platform suites were not needed for this blocked follow-up. Integrated
  Rust sanitizer linking was not retried; inherited limitation remains explicit.

Remaining blocker: no trusted engine-limit interception and no terminal OOM
propagation before source catch, including Error allocation failure. Gate remains
open. Only next task is a decision-only review of whether to authorize a separate
engine-level allocation/heap terminal-failure experiment and its minimum scope
and acceptance criteria. No new engine change or other gate starts here.
