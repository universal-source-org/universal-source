# QuickJS async/Promise uncatchable-stop termination fix and version feasibility

Status: completed — **Outcome B**. Baseline `304818de0619555e05b85184619509e3da38a734`
(`docs(engine): review process containment and async termination`); clean tree, no other
active plan. Phase 3 active.

Scope: one runtime-correctness + engine-version feasibility experiment. Determine (1) whether
the Promise/async uncatchable-stop swallowing defect is completely fixable by a bounded
engine change on the pinned QuickJS-NG revision, and (2) whether a published upstream
QuickJS-NG revision already contains that fix. Throwaway experiment-local patched builds
only; no change to the committed dependency.

Non-goals (unchanged): no change to production `runtime/`; no committed/vendored QuickJS
patch; no pin/rquickjs update; no RFC acceptance; no engine selection; no ADR; no process
containment; no portable-contract change; no module/path work; no service profiles; no
platform app code; no push/tag/release. Pins frozen. External watchdog is only a safety net
(never counts as engine-level success). All prior blocker documents preserved.

## Findings

- **Part 1:** Exhaustive `quickjs.c` enumeration of all 36 `JS_GetException` sites; only
  five `JS_IsUncatchableError` references exist (four guards). Reachable swallow set under
  RFC 0001 = 10 conversion sites across 9 functions, plus 2 caller-propagation functions =
  **12 edits / 11 functions**. Out-of-profile sites (explicit resource management, dynamic
  import) enumerated and left unpatched as unreachable.
- **Part 2:** Isolated throwaway experiment `experiments/quickjs-async-termination-fix-spike/`
  (SHA-verified copy, gitignored baseline/patched engines, per-rule replacement-count
  assertions, ASan+UBSan + QuickJS leak-abort). Committed dependency untouched.
- **Part 3:** Regression matrix `RESULT: PASS`. Every uncatchable route (native-gate and
  latched-deadline mechanisms) flips SWALLOWED→TERMINATED or stays TERMINATED (already
  guarded); every ordinary route stays CAUGHT; fresh runtime healthy.
- **Part 4:** Per-site cleanup verified against each site's existing rejection cleanup; no
  premature `JS_GetException`; ASan+UBSan+leak-abort clean on both engines (not "did not
  crash").
- **Part 5:** Completeness = Complete for the RFC 0001 profile (harness `UNRESOLVED` sentinel
  found the async-generator caller gap; zero remain after 7b/7c).
- **Part 6:** Preserved Promise-swallow reproducer still passes (defect reproduces on the
  pinned engine); preserved RegExp-compile reproducer still exits 2, non-polling.
- **Part 7:** No upstream fix — current `master` (0.17.0) is byte-identical to the pin at
  every affected conversion site; defect publicly documented (bellard/quickjs#341) but
  unfixed for these sites.
- **Part 8:** N/A (no candidate version). **Part 9:** fork surface recorded (12 edits, ~40
  lines, clean apply, low churn, design-principle tension); fork **not** authorised.

## Outcome and next task

**Outcome B** — bounded fix complete for the profile; upstream lacks it. Next coherent task:
a consolidated engine-feasibility review comparing a maintained fork carrying this patch vs
process/extension containment vs an alternative engine, weighed against the design
principles and the charter platform list. No engine selected; no fork authorised; no
RFC/ADR here.

## Validation

| Check | Result |
|-------|--------|
| `build_and_run.py` | PASS; SHA re-verified; 12 replacements to patched copy only; baseline byte-identical; ASan+UBSan+leak-abort clean; fresh runtime healthy |
| Preserved Promise-swallow reproducer | passes (defect reproduces on pinned engine, 5 shapes) |
| Preserved RegExp-compile reproducer | exit 2, `callback_count=0` (non-polling) |
| Upstream determination | direct inspection of public master source + history |
| Pins / RFC / ADR / engine selection | unchanged; committed engine untouched; patched copy gitignored |
