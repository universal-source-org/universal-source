# Declarative operation conformance cases

Status: Completed

## Scope

Record independent operation inputs, effective grants, and expected outcomes for the existing v0.1 static declarative profile. Reuse the minimal example and add a small home-only source for empty and undeclared-operation cases. Keep fixture metadata separate from the source contract.

## Work

1. Review source, host, lifecycle, manifest, and compatibility requirements and the example.
2. Add plain JSON cases for all five operations, defaults, empty results, input errors, unknown IDs, undeclared operations, and permission denial.
3. Validate fixture structure, source references, and stored expected data without dispatching or executing operations.
4. Record cross-document findings, unresolved semantics, deferred bindings, and dynamic-origin design pressure without changing the specification.
5. Run the complete conformance command, negative checks for fixture validation, link/format checks, and review the entire diff. Move this plan to completed and create one local Conventional Commit; do not push.

## Boundaries

No specification redesign, JavaScript binding, runtime, adapter, app, or new dependency. Do not invent fixed diagnostic message text. Ambiguous outcomes are recorded as issues instead of executable expectations. Phase 2 must execute these cases later; passing the current tooling proves fixture consistency only.

## Validation and results

- Added 34 independently authored cases: home 4, category 11, search 12 (including two undeclared-operation cases), detail 3, and play 4. The original example is referenced directly; a small home-only context supplies empty and undeclared cases.
- Added a fixture-only JSON Schema and static consistency checks for context manifests, contained entry files, stored shapes/references, grants, and expected-data snapshots. No dispatcher, evaluator, or source-operation execution was introduced.
- All six conformance test groups passed, including the existing 41 manifest fixtures and example. The new corpus and its two source contexts passed fixture validation; `pip check` passed.
- Nine temporary-copy failure checks passed: duplicate case IDs, unknown context, excess grants, invalid success shape, conflicting expectations, snapshot drift, broken playable reference, missing entry, and incorrect page label all caused test failure.
- Local links/anchors, Markdown structure, JSON parsing, documented case coverage, and whitespace were checked. The full diff was reviewed. Specification and original example files remain unchanged.
- Recorded U1 (non-ready invocation outcome), U2 (simultaneous result-structure/permission violations), and P1 (dynamic poster/media origins under exact-origin permissions). None changes the deterministic ready-instance expectations or authorizes contract redesign.
- Updated current architecture, contributor guidance, conformance navigation, and roadmap progress. Recorded deferred Phase 3 binding scope in the review.

## Remaining Phase 1 work

Complete the consolidated review of load diagnostics, cancellation, resource limits, multi-instance state, full origin/URL semantics, and version/extension handling, including explicit disposition or deferral of U1/U2 and P1. Then record the reviewed draft baseline and its limitations in Git. All selected operation cases still require Phase 2 execution; no runtime work is required to finish Phase 1. See the [review](../../reviews/2026-09-28-declarative-contract-review.md) and [roadmap](../../ROADMAP.md).
