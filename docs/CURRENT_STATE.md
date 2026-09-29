# Current working state

This is the compact continuation index, not a specification or decision record.
Specs govern contracts; ADRs govern accepted decisions; reviews own evidence;
plans track tasks; ROADMAP owns sequencing; ARCHITECTURE describes implementation.
Read the relevant route below, not the entire review archive. If this index
conflicts with an authority, surface the conflict and update the index explicitly.

## 1. Project position

Phase 3 — JavaScript binding design and feasibility — is active.
Snapshot verified 2026-09-30 against pre-documentation HEAD
`222029b74c278daa61fa3345fc0c131c6f449af3` on `main` (clean tree).
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
- **Candidate, not selection:** QuickJS-NG/rquickjs remains strongest-evidenced.
  Consolidated Outcome A means continue toward selection; latest integrated
  Outcome B leaves resource closure open ([latest review](reviews/2026-09-30-quickjs-integrated-resource-failure-spike.md#decision)).
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
| Integrated resource/failure handling | Blocked: allocator catch continuation; engine ceiling may bypass trusted latch | [Integrated Outcome B](reviews/2026-09-30-quickjs-integrated-resource-failure-spike.md#failure-and-resource-boundary) |
| Platform embedding / production integration | Open / unimplemented | [Architecture](ARCHITECTURE.md) |

## 4. Known limitations / conditions

- Native RegExp compilation does not poll on the pin; finite admission is a
  permanent candidate policy. Matching interruption is separately evidenced.
- The last upstream assessment found no published async-stop fix; this is dated
  evidence, not a live upstream check. The local fix is 12 edits / 11 functions.
- A trusted allocation latch can prevent publication while source catch code
  still runs. Engine heap rejection may occur before that latch. Do not infer a
  trusted stop from exception text; ordinary stack errors retain ordinary semantics.
- Engine-accounted bytes, host/Rust allocations and §6 budgets differ; no exact
  RSS or real-time guarantee. Residual native work such as sparse `Array.join`
  still requires finite policy coverage ([latest review](reviews/2026-09-30-quickjs-integrated-resource-failure-spike.md)).
- New integrated Rust sanitizer linking failed locally; prior C sanitizer proof
  does not establish sanitizer coverage of the Rust integration.
- No charter-wide embedding or Apple containment proof. Selection remains gated;
  local Darwin results do not establish device or cross-platform support.

## 5. Current pre-selection gates

The [consolidated gate list](reviews/2026-09-29-consolidated-engine-feasibility.md#must-complete-before-an-engine-selection-adr)
is qualified by the latest review and ROADMAP. None is closed by this index.

| Gate | Status | Remaining requirement |
| --- | --- | --- |
| Integrated resource/failure closure | **Next** | Integrated spike completed with Outcome B; failure-classification follow-up below |
| Complete module/path containment | Pending | Hostile graph/path proof, stable bytes and private module denial |
| Final parser/preflight strategy | Pending | Full restricted ES2023 / early errors / admission agreement |
| Cross-platform / Apple embedding | Pending | Same engine/binding/patch build and representative execution proof |
| Patch ownership/update/integration | Pending | Reproducible provenance, ownership, security/update tests and upstream strategy |

## 6. Current task

**Only next coherent task: non-production public-API failure-classification spike.**

- Status: **not started**. No technical active plan or uncommitted technical work
  at the verified baseline. Create a scoped plan in [active plans](plans/active/)
  when this task starts; no active plan path exists yet.
- Predecessor: integrated resource/failure spike **completed with Outcome B**,
  not closure. Read its [review](reviews/2026-09-30-quickjs-integrated-resource-failure-spike.md)
  and [experiment guide](../experiments/quickjs-integrated-resource-failure-spike/README.md).
- Discriminating question: can engine-heap and allocator failures become a
  trusted, no-continuation terminal class through public APIs without another
  substantive QuickJS change, while ordinary exceptions stay ordinary?
- Scope: evidence on the existing experimental pin/patch/binding path; preserve
  historical negative controls and report blockers. No new engine surgery,
  production integration, contract amendment, alternative-engine spike or other
  selection gate follows implicitly. This index does not authorize that expansion.

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
| Current execution work | Active plan if present + [integrated review](reviews/2026-09-30-quickjs-integrated-resource-failure-spike.md) + [experiment guide](../experiments/quickjs-integrated-resource-failure-spike/README.md) |
| Lifecycle / job retirement | [RFC §§4–5](../spec/rfcs/0001-javascript-execution-binding.md#4-initialization-state-and-disposal) + [lifecycle proof](reviews/2026-09-28-quickjs-lifecycle-spike.md) |
| §6 values | [RFC §6](../spec/rfcs/0001-javascript-execution-binding.md#6-value-boundary) + [complete-value review](reviews/2026-09-28-quickjs-complete-value-spike.md) |
| Modules / paths / preflight | [RFC §1](../spec/rfcs/0001-javascript-execution-binding.md#1-package-and-module-model) + [capture](reviews/2026-09-28-quickjs-module-capture-spike.md) + [attribute follow-up](reviews/2026-09-28-javascript-import-attributes-spike.md) |
| Hardening / ambient authority | [Global surface](reviews/2026-09-28-quickjs-global-surface-spike.md) + [compiler denial](reviews/2026-09-28-quickjs-dynamic-code-suppression-spike.md) |
| RegExp | [Admission implementation](reviews/2026-09-29-quickjs-regexp-admission-impl-spike.md) + [integrated qualification](reviews/2026-09-30-quickjs-integrated-resource-failure-spike.md) |
| Async termination | [Async fix](reviews/2026-09-29-quickjs-async-termination-fix-spike.md) + [integrated qualification](reviews/2026-09-30-quickjs-integrated-resource-failure-spike.md) |
| Engine selection / patch ownership / platform containment | [Consolidated review](reviews/2026-09-29-consolidated-engine-feasibility.md) + [latest review](reviews/2026-09-30-quickjs-integrated-resource-failure-spike.md) |
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
