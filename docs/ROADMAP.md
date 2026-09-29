# Roadmap

**Current active phase: Phase 3 — JavaScript runtime, beginning with binding design.** Phases 0, 1 and 2 are complete. The [Phase 1 review](reviews/2026-09-28-phase-1-closure-review.md) records the reviewed v0.1 draft baseline; the [Phase 2 closure review](reviews/2026-09-28-phase-2-closure-review.md) accepts the bounded Rust static declarative runtime and its host-policy boundaries. All 34 authored cases execute through managed instances. JavaScript execution is not implemented. v0.1 is not stable or released.

This roadmap sequences work, not release dates or permission to implement future phases. Keep completed work intact. Update phase status and supporting evidence when exit criteria are met. A phase transition is a deliberate repository update; it never implies a tag, release, push, or repository-setting change.

Specification work remains iterative across implementation phases. Exiting Phase 1 means having a reviewable and testable draft baseline, not declaring v0.1 stable or claiming JavaScript interoperability. Later evidence may lead to RFCs and versioned changes under the existing compatibility rules.

## Phase 0 — Repository foundation

Status: **Complete**.

Goals: establish durable project purpose, design constraints, current-state documentation, decision history, and a contributor/agent workflow in one monorepo.

Exit criteria:

- A concise AGENTS.md maps to authoritative context and the specification.
- The charter, principles, architecture, roadmap, initial ADR, and contribution workflow are recorded and consistent.
- Active and completed task-plan locations exist; local commit and explicit publication rules are documented.
- Documentation links and formatting have been checked, and the foundation is committed locally.

Evidence: [agent guide](../AGENTS.md), [contribution workflow](../CONTRIBUTING.md), and [ADR 0001](decisions/0001-initial-architecture.md).

## Phase 1 — Specification v0.1

Status: **Complete; reviewed experimental draft baseline**.

Goals: make the existing minimal contract internally consistent and independently testable without selecting a runtime language or expanding the five-operation scope.

Already present: English and Chinese introductions, [specification documents](../spec/README.md), [manifest schema](../spec/schema/manifest.schema.json), [static example](../examples/json/minimal/README.md), [RFC template](../spec/rfcs/0000-template.md), and [manifest conformance fixtures and command](../conformance/README.md). The command checks the schema, seven positive fixtures, 34 negative fixtures, and the example manifest using Draft 2020-12. The [34 declarative operation cases](../conformance/declarative/README.md) cover independent ready-instance calls; Phase 2 now executes all 34 in addition to structural/reference validation. The [scoped contract review](reviews/2026-09-28-declarative-contract-review.md) records example agreement, unresolved issues, dynamic-origin pressure, and deferred Phase 3 bindings.

Exit criteria:

- Review the existing operation, manifest, lifecycle, permission, and compatibility contracts together; record unresolved issues without silently redesigning them. **Complete in the [closure review](reviews/2026-09-28-phase-1-closure-review.md), including load diagnostics, cancellation, limits, multi-instance state, origin/URL semantics, version/extension handling, and explicit U1/U2/P1 deferrals. No Phase 2 blocker remains.**
- Add deterministic valid/invalid manifest fixtures and declarative input/expected-result cases under `conformance/`, including failures and edge cases. **Complete for the initial corpus: manifest fixtures and 34 declarative operation cases are present. Actual operation execution belongs to Phase 2.**
- Provide a documented, repeatable way to validate schema structure and manifest fixtures with a standards-compliant validator; distinguish schema checks from semantic checks. **Complete for the current manifest schema and corpus.**
- Verify the example against the documented contract and identify which cases require a future runtime harness. **Complete by static inspection and fixture consistency checks; all operation cases required Phase 2 execution at closure and now pass the Rust harness.**
- Record the scope of deferred JavaScript and host-service binding work for Phase 3. A reviewed draft baseline and its limitations are identifiable in Git. **Complete: the scoped review records the binding handoff, and the closure review identifies the audited Git baseline, closure commit, limitations, and allowed implementation choices. No normative contract changes were required.**

