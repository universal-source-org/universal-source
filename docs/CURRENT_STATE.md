# Current working state

This is the compact continuation index, not a specification or decision record.
Specs govern contracts; ADRs govern accepted decisions; reviews own evidence;
plans track tasks; ROADMAP owns sequencing; ARCHITECTURE describes implementation.
Read the relevant route below, not the entire review archive. If this index
conflicts with an authority, surface the conflict and update the index explicitly.

## 1. Project position

Phase 3 — JavaScript binding design and feasibility — is active.
Task baseline verified 2026-09-30 at
`6fe23ddefc3274d8986f87164cfbca0730e33679` on `main` (clean tree).
The completed maintenance acceptance review is recorded below; no active plan remains.
Always inspect live Git status/HEAD and active plans; this baseline is not a pin.

Universal Source defines a portable content-source/host contract for independent
mobile, TV and desktop hosts: write once, run everywhere, with explicit authority
and testable behavior. It is not a player application. The Rust declarative
reference loader, dispatcher and lifecycle are implemented; Phases 1 and 2 are
complete. Production JavaScript execution is unimplemented. RFC 0001 remains
proposed; no JavaScript engine/parser is selected and no engine ADR exists.
See [charter](PROJECT_CHARTER.md), [architecture](ARCHITECTURE.md) and
[roadmap](ROADMAP.md#recommended-next-coherent-task).

## 2. Current decisions

These are the working constraints of the proposed binding, not RFC acceptance.

- **Lifecycle Model B:** fresh mutable JS realm per invocation; QuickJS evidence
  uses a fresh Runtime/Context pair. Logical snapshots may be reused, JS state
  may not. Close delivery and retire the graph with **no final job drain**
  ([RFC §§4–5](../spec/rfcs/0001-javascript-execution-binding.md#4-initialization-state-and-disposal)).
- **§6 portable values:** bounded structural copying of null, booleans, finite
  binary64 numbers, valid Unicode strings, dense arrays and ordinary records;
  no source hooks or helper/native identities in returned data
  ([RFC §6](../spec/rfcs/0001-javascript-execution-binding.md#6-value-boundary)).
- **Authority:** explicit invocation-scoped capabilities, no ambient OS/network/
  filesystem authority; exact services remain separate work
  ([RFC §8](../spec/rfcs/0001-javascript-execution-binding.md#8-host-capability-injection-and-ownership)).
- **Maintenance gate: Fail.** QuickJS loses default candidate status; it remains
  an option for comparison, not a selected or permanently rejected engine.
  The full terminal-OOM artifact/source-bound success and accepted maintenance
  ownership are absent ([maintenance review](reviews/2026-09-30-quickjs-maintenance-acceptance.md)).
  Only end-to-end candidate architecture cost comparison under the same RFC is next.
- **Conditional mechanisms:** RegExp needs pre-admission, not assumed compiler
  interruption; the async-stop patch is experiment-only, not an authorized
  maintained fork ([consolidated review](reviews/2026-09-29-consolidated-engine-feasibility.md)).

## 3. Proven / evidence-backed capabilities

Statuses describe local feasibility evidence, not production conformance.

| Area | Current evidence | Authority |
| --- | --- | --- |
| Lifecycle / job retirement | Viable: fresh runtime, no drain, late-delivery rejection | [Lifecycle proof](reviews/2026-09-28-quickjs-lifecycle-spike.md) |
| Pre-evaluation module capture | Viable: public native callback; full path containment open | [Module capture](reviews/2026-09-28-quickjs-module-capture-spike.md) |
| Static analysis / preflight | Viable with condition: tested declarations and attribute detection; complete strategy open | [Attribute follow-up](reviews/2026-09-28-javascript-import-attributes-spike.md) |
| Dynamic-code suppression | Viable for tested compilation routes | [Compiler denial](reviews/2026-09-28-quickjs-dynamic-code-suppression-spike.md) |
| Global hardening | Viable for tested restricted surface / reachability | [Global surface](reviews/2026-09-28-quickjs-global-surface-spike.md) |
| §6 value boundary | Experimental proof: class-gated, non-executing bidirectional transfer; exhaustion qualified below | [Complete values](reviews/2026-09-28-quickjs-complete-value-spike.md) |
| RegExp admission | Viable with condition: literal preflight + trusted dynamic gate + protected native constructor | [Admission implementation](reviews/2026-09-29-quickjs-regexp-admission-impl-spike.md) |
| Promise/async hard stop | Experimental proof for restricted profile, only with local patch | [Async fix](reviews/2026-09-29-quickjs-async-termination-fix-spike.md) |
| Integrated resource/failure handling | Public-API configuration remains negative; engine-level terminal-OOM feasibility accepted as task premise, complete patched artifact/composition not audited at this HEAD | [Artifact audit](reviews/2026-09-30-quickjs-maintenance-acceptance.md#artifact-search-and-what-was-actually-found) |
| Platform embedding / production integration | Open / unimplemented | [Architecture](ARCHITECTURE.md) |

## 4. Known limitations / conditions

- Native RegExp compilation does not poll on the pin; finite admission is a
  permanent candidate policy. Matching interruption is separately evidenced.
- The last upstream assessment found no published async-stop fix; this is dated
  evidence, not a live upstream check. The local async fix is 12 edits / 11
  functions in one engine file. Fresh diff audit: 43 added / 4 removed lines,
  11 recipe rules / 12 sites. This is not the total candidate patch size.
- Terminal allocation/heap failure is engine-level feasible per the current task
  premise. The checked-in evidence ends at public-API Outcome B: no corresponding
  terminal-OOM patch/site audit is present. Its exact cost/provenance is an audit
  gap, not a reason to repeat feasibility. Eighteen local engine copies have only
  baseline or async-only digests; full provenance/combined scope is unidentified.
- Maintenance roles and update/security procedures are specified in the new
  review but not operationally accepted. No owner/reviewer commitment exists;
  SECURITY.md is empty. Existing build provenance is not a test-success record.
- A trusted allocation latch prevents capability action/publication while
  catch/finally still run. Engine heap rejection bypasses that latch and can
  permit action/success. Public error marking is too late after host return and
  cannot mark the null OOM fallback; heap-cap tuning and later polling do not
  close this boundary. Ordinary/stack exceptions retain ordinary semantics.
- Engine-accounted bytes, host/Rust allocations and §6 budgets differ; no exact
  RSS or real-time guarantee. Residual native work such as sparse `Array.join`
  still requires finite policy coverage ([latest review](reviews/2026-09-30-quickjs-integrated-resource-failure-spike.md)).
- New integrated Rust sanitizer linking failed locally; prior C sanitizer proof
  does not establish sanitizer coverage of the Rust integration.
- No charter-wide embedding or Apple containment proof. Selection remains gated;
  local Darwin results do not establish device or cross-platform support.

## 5. Current pre-selection gates

The [maintenance decision](reviews/2026-09-30-quickjs-maintenance-acceptance.md#acceptance-decision-and-next-task)
ends default QuickJS gate progression. Remaining technical gates are retained as
comparison evidence; none is authorized as the next implementation task.

| Gate | Status | Remaining requirement |
| --- | --- | --- |
| Integrated resource/failure closure | Open; feasibility premise accepted | Audit complete patched composition and residual resource coverage; do not repeat whether terminal OOM is fixable |
| Complete module/path containment | Pending | Hostile graph/path proof, stable bytes and private module denial |
| Final parser/preflight strategy | Pending | Full restricted ES2023 / early errors / admission agreement |
| Cross-platform / Apple embedding | Pending | Same engine/binding/patch build and representative execution proof |
| Complete patch-set maintenance acceptance | **Completed — Fail** | Missing complete artifact/provenance and accepted ownership; procedure defined, not operational. Compare candidate architectures next |

## 6. Current task

**Completed: QuickJS complete patch-set maintenance acceptance — Fail.**

- [Review](reviews/2026-09-30-quickjs-maintenance-acceptance.md) and
  [completed plan](plans/completed/2026-09-30-quickjs-maintenance-acceptance.md).
- The async subset is auditable; the full terminal-failure set is not identified.
  Feasibility remains the premise. Missing artifact and ownership fail the gate;
  no new engine experiment or maintained fork follows.
- **Only next task: end-to-end candidate architecture cost comparison under the
  same RFC.** Compare complete termination/resource, host-policy, preflight,
  binding, target qualification and maintenance/security models. Retain QuickJS
  as one option without assuming it is the default or an alternative is superior.
- Next task is **not started**. Do not begin alternative-engine experiments,
  IPC/helper-process design, another selection gate or contract relaxation.
- No production runtime, engine ADR or RFC acceptance is authorized.

## 7. Do not do

- Do not call QuickJS selected, accept RFC 0001 implicitly, or authorize a fork.
- Do not add production JS execution before the separate selection/adoption work.
- Do not rewrite historical negative evidence because later evidence qualifies it.
- Do not broaden one coherent task into another gate; do not push/tag/release
  without explicit authorization.

## 8. Evidence routing

Read applicable contract sections and these starting points; follow further
links only to resolve a concrete question. Older reviews' “next task” paragraphs
are historical handoffs; use the current ROADMAP for today's sequence.

| If the task concerns... | Read these first |
| --- | --- |
| Current failure-boundary decision | Active plan if present + [public-API review](reviews/2026-09-30-quickjs-public-api-failure-classification.md) + [integrated predecessor](reviews/2026-09-30-quickjs-integrated-resource-failure-spike.md) + [experiment guide](../experiments/quickjs-integrated-resource-failure-spike/README.md) |
| Lifecycle / job retirement | [RFC §§4–5](../spec/rfcs/0001-javascript-execution-binding.md#4-initialization-state-and-disposal) + [lifecycle proof](reviews/2026-09-28-quickjs-lifecycle-spike.md) |
| §6 values | [RFC §6](../spec/rfcs/0001-javascript-execution-binding.md#6-value-boundary) + [complete-value review](reviews/2026-09-28-quickjs-complete-value-spike.md) |
| Modules / paths / preflight | [RFC §1](../spec/rfcs/0001-javascript-execution-binding.md#1-package-and-module-model) + [capture](reviews/2026-09-28-quickjs-module-capture-spike.md) + [attribute follow-up](reviews/2026-09-28-javascript-import-attributes-spike.md) |
| Hardening / ambient authority | [Global surface](reviews/2026-09-28-quickjs-global-surface-spike.md) + [compiler denial](reviews/2026-09-28-quickjs-dynamic-code-suppression-spike.md) |
| RegExp | [Admission implementation](reviews/2026-09-29-quickjs-regexp-admission-impl-spike.md) + [integrated qualification](reviews/2026-09-30-quickjs-integrated-resource-failure-spike.md) |
| Async termination | [Async fix](reviews/2026-09-29-quickjs-async-termination-fix-spike.md) + [integrated qualification](reviews/2026-09-30-quickjs-integrated-resource-failure-spike.md) |
| Candidate comparison / maintenance / selection readiness | [Maintenance decision and artifact audit](reviews/2026-09-30-quickjs-maintenance-acceptance.md) + [candidate cost ledger](reviews/2026-09-30-quickjs-candidate-viability.md); for a concrete alternative comparison question, follow the prior [consolidated assessment](reviews/2026-09-29-consolidated-engine-feasibility.md). Historical claims require appropriate revalidation; no archive preload |
| Exact async patch / failure provenance | [Async patch audit](reviews/2026-09-29-quickjs-async-termination-fix-spike.md) + [public-API boundary](reviews/2026-09-30-quickjs-public-api-failure-classification.md) + [fresh artifact inventory](reviews/2026-09-30-quickjs-maintenance-acceptance.md#exact-inspectable-provenance) |
| Platform containment context | [Prior consolidated review](reviews/2026-09-29-consolidated-engine-feasibility.md) + [qualification limits](reviews/2026-09-30-quickjs-maintenance-acceptance.md#single-engine-build-and-regression-qualification); historical platform observations are dated |
| Declarative runtime / portable contracts | [Runtime guide](../runtime/README.md) + [spec index](../spec/README.md) + [ADR index](decisions/README.md) |

## 9. Validation expectations

Follow [CONTRIBUTING](../CONTRIBUTING.md) and the touched experiment's guide.
During implementation, run new/touched tests and directly dependent suites.
Before committing architecture-critical evidence, also run directly affected
regressions and retain negative controls. Report failures and unrun checks.
Run the full Phase 3 corpus when a shared engine patch, lifecycle semantics,
shared hardening or shared §6 conversion changes, or for closure/consolidated
reviews. Broaden checks for a concrete dependency/risk or explicit task mandate,
not just because a suite exists. Documentation-only work needs Markdown,
links/anchors and whitespace checks, not experiment rebuilds.

## 10. Chat/report discipline and upkeep

Detailed evidence matrices belong in reviews, not chat. Final reports normally
stay within about 600–800 words: outcome, commit, checks/limits, blockers and one
next task, with links for detail. Do not paste the historical evidence chain.
At a task boundary, refresh this index's baseline, status, active-plan route and
next task from the authorities; preserve historical reviews and keep this compact.

## Fresh-conversation prompt

```text
Continue Universal Source from the current repository state.
Read AGENTS.md, docs/CURRENT_STATE.md and docs/ROADMAP.md first.
Then read only the documents routed for the current task.
Complete only the coherent task recorded there; do not expand into the next gate.
Inspect git status before editing and preserve existing work.
Commit locally; do not push.
```
