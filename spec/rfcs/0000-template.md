# RFC 0000: <Title>

- Status: Draft
- Authors: <Names or handles>
- Created: <YYYY-MM-DD>
- Target specification: <Version or draft revision>
- Discussion: <Issue or pull request link>
- Supersedes / superseded by: <None or RFC link>

Copy this template to a new numbered RFC. An RFC proposes a change; its presence does not amend the specification. Record acceptance and update the affected normative documents and fixtures together.

## Summary

Describe the concrete problem and proposed observable behavior in a few sentences.

## Motivation and scope

Who needs this, and what real source demonstrates the need? State non-goals and why the existing contract is insufficient.

## Proposed contract

Specify inputs, outputs, errors, defaults, and lifecycle effects. Use MUST, SHOULD, and MAY where appropriate. Include a minimal source or manifest example. Keep implementation language and platform choices out of normative behavior.

## Compatibility and versioning

Explain old-source/new-runtime and new-source/old-runtime behavior, schema changes, migration, and whether a new specification version is needed. Describe effects on declarative sources, JavaScript sources, and legacy adapters separately where relevant.

## Host capabilities and permissions

List new authority, isolation requirements, resource limits, and denial behavior. Explain how adapters and indirect network access obey the same boundary.

## Portability and alternatives

Explain how independent implementations on mobile, TV, and desktop can agree. Compare smaller alternatives, including leaving the feature outside the core or making no change.

## Validation

Provide deterministic positive and negative cases, including edge cases and expected errors. Identify proposed conformance fixtures; do not rely only on live-site tests.

## Implementation considerations

Describe feasibility and costs without prescribing a reference runtime's language. Note any dependencies or platform assumptions the proposal would introduce.

## Open questions

List unresolved semantics and evidence needed before acceptance.

## Decision

Record the outcome, rationale, discussion link, and follow-up specification changes when decided.