## Phase 2 — Declarative reference runtime

Status: **Complete for the bounded static declarative profile; closure accepted**.

Goals: implement the existing static declarative contract as the first small reference runtime. The reference implementation uses Rust under [ADR 0002](decisions/0002-reference-runtime-language.md); the specification remains language-neutral.

Use the [closure handoff](reviews/2026-09-28-phase-1-closure-review.md#phase-2-implementation-boundary) when designing the loader and harness. Deferred reporting/precedence/parser questions are not permission to invent portable behavior; document implementation policies and keep them distinct from contract assertions.

Completed unit: the [Rust loader](../runtime/README.md) validates manifests with the authoritative schema and semantic rules, resolves contained entries, validates all static data/references, enforces finite input budgets, and returns host-facing diagnostics. Loading tests and independent conformance checks pass. The second completed unit adds the five-operation dispatcher, result validation, exact returned-origin checks, and a production-runtime harness. The lifecycle unit adds instance-local serialization, cooperative deadlines/cancellation, exactly-one publication, quiescent disposal, checked invocation budgets and unsafe-context handling. All 34 unchanged authored cases execute through the managed path and pass. The [closure review](reviews/2026-09-28-phase-2-closure-review.md) explicitly assesses the implementation, limits and remaining host responsibilities and accepts this phase transition.

Exit criteria:

- Load and validate a source, dispatch its declared operations, and return contract-compliant results and errors.
- Enforce entry containment, permission checks, isolation, lifecycle behavior, and documented limits applicable to the static profile.
- Run the minimal example and the applicable conformance cases through a documented test harness with repeatable results.
- Document implementation scope and gaps. Do not add a scraping DSL, player UI, native apps, or WASM as part of this phase.

Current exit-criteria evidence:

- Loading/dispatch/results: **met**; schema and semantic validation, all five operations and envelope/error tests have been reviewed against normative requirements.
- Containment/permissions/isolation/lifecycle/limits: **met for bounded static execution**; resolved containment, exact grants, synchronization, interruption, disposal and budget tests support closure. Cooperative checkpoint latency, host-owned process memory and stable loading directories remain explicit host-policy boundaries. Network consumers remain separate future responsibilities.
- Example/conformance harness: **met**; all 34 unchanged authored cases pass through production managed instances with default finite limits; Python structural checks remain independent.
- Scope/gaps: **met**; the runtime guide, current architecture and closure review distinguish implementation, local policies, deferred portability questions and future work. Closure does not claim full v0.1, consumer or platform conformance.

## Phase 3 — JavaScript runtime

Status: **Active; binding RFC and fresh-realm lifecycle decision internally reviewed; QuickJS Model B lifecycle and pre-evaluation capture primitives demonstrated; no engine selected; remaining feasibility, normative adoption and production implementation pending**.

Goals: specify and implement controlled JavaScript execution while preserving the platform-neutral source contract.

Completed first design unit: [RFC 0001](../spec/rfcs/0001-javascript-execution-binding.md) proposes a restricted local ES module graph, named exports, full Source API envelope returns, synchronous module initialization, Promise completion, JSON-only value transfer and invocation-scoped capability injection. Its self-review preserves A1/U1/U2/P1/D1/D2 and Phase 2 resource limits. The [completed plan](plans/completed/2026-09-28-javascript-binding-rfc.md) records validation. The RFC is not accepted or implemented; engine feasibility and service profiles remain separate work.

The [engine evaluation](reviews/2026-09-28-javascript-engine-evaluation.md) compares QuickJS/QuickJS-NG, Boa, V8 and JavaScriptCore. The isolated QuickJS-NG spike proves several primitives but reproduces retained Promise/await reactions crossing the proposed terminal boundary. Outcome B: no engine selected, no engine ADR, and no RFC acceptance. The [completed plan](plans/completed/2026-09-28-javascript-engine-evaluation.md) records baseline and spike validation. Desktop evidence establishes no mobile or other-platform support.

The [focused Boa ownership review](reviews/2026-09-28-boa-reaction-ownership.md) records Outcome B. Public callback records preserve `.then` and `await` registration tokens, but opaque future jobs and absent handlers prevent complete pre-execution attribution; callback substitution changes child Promise state and can still execute source through species resolvers. That experiment left F1 unresolved under the persistent-realm proposal. Fourteen diagnostic tests pass in an isolated crate; no Boa engine selection, ADR, RFC amendment or production change follows. See the [completed plan](plans/completed/2026-09-28-boa-reaction-ownership.md).

The [lifecycle decision](reviews/2026-09-28-javascript-lifecycle-decision.md) compares Models A–E and chooses fresh realms per invocation, independent of the unchanged full-envelope return model. Logical instances reuse immutable snapshots, but mutable module/global/intrinsic state never crosses calls. Whole-realm retirement eliminates the proposed need for selective stale-reaction revocation; direct settlement observation, source quiescence and native delivery gating remain required. F1 now has an explicit proposed lifecycle resolution, not an engine implementation proof. RFC 0001 is narrowly revised and remains unaccepted; [validation](plans/completed/2026-09-28-javascript-lifecycle-decision.md) reruns unchanged baseline and experiment suites.

The [QuickJS fresh-realm lifecycle spike](reviews/2026-09-28-quickjs-lifecycle-spike.md) records **Outcome A**: public pinned rquickjs/QuickJS-NG APIs support the demonstrated Model B lifetime and terminal primitives using a dedicated runtime/context per invocation. Fifteen behavior tests and three compile-fail checks cover retirement, roots, pending/chained jobs, late delivery, setup failures, interruption and non-executing extraction. Context-only destruction is explicitly disproved by a negative control. No engine selection, RFC change or production behavior follows; the [completed plan](plans/completed/2026-09-28-quickjs-lifecycle-spike.md) records validation and limits. Complete module preflight, global hardening and other engine-selection evidence remain separate.

The [QuickJS module-capture spike](reviews/2026-09-28-quickjs-module-capture-spike.md) records **Outcome A — pre-evaluation capture viable**. Fourteen tests demonstrate a public native-module callback after graph linking and before any package evaluation, stable saved functions, replacement detection, dependency ordering and initialization-job abandonment. Direct `eval` alone returns too late; namespace reflection alone cannot reject aliases/re-exports and other forbidden declaration forms. No RFC or production change and no engine selection follows. The [completed plan](plans/completed/2026-09-28-quickjs-module-capture-spike.md) records validation. The next scoped evidence unit is recorded below; production execution remains unimplemented.

The [static-analysis spike](reviews/2026-09-28-javascript-static-analysis-spike.md) records **Outcome B — static preflight blocker**. Boa Parser 0.22.0 distinguishes the tested operation forms, escaped names and nested syntax while preserving exact bytes for pinned QuickJS compilation. Fifteen tests and a 62-case matrix expose loss of empty import/re-export attribute-clause presence in the public AST; Boa and QuickJS accept `with {}` although the RFC excludes it. Extra syntax-aware source validation or a lossless representation is required. No parser/engine selection, RFC amendment or production change follows; the [completed plan](plans/completed/2026-09-28-javascript-static-analysis-spike.md) records validation.

The [attribute-clause follow-up](reviews/2026-09-28-javascript-import-attributes-spike.md) records **Outcome A — attribute-clause preflight viable**. Supplemental Oxc 0.152.0 public syntax nodes preserve empty-clause presence and ownership on unchanged source bytes, closing the Boa-only gap without lexical false positives in the tested contexts. The combined experiment has 24 passing tests, with parser-error/disagreement rejection. This is not complete preflight or parser selection; RFC and production code remain unchanged. See the [completed plan](plans/completed/2026-09-28-javascript-import-attributes-spike.md).

The [dynamic compilation suppression spike](reviews/2026-09-28-quickjs-dynamic-code-suppression-spike.md) records **Outcome A — dynamic compilation suppression viable**. Fourteen tests cover 80 guarded eval/constructor/alias/Reflect routes, mutation, host compilation/capture ordering and fresh-realm reset. Public object-model guards preserve tested function-family relationships and throw EvalError before compilation; negative controls establish why global deletion, late hardening and missed constructor-parent links fail. No engine selection or complete sandbox claim follows. See the [completed plan](plans/completed/2026-09-28-quickjs-dynamic-code-suppression-spike.md).

The [restricted-global spike](reviews/2026-09-28-quickjs-global-surface-spike.md) records **Outcome A — restricted-global surface viable**. Nineteen tests establish the exact 38-name global allowlist, tested intrinsic/stack/clock/randomness/locale removal or guards, bounded forbidden-identity traversal, host-boundary isolation, mutation resistance and fresh-realm reset. All 80 prior dynamic-code routes remain denied. This is not a complete sandbox or engine selection; the [completed plan](plans/completed/2026-09-28-quickjs-global-surface-spike.md) records unchanged baseline/prior-suite validation. RFC 0001 remains proposed.

The [value-boundary spike](reviews/2026-09-28-quickjs-value-boundary-spike.md) records **Outcome B — blocker demonstrated** under its no-unsafe-Rust constraint: pinned rquickjs lacks a safe positive ordinary-object class query before reflection. A prototype-mutated Map defeats prototype-only admission; naive property iteration and native brand stringification execute source getters. Six behavior tests and two compile-fail checks preserve the counterexamples. Public C class/descriptor APIs exist, but require a separately reviewed safe binding. That constrained spike did not prove complete §6 conversion; no prior evidence or RFC rule is weakened. The [completed plan](plans/completed/2026-09-28-quickjs-value-boundary-spike.md) records all regression validation.

The [audited class bridge](reviews/2026-09-28-quickjs-class-bridge-spike.md) records **Outcome A — audited public-API class bridge viable** with explicitly permitted unsafe Rust confined to one public `JS_GetClassID` call. Ten tests distinguish ordinary object/array classes from prototype-mutated native objects, proxies and an opaque Rust callback without source hooks. No private IDs/layout are used; new distinct native classes fail closed. The earlier no-unsafe blocker evidence remains valid and unchanged. That unit closed only classification; the subsequent complete boundary evidence follows below.

The [complete value-boundary spike](reviews/2026-09-28-quickjs-complete-value-spike.md) records **Outcome A — complete RFC §6 value boundary feasible**. Twenty tests demonstrate bounded non-executing conversion in both directions using the unchanged class bridge, class-gated public descriptors, strict Unicode, host-held helper identities, dense arrays, path-cycle rejection, repeated-reference expansion and incoming own data definitions. Working-path hooks/proxy traps remain uncalled; three additional audited public-API unsafe blocks are isolated in the experiment. All prior evidence remains unchanged. Portable transfer budgets do not prove engine CPU/heap/RegExp enforcement; production JavaScript remains unimplemented.

The [resource-enforcement spike](reviews/2026-09-29-quickjs-resource-spike.md) records **Outcome B — blocker demonstrated**. Eleven tests show local CPU/module/job interruption, controlled heap rejection, bounded recursion and RegExp matching interruption. A separate local forward-backreference compiler reproducer performs long native work with zero interrupt callbacks; its larger case requires an external safety kill despite an 8 MiB engine limit. The kill is not engine-control or teardown evidence. Full resource classification and the remaining exhaustion matrix stay open; scope expansion stopped at this blocker. All prior evidence is preserved; see the [completed plan](plans/completed/2026-09-29-quickjs-resource-spike.md).

The [RegExp compilation remedy review](reviews/2026-09-29-quickjs-regexp-admission-review.md) records **Outcome A — pre-admission viable at design level**, without changing that blocker. RFC 0001 requires finite published bounds and eventual interruption, not mid-compile polling. A host-published pattern length bound can make each non-polling compile a bounded slice: a byte-level preflight for literals, plus a trusted gate with deadline checks for dynamic construction. The pinned compile paths audited are at most quadratic. An eight-test [route probe](../experiments/quickjs-regexp-admission-spike/README.md) finds a finite wrap set: the constructor, `String.prototype.match`/`matchAll`/`search`, `RegExp.prototype.compile`, and generic-receiver `@@split`/`@@matchAll`. Nothing is implemented. Process containment is the ranked fallback; no upstream remedy is locally evidenced. See the [completed plan](plans/completed/2026-09-29-quickjs-regexp-admission-review.md).

The [RegExp admission implementation spike](reviews/2026-09-29-quickjs-regexp-admission-impl-spike.md) records **Outcome B — a gated stop is not guaranteed to stop source in-process**. Sixteen tests implement the facade, the trusted gate and exact-byte literal preflight. All 30 inventoried dynamic routes, including the 64,000-reference reproducer, are refused with zero native compiles. The native constructor is unreachable, coercion follows ES2023 order, and the worst compile at a candidate 4,096-unit bound is about 81–86 ms (debug). However, pinned promise machinery (the `Promise` executor, thenable resolution reached by every `await`, `Promise.all`, async generators) converts uncatchable errors into rejections. That absorbs both gate stops and engine deadline interrupts, so source can keep running after a latched stop. `await` ignores the global `Promise`, so no JavaScript facade can close the path. The finding qualifies in-process CPU enforcement generally on this pin; the resource Outcome B and its reproducer are unchanged. See the [completed plan](plans/completed/2026-09-29-quickjs-regexp-admission-impl-spike.md).

The [process-containment and async-termination review](reviews/2026-09-29-process-containment-and-termination-review.md) records **Outcome C — a bounded engine-level fix is the more plausible portable path**. Disposable helper-process containment restores hard termination and maps cleanly onto Model B, §6 transfer, host-service injection and trusted parent-side kill-reason classification, but that review overbroadly excluded ordinary iOS/iPadOS/tvOS App Store applications (corrected by the consolidated review below); it is straightforward on Linux/macOS/Windows and achievable on Android via isolated-process services. The remaining in-process termination blocker (Promise/async stop-swallowing) is plausibly one small, enumerable engine defect: `JS_IsUncatchableError` is checked in only four places, and `promise_reaction_job` already carries the exact guard the sibling exception-to-rejection conversion sites omit. Combined with the already-working in-process RegExp pre-admission, a bounded fix would restore §§5/7/10 termination in process on every platform. Both blockers still reproduce; no engine, RFC, dependency or ADR change was made. See the [completed plan](plans/completed/2026-09-29-process-containment-and-termination-review.md).

The [async-termination fix and version review](reviews/2026-09-29-quickjs-async-termination-fix-spike.md) records **Outcome B — a bounded engine change completely fixes the swallow for the RFC 0001 profile, and no published upstream revision contains it**. An isolated throwaway experiment (SHA-verified copy of the pinned engine, gitignored baseline/patched builds, ASan+UBSan and QuickJS leak-abort) applies 12 guard edits across 11 Promise/async functions, each reusing the `JS_IsUncatchableError` pattern `promise_reaction_job` already carries. Every uncatchable route — native admission-gate throw and latched deadline interrupt, across the executor, thenable resolution/jobs, `Promise.all`/`race`/`try`, async generators and `for await` — terminates in process while ordinary exceptions stay catchable, with a healthy fresh runtime and no leaks. The async-generator route needed the conversion guard **plus** propagation from the void helper's two callers, a nuance corroborated by [bellard/quickjs#341](https://github.com/bellard/quickjs/issues/341). Current upstream `master` (0.17.0) is byte-identical to the pin at every affected site, so upgrading would not fix it. The committed dependency is untouched; no engine, RFC, pin or ADR change was made, and no fork is authorised. See the [completed plan](plans/completed/2026-09-29-quickjs-async-termination-fix-spike.md).

Exit criteria:

- Review an explicit execution binding covering modules, exports, asynchronous calls, initialization, and host-service injection. **Initial draft and F1 fresh-realm revision internally reviewed in RFC 0001; embedding feasibility and acceptance remain pending.**
- Define and test the exact host-service signatures, value encodings, and algorithm profiles needed to claim support; report unavailable required services explicitly.
- Demonstrate sandboxing, denied authority, state isolation, cancellation, and resource-limit behavior with deterministic cases.
- Execute representative JavaScript sources through the same source contract; document actual service and platform support without broad interoperability claims unsupported by evidence.

## Phase 4 — Legacy compatibility

Status: **Planned; adapter boundary documented**.

Goals: prove useful migration paths using selected, shareable samples from legacy ecosystems such as TVBox, drpy, XBPQ, and XYQ.

Exit criteria:

- Deliver at least one bounded importer or adapter with an explicit supported input/version scope.
- Test mappings into the existing core contract and verify that permissions cannot be bypassed.
- Publish repository documentation of supported behavior, transformations, rejected features, and any platform restrictions; do not promise universal conversion.
- Keep legacy-native concepts and execution requirements within adapters, with no implicit additions to the core API.

## Phase 5 — Android + Apple cross-platform proof

Status: **Planned; no platform integration**.

Goals: demonstrate that unchanged source packages behave consistently on Android and an Apple platform before making broader platform-support claims.

Exit criteria:

- Run the same representative declarative and JavaScript sources and applicable conformance cases on Android and at least one named Apple target.
- Compare results, errors, and permission enforcement; record platform versions, harnesses, supported services, and remaining differences.
- Keep platform bindings behind the contract and use small proof harnesses, not production apps or a player UI.
- State precisely what was demonstrated. Android TV, iPadOS, tvOS, macOS, Windows, and Linux support require their own evidence where not covered by the actual proof.

## Phase 6 — WASM research

Status: **Exploratory and deferred; no architecture or implementation**.

Goals: investigate whether real source requirements justify another engine after the initial engines and portability work provide evidence.

Exit criteria:

- Record concrete use cases, portability constraints, security implications, implementation costs, and alternatives.
- Reach a documented go/no-go decision; deciding not to add WASM is a valid research outcome.
- Any proposal to standardize or implement an engine has a separate RFC, versioning analysis, and explicitly scoped follow-up. Research completion does not add `wasm` to the v0.1 manifest.

## Recommended next coherent task

The [consolidated engine-feasibility review](reviews/2026-09-29-consolidated-engine-feasibility.md) records **Outcome A — continue QuickJS toward selection**. A narrow maintained async-termination patch and permanent RegExp admission policy are defensible with explicit ownership and upgrade audits; JSC, V8 and Boa offer no demonstrated better complete architecture. Current Apple ExtensionFoundation documentation makes system-managed containment plausible on newer mobile/TV OS versions, correcting the earlier blanket exclusion, without proving RFC termination or authorizing IPC. See the [completed plan](plans/completed/2026-09-29-consolidated-engine-feasibility.md).

Next, run one **non-production QuickJS integrated resource/failure closure spike** using the existing pinned engine and experiment-local patch. Combine actually invoked hardening, RegExp admission, §6 transfer and lifecycle retirement through one correctly linked patched Rust binding. Test trusted allocation/stack/native-work classification, initialization/calls/jobs, ordinary versus uncatchable failures, no continuation or late delivery, and fresh-runtime health. Determine whether this composes without another substantive engine change. The [exact scope and pass/fail boundary](reviews/2026-09-29-consolidated-engine-feasibility.md#exact-next-task) govern this unit.

A pass closes the resource-composition gate only. Complete module/path containment, final parser/preflight strategy, multi-platform/Apple embedding feasibility and patch ownership/integration strategy must also precede an engine-selection ADR. Selection is not yet justified; no engine/parser, maintained fork, RFC acceptance or production integration is authorized. Host-service signatures, full conformance and optional containment implementation remain separate later work.
