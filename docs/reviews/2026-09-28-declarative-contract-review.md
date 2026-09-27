# Declarative contract review — 2026-09-28

Status: scoped Phase 1 review; observations and unresolved issues, **not normative changes or accepted design decisions**.

Reviewed the [specification index](../../spec/README.md), [Source API](../../spec/source-api.md), [Host API](../../spec/host-api.md), [lifecycle](../../spec/lifecycle.md), [compatibility](../../spec/compatibility.md), manifest schema, and [minimal example](../../examples/json/minimal/README.md) for independent static declarative calls. The [34 operation cases](../../conformance/declarative/README.md) record only outcomes determined by these documents. No specification or example files were changed.

## Agreement demonstrated by inspection and fixture checks

- The manifest declares precisely the entry's five operations, requires no host services, and requests the media origin independently of the `http` capability. This matches static data resolution.
- Home category `all` exists in `category`; home/category/search content ID `demo` exists in `detail`; detail ID equals its key; playable `demo-main` exists in `play`. IDs are unique in their arrays. The stored title, description, and resource match the authored success snapshots.
- Only the play result contains a URL. Its exact origin matches the manifest request. Denying that grant yields `PERMISSION_DENIED` on known play resolution; static home results without URLs can still succeed. No network fetch is needed to inspect a URL or resolve static data.
- Page defaults, bounds, empty pages, exact search, unknown IDs, and declaration-before-input validation agree across the Source API, lifecycle, and example explanation. The maximum page is preserved numerically in JSON.
- Error codes can be compared exactly. Diagnostic message wording is deliberately unconstrained beyond the stated nonempty/confidentiality requirements, so fixtures do not assign fixed text.
- The engine and permission boundaries in the governance documents remain consistent with the specification: no runtime exists, and an accepted manifest is not proof of engine support.

There is no contradiction blocking the requested ready-instance cases. Fixture checks establish structural and reference consistency, not actual execution or exhaustive contract conformance.

## Unresolved contract issues excluded from expected outcomes

### U1 — Calls on a non-ready instance

The [invocation lifecycle](../../spec/lifecycle.md#3-invoke-an-operation) requires verifying readiness. The disposal rules require a fresh load before subsequent calls, but neither the Source API error list nor the lifecycle defines a specific observable outcome for an attempted call before readiness or after disposal. It is unclear whether this is an implementation-defined host misuse diagnostic, a source-call envelope, or another explicitly documented boundary.

No `SOURCE_ERROR`, `UNSUPPORTED_OPERATION`, or new error code is assigned here. All operation fixtures explicitly assume a ready instance. Before a lifecycle misuse fixture or harness behavior is standardized, record an explicit disposition through the existing review/RFC process. This is an actual missing outcome for an out-of-scope lifecycle condition, not a reason to reinterpret ready calls.

### U2 — Multiple simultaneous result violations

Lifecycle invocation step 4 names `INVALID_RESULT` for invalid structure and `PERMISSION_DENIED` for denied URL access without specifying which wins when a result violates both. The ordered invocation steps establish declaration-before-input validation, but do not establish precedence within step 4.

These fixtures use structurally valid stored data and isolate permission denial; they do not choose a winner for combined violations. Invalid stored declarative data is also a load-time validation problem, not automatically a call-time `INVALID_RESULT`. Future result-validation tests need a documented precedence decision or an explicitly permitted set of outcomes. No such contract is added here.

## Design pressure P1 — Dynamically discovered resource origins

The current rule is unambiguous: returned poster/media URLs outside requested and effective exact-origin grants must be rejected. Grants cannot exceed requests; wildcard requests are forbidden. [Manifest updates](../../spec/lifecycle.md#5-dispose) require loading and permission evaluation again. A redirect is subject to another permission check, not an implicit extension of trust.

A future real-world source may discover a resource on a changing CDN hostname or a third-party poster host only after resolving content. A manifest author might not know every possible exact origin in advance. Under the current model, an unlisted returned origin must fail even if source parsing and content identification succeeded. A denied poster can therefore reject a content result as well as a denied media URL rejecting `play`; silently stripping posters is not the stated rule. This is a potential compatibility constraint, not a demonstrated flaw or permission to change the model.

Evidence needed: legally shareable, credential-free source samples showing requested versus discovered origins, host rotation and redirect chains, predictability of origin sets, observed frequency of failures, and whether explicit manifest updates suffice. Test poster and media paths separately and include adversarial destinations. There is no real-site corpus or platform evidence for this pressure yet.

Do not introduce wildcard access, automatic grants, inferred trust from a parent domain, arbitrary URL exceptions, or a new permission API through fixtures. Any proposed resolution requires a future RFC with evidence, denial behavior, portability, security boundaries, and versioning analysis. Until then the exact-origin rule remains authoritative.

## Deferred Phase 3 binding scope

The following are already identified as incomplete in the specification. This record groups the handoff scope; it defines no signatures or JavaScript binding.

| Area | Work to specify and prove later |
| --- | --- |
| JavaScript execution | Module/entry loading, export discovery, initialization, asynchronous completion, value transfer, and controlled service injection. |
| HTTP and cookies | Exact request/response signatures and byte encodings, transport/header handling, cookie algorithms and persistence profile, cancellation, and redirects. |
| Storage | Exact operations, missing-key representation distinct from JSON null, quotas, and concurrent access semantics. |
| HTML | Parser behavior, selector coverage, returned parser values, and inertness guarantees. |
| JSON and URLs | Exact service signatures, serialization/parse behavior and URL parsing/resolution profile across implementations. |
| Crypto | Supported primitives and algorithms, input/output encodings, and unsupported-algorithm errors. |
| Logging | Callable shape, diagnostic values and limits, and filtering behavior within the existing privacy rules. |

Phase 2 can target the static profile without inventing these services. Recording this scope does not establish interoperability or change the initial engine vocabulary.

## Phase 1 review boundary and follow-up

The ready-instance operation corpus and static example agreement are complete for this task. All cases need Phase 2 execution later; that is not a Phase 1 implementation requirement. Load diagnostics, cancellation races, resource limits, multi-instance state, the full origin/URL semantic boundary, and version/extension handling still need a consolidated cross-document review. Their review may produce additional issues or confirm that implementation-defined behavior is intentional.

The next coherent task is that remaining consistency review and an explicit draft-baseline readiness record, including the disposition or documented deferral of U1/U2 and P1. Do not treat this scoped review as acceptance of a stable v0.1 release. The [roadmap](../ROADMAP.md) remains the phase-status authority.
