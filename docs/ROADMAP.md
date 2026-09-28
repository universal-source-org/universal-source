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

Status: **Active; binding RFC drafted and internally reviewed; initial engine evaluation complete with no selection; normative adoption and production implementation pending**.

Goals: specify and implement controlled JavaScript execution while preserving the platform-neutral source contract.

Completed first design unit: [RFC 0001](../spec/rfcs/0001-javascript-execution-binding.md) proposes a restricted local ES module graph, named exports, full Source API envelope returns, synchronous module initialization, Promise completion, JSON-only value transfer and invocation-scoped capability injection. Its self-review preserves A1/U1/U2/P1/D1/D2 and Phase 2 resource limits. The [completed plan](plans/completed/2026-09-28-javascript-binding-rfc.md) records validation. The RFC is not accepted or implemented; engine feasibility and service profiles remain separate work.

The [engine evaluation](reviews/2026-09-28-javascript-engine-evaluation.md) compares QuickJS/QuickJS-NG, Boa, V8 and JavaScriptCore. The isolated QuickJS-NG spike proves several primitives but reproduces retained Promise/await reactions crossing the proposed terminal boundary. Outcome B: no engine selected, no engine ADR, and no RFC acceptance. The [completed plan](plans/completed/2026-09-28-javascript-engine-evaluation.md) records baseline and spike validation. Desktop evidence establishes no mobile or other-platform support.

Exit criteria:

- Review an explicit execution binding covering modules, exports, asynchronous calls, initialization, and host-service injection. **Initial draft and internal review complete in RFC 0001; acceptance and feasibility evidence remain pending.**
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

Resolve the evaluation's [F1 retained-reaction ownership issue](reviews/2026-09-28-javascript-engine-evaluation.md#rfc-feedback-and-smallest-follow-up) before production execution. Run only a focused Boa callback/job ownership experiment for queued and unqueued `.then`/`await` reactions across synthetic invocations, and review whether RFC 0001 can be implemented faithfully through supported hooks. If not, document the necessary engine extension or independently justified RFC revision; do not silently relax terminal timing or persistent instance state. Engine selection still requires the remaining control/hardening/build evidence. Keep RFC acceptance explicit and preserve A1/U1/U2/P1/D1/D2 and Phase 2 resource limits. No full JavaScript runtime or host-service implementation is authorized by this follow-up.
