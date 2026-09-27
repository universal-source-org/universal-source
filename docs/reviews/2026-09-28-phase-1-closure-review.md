# Phase 1 closure review — 2026-09-28

Status: **Phase 1 closure accepted; v0.1 remains an experimental draft.** Phase 2 may begin with the static declarative profile. No runtime has been implemented or certified, and no version is stable or released.

This is a review and implementation handoff, not a second specification. The [normative prose and manifest schema](../../spec/README.md#reading-order-and-authority) remain authoritative. No normative wording, schema, example, or expected operation outcome changes in this audit.

## Scope and baseline

Reviewed together:

- [Specification index and manifest rules](../../spec/README.md), [Source API](../../spec/source-api.md), [Host API](../../spec/host-api.md), [lifecycle](../../spec/lifecycle.md), [compatibility](../../spec/compatibility.md), and [manifest schema](../../spec/schema/manifest.schema.json).
- [Minimal example](../../examples/json/minimal/README.md), both of its JSON files, [manifest validation and fixtures](../../conformance/README.md), and [declarative cases, schema, contexts, and consistency checks](../../conformance/declarative/README.md).
- [Charter](../PROJECT_CHARTER.md), [principles](../DESIGN_PRINCIPLES.md), [architecture](../ARCHITECTURE.md), [roadmap](../ROADMAP.md), [contribution workflow](../../CONTRIBUTING.md), [ADR process](../decisions/README.md), [ADR 0001](../decisions/0001-initial-architecture.md), and the [earlier scoped review](2026-09-28-declarative-contract-review.md).

The audited contract and corpus are the files at Git commit `d83adebc06474c0972ca9dafae77f04c5a320210`. The commit introducing this review records the closure assessment and phase transition; those contract and corpus files remain byte-for-byte unchanged. Retrieve that closure commit from the repository root with:

```sh
git log --diff-filter=A --format=%H -- docs/reviews/2026-09-28-phase-1-closure-review.md
```

Implementation reports should identify their target commit as well as `specVersion: "0.1"`. This identifies a reviewed draft, not a tag, release, certification, or promise that the draft will never change.

## Audit of required behavior

These findings explain existing requirements. The issue register below assigns exactly one disposition to each question needing explicit closure.

### Loading and validation

The [loading sequence](../../spec/lifecycle.md#1-validate-without-executing) requires UTF-8 JSON parsing, duplicate-member rejection, schema validation, semantic validation, exact version/engine support, contained existing entry resolution including symlinks, and finite package/entry limits before execution. Schema acceptance alone is insufficient: origin canonicalization and semantic duplicates, filesystem containment, and complete declarative shapes/references still need checks. Invalid static data is an invalid-entry load failure, not an executed call returning `INVALID_RESULT`.

Load diagnostics must distinguish invalid manifest, unsupported version, unsupported engine, unavailable required service, invalid entry, and resource limit. Their representation is host-facing and implementation-defined; Source API error envelopes do not apply. A schema error caused by an unsupported version or engine does not remove the requirement to distinguish that cause. A known engine unsupported by the implementation must also be refused. Several independent load faults have no prescribed diagnostic priority (D1 below).

Services and permissions are separate. Required services cannot be silently omitted; denied grants cannot be silently widened. A declarative source has no host services and, through the manifest's service/permission implications, cannot request cookies or storage. It can request network authority for returned URLs without `http`. Loading validates URL syntax as stored data; returning and consuming URLs enforce grants. No fetch, DNS lookup, playback, or implicit permission approval is needed to inspect the example. See A1 for the fixture boundary.

### Operations and precedence

The [invocation steps](../../spec/lifecycle.md#3-invoke-an-operation) and [Source API](../../spec/source-api.md) agree: ready-instance declaration checks precede input checks; inputs precede dispatch; result validation precedes delivery. Unknown input/result fields, invalid types, invalid IDs/queries/pages, and undocumented defaults are not permitted. Omitted category/search page alone defaults to 1; valid pages range through 9007199254740991. Query lookup is exact without trimming, folding, or normalization.

Static category existence is checked before handling a later page. Unknown category/detail/play IDs yield `NOT_FOUND`; unmatched search and later pages yield empty successful pages. A missing playable does not produce a URL to check, so denied grants do not turn that lookup into `PERMISSION_DENIED`. Undeclared-operation errors win over invalid input. Combined result-structure/origin violations remain U2 below; cancellation races are treated separately under lifecycle rules.

Success and failure envelopes are exclusive and closed. Error messages are nonempty and confidential, but their wording is not standardized. The ten codes are `INVALID_ARGUMENT`, `UNSUPPORTED_OPERATION`, `NOT_FOUND`, `PERMISSION_DENIED`, `NETWORK_ERROR`, `TIMEOUT`, `CANCELLED`, `RESOURCE_LIMIT`, `INVALID_RESULT`, and `SOURCE_ERROR`. Uncaught execution failures and malformed results have distinct codes. HTTP status failures alone are not transport failures. The static profile performs no HTTP requests, so its fixture corpus does not manufacture `NETWORK_ERROR` cases.

### Lifecycle and isolation

Readiness is required; disposal ends an instance and reuse requires a fresh load (observable misuse reporting is U1). Calls on one instance are serialized. No preceding `home` call is required. Different instances can run concurrently but cannot share mutable source memory. A returned mutable implementation object must not become a way to mutate another instance's source data.

Deadlines are finite; cancellation yields `CANCELLED`, expiration yields `TIMEOUT`, and a completion/cancellation race selects one terminal outcome and suppresses later ones. Cancellation stops further source access; work may be terminated or detached, with no rollback of effects already completed. Disposal cancels outstanding calls and prevents further execution. No automatic retries are allowed. Ordinary failure need not destroy a healthy instance, but an unsafe context after cancellation, timeout, or exhaustion must be disposed. Scheduling, quota values, and the health test are implementation policies within these obligations (D1).

The [isolation rules](../../spec/host-api.md#isolation-and-limits) separate persistent state by source and host user context. Sharing persistent state within one host-managed installation is distinct from sharing mutable instance memory. Source IDs are not publisher credentials or entitlement to another installation's state. Replacement requires a host-controlled update decision, full loading, and reevaluation of permissions; previous grants cannot authorize new requests. Disposal does not erase persistent storage. Storage retention/concurrency and cookie persistence need documented host policies; exact service bindings remain Phase 3 work. Phase 2's static profile needs instance isolation, not invented storage or cookie services.

### Network, resources, and versions

The [origin model](../../spec/host-api.md#capabilities-and-permission-grants) requires parsed normalized exact origins, not prefix matching. Declaration syntax excludes paths, even a trailing slash, credentials, wildcards, queries, and fragments; scheme/DNS names are lowercase, international DNS names use ASCII, IPv6 uses brackets, explicit ports are 1–65535 without leading zeros, and default ports are omitted. A resource URL's normalized default port does not create a different origin. A subdomain, different scheme, or non-default port needs its own request and grant. Schema patterns are intentionally coarse: malformed hosts, excessive ports, explicit default ports, and semantically equivalent origin spellings need semantic checks. Parser-profile edge cases are D2 below.

Returned posters and media are equally subject to requested and effective grants. A denied URL rejects the result; dropping a poster or returning an empty success is not the rule. Consumers recheck every access and redirect, including media subsystems, and refuse access if they cannot enforce this boundary. Returned headers apply only to the original resource origin, never another redirect origin. HTTP field-name syntax, case-insensitive uniqueness, CR/LF rejection, and the six prohibited transport/cookie headers apply as stated. Cookies cannot be attached or retained when denied or undeclared. Optional stricter destination policy also applies after DNS resolution and redirects. Phase 2 can resolve data without implementing a network consumer; it must not claim that result validation proves redirect enforcement by a future consumer.

The [version contract](../../spec/compatibility.md#version-contract) requires exact supported version and engine selection, rejection of unknown fields/operations/services, and no fallback, vendor-extension namespace, or lexical version inference. Draft changes are possible; finalized incompatible versions require the stated version increment and compatibility analysis. Nothing in the static profile needs an unspecified extension mechanism. Recognizing `javascript` is not implementing its unfinished binding; Rust is not a source format, and WASM is not a valid engine.

### Declarative completeness and authority

All five static operations, map shapes, references, envelopes, page construction, missing-key behavior, and exact matching are specified in [the normative static format](../../spec/source-api.md#minimal-declarative-entry-format). The example contributes data, not extra rules. Its category/content/detail/playable references and media origin agree with the specification. Subsets of operations are legal; the home-only context is valid, and references are required only when the target operation exists. Unreferenced entries are permitted. Opaque map keys must remain data, including names special to an implementation language.

Every current case is derivable from that contract and its explicit fixture preconditions. The [case-to-requirement table](../../conformance/declarative/README.md#cases-and-normative-basis) covers all 34 cases, including declaration-before-input precedence and missing-playable lookup with denied grants. Error message text and cancellation timing are not invented expectations. Structural JSON comparison preserves array order, types, and missing/null distinctions. Fixture schema definitions and Python consistency checks are aids for this corpus, not complete validators or source execution. A1 records a specific limitation of those checks.

No contradictory expected outcomes or example-only dispatch rules were found. Governance and ADR 0001 preserve the same five operations, language-neutral boundary, adapter policy, and draft status. The earlier review's unresolved findings are historical; their current dispositions are below. Neither reviews, plans, nor passing fixtures override normative prose or schema.

## Issue register and dispositions

**A** explains the existing intended contract without changing it. **B** explicitly defers a narrow question with current obligations and a revisit condition. **C** would block Phase 2. Each issue below has one disposition; there are no C issues.

### A1 — Fixture validation is narrower than semantic conformance

**Disposition: A — clarified from existing authority and documented test scope.** The current consistency checker requires all stored URLs in its two contexts to match the requested origin list. That is a corpus sanity assertion, not a general rule rejecting every source at load time because an unused stored URL lacks a grant. Whole-entry shape/reference validation and result-time permission checks are separate requirements. A denied permission need not prevent useful calls that return no affected URLs. The future loader must derive its rules from the specification, not copy this checker as a general validator. No test or normative change is needed: these contexts already satisfy the narrower assertion, and their documentation explicitly disclaims general URL/semantic validation.

### U1 — Observable outcome of calls on non-ready instances

**Disposition: B — defer portable misuse reporting to a lifecycle binding decision.** Current required behavior is to verify readiness, prevent execution in a disposed/non-ready instance, and require fresh loading after disposal. No portable error code or diagnostic representation for misuse is specified. The Phase 2 host-facing API must document and test its own misuse reporting; this report does not designate any Source API code as the required answer or add a new one.

This is outside the ready-instance operation domain. The harness must establish readiness and report setup failure separately from an expected operation outcome. A new portable lifecycle misuse fixture needs a subsequent specification decision. Choosing a binding or adding a sixth lifecycle operation now would be unjustified. The absence of portable misuse reporting does not prevent implementing the mandatory no-execution boundary or any current operation case.

### U2 — Malformed result and denied origin together

**Disposition: B — defer portable precedence for simultaneous result violations.** Both individual rejection requirements remain. A multiply invalid result cannot be delivered as success or consumed; the draft does not determine a unique error winner within invocation step 4. This review deliberately does not define an allowed-error set as a new conformance rule.

Static stored shape errors are rejected at load time; a correctly implemented static dispatcher constructs valid envelopes from validated data. Thus the ambiguity is not needed to derive normal static operation outcomes. Phase 2 must validate its own outputs and document any internal validation order, without claiming that order is portable. Revisit before standardizing combined-violation tests or the Phase 3 execution boundary. Internal fault tests may exercise rejection now; they cannot establish an unspecified universal precedence by assertion.

### P1 — Dynamic poster, media, and CDN origins

**Disposition: B — defer any permission-model expansion pending evidence.** Unrequested or ungranted origins still fail with `PERMISSION_DENIED`, including posters, media, and redirects. Revocation must take effect on use; neither source discovery nor a redirect expands grants. An explicit manifest update requires full reload and permission evaluation.

Changing CDN names may make advance exact-origin declarations impractical for some real sources, but this repository has no real-site corpus proving the frequency or severity. The [earlier evidence plan](2026-09-28-declarative-contract-review.md#design-pressure-p1--dynamically-discovered-resource-origins) remains the follow-up: shareable credential-free samples, origin predictability/rotation, redirect chains, poster and media failures, update feasibility, and adversarial destinations. Any broader model needs an RFC and versioning analysis. Phase 2 must enforce the existing rule; it may not introduce wildcard grants, automatic expansion, parent-domain trust, poster stripping, or arbitrary URL exceptions.

### D1 — Host policy and simultaneous lifecycle/load failures

**Disposition: B — defer portable policy values and unspecified diagnostic ordering.** The draft intentionally leaves load diagnostic format, finite quota values, storage concurrency/retention, cookie persistence, and stricter destination policy to hosts. It does not impose a total ordering among independent load defects or prescribe a winner for every concurrent cancellation/deadline/completion race. Single-cause load failures must remain distinguishable; ordered invocation checks and single terminal delivery still apply.

Phase 2 must publish its limits, diagnostic categories, race handling, and unsafe-instance disposal policy, and test those policies locally. Use isolated failures for portable expectations and identify implementation-specific assertions explicitly. There is no evidence for common quota numbers, a global scheduler, transactions, or a universal multi-fault priority table. Revisit only when independent implementations need a shared observable choice; persistent-service details remain in the existing Phase 3 handoff.

### D2 — Complete cross-implementation URL parsing profile

**Disposition: B — defer standardization of parser edge cases, not origin enforcement.** The current normalizing origin rules are mandatory and sufficient for the canonical origins/URLs in the corpus. The draft does not select a complete URL parsing/resolution algorithm for all unusual textual forms. The fixture checker's simple URL splitting is not such a selection, and schema acceptance cannot substitute for a real parser and semantic checks.

Phase 2 must document its parser and normalization policy and test ordinary DNS/IPv6 origins, default and non-default ports, semantic duplicates, and forbidden URL components. Parser differences outside explicit rules must be recorded as unresolved portability cases, not silently promoted to normative expectations. Any stricter access policy must be identified as host policy and preserve the specified denial behavior; it cannot authorize additional origins or skip required parsing. Choosing a universal service algorithm now would preempt the deferred URL binding work without comparative evidence. Revisit concrete disagreements as they appear, before claiming broader interoperability.

## Phase 2 implementation boundary

Allowed assumptions and choices:

- Target the static `declarative` profile only; reject unsupported engines and declare that scope. Select a reference implementation language through a scoped ADR; no language has been selected here.
- Consume a host-provided source root with a manifest and contained entry. Installation, signing, discovery, and trust UI are outside this draft; an ID alone is not trust.
- Provide an implementation-specific host-facing loader/call boundary, diagnostics, cancellation mechanism, quotas, and documented policies within the obligations and deferrals above. Do not present those bindings as additions to the portable source format.
- Run the existing cases under their explicit ready-instance, adequate-limit, no-cancellation, exact-grant preconditions. Resolve stored data and URLs without fetching them. Result grants still need enforcement on every call.
- Add semantic loading, containment, isolated-instance, limit, cancellation/disposal, and output-validation tests as the corresponding implementation exists. Clearly separate contract assertions from host-policy tests.

Phase 2 **MUST NOT invent** new source operations, defaults, error codes, extension fields, scraping expressions, JavaScript bindings, ambient capabilities, cross-origin authority, automatic retries, or a portable precedence that the draft has deferred. It must not use a fixture helper as a substitute for full semantic validation, alter authored outcomes to fit an implementation, or claim service, redirect-consumer, platform, or JavaScript conformance from static fixture results. New contract conflicts require explicit review rather than implementation-driven amendments. WASM, adapters, and platform apps remain outside Phase 2.

## Validation, limitations, and exit assessment

Validation performed for this closure:

- All six test groups passed with the existing pinned Draft 2020-12 validator: seven valid and 34 invalid manifest fixtures, the minimal manifest, the 34 operation cases, both source contexts, references, grants, and stored success snapshots.
- Repository JSON was parsed with duplicate-member and non-JSON-number rejection; both schema documents passed Draft 2020-12 metaschema checks. JSON examples in Markdown were parsed as well.
- Local documentation links and anchors, Markdown fences/table structure, final newlines, and whitespace were checked. External link syntax was checked; external availability was not tested.
- The complete diff was reviewed. Specification, schema, example, fixtures, validation code, dependencies, and ADR decisions are unchanged. Only review, navigation, phase-status documentation, and the task record change; no runtime or speculative engine/platform work was added.

Passing checks prove schema/corpus consistency, not execution. All 34 operation cases still need a Phase 2 harness. Semantic loader failures, non-ready reporting, cancellation/limits, state isolation, normalized-origin edge cases, and redirect consumers do not have an executing conformance suite yet. Host-service signatures, cookie/crypto/HTML/URL profiles, and JavaScript bindings remain deferred under the [Phase 3 handoff](2026-09-28-declarative-contract-review.md#deferred-phase-3-binding-scope). There is no real-site compatibility or cross-platform proof.

| Phase 1 exit criterion | Assessment and evidence |
| --- | --- |
| Joint review of operations, manifest, lifecycle, permissions, and compatibility; record issues | Met by this audit, A1 clarification, and explicit U1/U2/P1/D1/D2 deferrals. No Phase 2 blocker found. |
| Deterministic manifest and declarative cases, including failures and edges | Met by the existing 41 manifest fixtures and 34 operation cases; execution is Phase 2 work. |
| Repeatable standards-compliant schema validation, distinguished from semantics | Met by the documented unittest command, Draft 2020-12 checks, and explicit semantic limitations. |
| Example agreement and identification of cases awaiting execution | Met by source/schema inspection, consistency checks, and the case-to-requirement table; all operation cases await execution. |
| Deferred Phase 3 binding scope and identifiable reviewed draft baseline | Met by the linked handoff, issue register, audited Git commit, and this closure commit. |

**All Phase 1 exit criteria are met; Phase 1 is complete and Phase 2 is active.** This is readiness to implement a bounded reference runtime, not a claim of complete behavior coverage or stable interoperability. The smallest next task is a scoped reference-language ADR and a tested static source loader: manifest/schema/semantic checks, entry containment, complete stored-data/reference validation, finite load limits, and distinct host-facing diagnostics. Keep dispatch, the operation execution harness, and broader lifecycle execution for subsequent coherent units; do not mark Phase 2 complete after loading alone.
