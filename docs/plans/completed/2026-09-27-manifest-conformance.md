# Manifest conformance workflow

Status: Completed

## Scope

Add representative schema-valid and schema-invalid manifest fixtures and one local validation command using a real JSON Schema Draft 2020-12 validator. Include the existing minimal example manifest. This advances Phase 1 without changing the v0.1 contract.

Use Python's standard unittest runner with a pinned `jsonschema` development dependency in an isolated virtual environment. This is conformance tooling, not a choice of source-engine or reference-runtime implementation language. Keep fixtures as plain JSON and document the boundary between schema validation and semantic source validation.

## Non-goals and stop condition

Do not change `spec/`, implement source execution, add compatibility adapters, or build platform apps. If a genuine contradiction between schema and normative prose is discovered, stop and report it rather than changing either contract.

## Work

1. Review the current schema and corresponding manifest/host requirements.
2. Add valid and invalid fixtures covering engines, capabilities, permissions, version, paths, and closed object shapes.
3. Add a repeatable validator command, isolated dependency setup, and fixture documentation.
4. Update contribution guidance, current architecture, and Phase 1 progress without claiming runtime conformance.
5. Run validation and failure-path checks, check links/formatting, review the complete diff, complete this plan, and create one local Conventional Commit. Do not push.

## Validation

- Validate the schema against the Draft 2020-12 metaschema using `jsonschema`.
- Require all positive fixtures and the example manifest to pass; require every negative fixture to be valid JSON rejected by the schema.
- Fail if a fixture group is empty, JSON is malformed, or expectations are reversed.
- Verify failure reporting with temporary copies, leaving repository fixtures intact.
- Confirm specification/example content is unchanged; check documentation links and Git whitespace checks.

## Results

- Added seven schema-valid and 34 schema-invalid standalone manifests, with each case documented in the conformance guide.
- Added one unittest command using `jsonschema==4.26.0`, including Draft 2020-12 metaschema validation and the existing minimal example manifest. No specification or example content changed, and no contradiction was found in this work.
- Validation passed on Python 3.14.7: all 42 manifests had the expected result across three test groups, and `pip check` reported no broken requirements.
- Ten failure-path checks in temporary copies confirmed nonzero exits for reversed expectations, malformed JSON, duplicate members, non-JSON constants, empty/missing fixture groups, missing/invalid example, and invalid schema. The real corpus also passed with network connections blocked.
- Updated contribution instructions, README navigation, current architecture, and Phase 1 progress. Validation covers manifest structure, not entry-file semantics or source execution.
- Reviewed the full diff and checked local documentation links, Markdown/JSON formatting, unchanged specification/example files, and Git whitespace. No runtime, adapter, or app work was added.

## Remaining Phase 1 work

Add declarative operation input/expected-result cases; finish the cross-document contract review and example behavior/reference checks; record unresolved semantics and deferred JavaScript/host-service bindings for Phase 3. Keep Phase 1 active. See the [roadmap](../../ROADMAP.md) and [validation guide](../../../conformance/README.md).
