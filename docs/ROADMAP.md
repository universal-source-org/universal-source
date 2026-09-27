# Roadmap

**Current active phase: Phase 1 — Specification v0.1.** Phase 0 is complete with the repository governance foundation. Phase 1 already has a substantial draft, manifest schema, RFC template, and static example; it is not starting from zero.

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

Status: **Active; draft foundation present**.

Goals: make the existing minimal contract internally consistent and independently testable without selecting a runtime language or expanding the five-operation scope.

Already present: English and Chinese introductions, [specification documents](../spec/README.md), [manifest schema](../spec/schema/manifest.schema.json), [static example](../examples/json/minimal/README.md), and [RFC template](../spec/rfcs/0000-template.md). Exact JavaScript and host-service bindings remain documented gaps.

Exit criteria:

- Review the existing operation, manifest, lifecycle, permission, and compatibility contracts together; record unresolved issues without silently redesigning them.
- Add deterministic valid/invalid manifest fixtures and declarative input/expected-result cases under `conformance/`, including failures and edge cases.
- Provide a documented, repeatable way to validate schema structure and manifest fixtures with a standards-compliant validator; distinguish schema checks from semantic checks.
- Verify the example against the documented contract and identify which cases require a future runtime harness.
- Record the scope of deferred JavaScript and host-service binding work for Phase 3. A reviewed draft baseline and its limitations are identifiable in Git.

## Phase 2 — Declarative reference runtime

Status: **Planned; no implementation**.

Goals: implement the existing static declarative contract as the first small reference runtime. Choose an implementation language through a scoped decision; Rust is a candidate, not a prerequisite.

Exit criteria:

- Load and validate a source, dispatch its declared operations, and return contract-compliant results and errors.
- Enforce entry containment, permission checks, isolation, lifecycle behavior, and documented limits applicable to the static profile.
- Run the minimal example and the applicable conformance cases through a documented test harness with repeatable results.
- Document implementation scope and gaps. Do not add a scraping DSL, player UI, native apps, or WASM as part of this phase.

## Phase 3 — JavaScript runtime

Status: **Planned; engine name exists, binding and implementation do not**.

Goals: specify and implement controlled JavaScript execution while preserving the platform-neutral source contract.

Exit criteria:

- Review an explicit execution binding covering modules, exports, asynchronous calls, initialization, and host-service injection.
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

Add an initial manifest conformance corpus and a repeatable schema-validation command, checking valid and invalid cases against the existing schema without changing the contract. Document semantic checks the schema cannot express and existing validator/tooling gaps. This advances Phase 1; runtime implementation remains Phase 2 work.
