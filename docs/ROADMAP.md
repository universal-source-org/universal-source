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

Perform one **complete non-executing value-conversion feasibility proof** against RFC §6 on the pinned public APIs. Extend the earlier bounded copier evidence to full string/surrogate and key handling, native/helper-brand rejection, expanded-value accounting and hostile mutation without invoking getters, proxies, coercion, thenables or serialization hooks. The [restricted-global review](reviews/2026-09-28-quickjs-global-surface-spike.md#required-functionality-tests-and-remaining-gap) closes the tested ambient-surface gap only. Do not implement production execution, select an engine or accept the RFC. Resource/CPU/memory/regexp, module/path containment, parser selection, services, platform/distribution and conformance evidence remain separate.
