# Declarative operation cases

[`cases.json`](cases.json) contains **34 independent calls** with authored expected outcomes for the existing [v0.1 Source API](../../spec/source-api.md). The Rust execution harness now runs these unchanged test data through the production loader and dispatcher. All selected outcomes are deterministic under the preconditions below; no ambiguous behavior is assigned an invented result.

## Context and execution preconditions

The `sources` map names repository-relative manifest paths. `minimal` refers directly to the unchanged [minimal example](../../examples/json/minimal/README.md), not a copied source. `empty-home` refers to the [home-only manifest](sources/empty-home/manifest.json) and [entry](sources/empty-home/source.json) added here. Entry files are resolved relative to their manifests, as specified. These two contexts contain no scripts or host-service requirements.

Each case assumes a fresh, successfully validated, ready declarative instance, adequate resource limits, no cancellation, and no host restrictions beyond its explicit effective `grants`. The harness supports this profile and arranges these preconditions; failure to arrange them is a harness/setup failure, not a different expected operation result. Calls are independent, not a sequence. In particular, `detail` and `play` require no earlier `home` or other operation.

`grants` repeats the existing permission shape (`network`, `cookies`, `storage`) as **fixture context**, not a new Source API argument. Grants are the effective authority for that case and never exceed the manifest request. Even successful resource resolution is only a return-value check: do not fetch the placeholder media URL or perform DNS, redirects, or playback. Denied network access does not prevent static `home` data without URLs from being returned.

## Fixture shape and comparison rules

Every case has `id`, `source`, `operation`, `input`, `grants`, and `expect`. The fixture-format schema is [`cases.schema.json`](cases.schema.json). It is a non-normative validation aid for this corpus, not a new source format, an official Source API schema, or a language binding. In particular, it allows invalid operation inputs so argument-error cases can be expressed.

`expect` contains exactly one of:

- `result`: the complete successful envelope `{"ok":true,"data":...}`. The execution harness compares JSON values structurally, ignoring object member order but preserving array order, values, and the distinction between omitted members and `null`. It must not trim strings, normalize IDs, coerce types, or fill additional defaults. JSON integer representations such as `1` and `1.0` denote the same numeric value; strings and booleans are different types.
- `errorCode`: the exact required Source API error code. This is an assertion descriptor, **not** a returned error envelope. The actual outcome must be `{"ok":false,"error":{"code":CODE,"message":TEXT}}`, with no `data` or unknown members. `TEXT` must be a nonempty diagnostic string meeting the Source API's confidentiality rules. Its wording is intentionally not compared; the contract does not standardize it.

No runtime functions, module exports, callbacks, or transport protocol are prescribed. The Rust harness adapts these logical calls to its implementation-specific boundary and separately checks actual envelopes. Python fixture validation remains a distinct structural check.

## Cases and normative basis

| Case IDs | Authored outcome | Basis |
| --- | --- | --- |
| `home-normal` | Example categories and content. | [Home](../../spec/source-api.md#home), stored data. |
| `home-empty` | Both arrays empty in the home-only context. | Home arrays may be empty. |
| `home-with-denied-network` | Same example home data with no network grant. | [Denied permissions](../../spec/host-api.md#capabilities-and-permission-grants); no returned URLs or network use. |
| `home-unknown-input-field` | `INVALID_ARGUMENT`. | Closed input objects and [input validation](../../spec/lifecycle.md#3-invoke-an-operation). |
| `category-page-one`, `category-default-page` | Example content, page 1, `hasMore: false`. | [Operation defaults](../../spec/source-api.md#operations). |
| `category-past-end`, `category-largest-page` | Empty page 2 / 9007199254740991, `hasMore: false`. | [Static dispatch](../../spec/source-api.md#minimal-declarative-entry-format), inclusive page bound. |
| `category-unknown`, `category-unknown-past-end` | `NOT_FOUND`. | Category lookup precedes past-end handling. |
| `category-page-zero`, `category-page-fraction`, `category-page-null`, `category-page-string`, `category-page-too-large` | `INVALID_ARGUMENT`. | Page integer range and missing/null distinction. |
| `search-exact`, `search-default-page` | Exact `demo` match, page 1, `hasMore: false`. | Static exact lookup and default page. |
| `search-no-match`, `search-case-sensitive`, `search-no-trimming` | Empty page 1, `hasMore: false`. | No case folding or trimming; missing query key is not `NOT_FOUND`. |
| `search-past-end` | Empty page 2, `hasMore: false`. | Static profile has only page 1 data. |
| `search-empty-query`, `search-whitespace-query`, `search-missing-query`, `search-invalid-page` | `INVALID_ARGUMENT`. | Query requirements; whitespace case uses exactly U+0020, U+0009, U+000D, U+000A. |
| `detail-known` | Entire stored detail, including description and playable. | [Detail](../../spec/source-api.md#detail) and static lookup. |
| `detail-unknown` | `NOT_FOUND`. | Missing content ID. |
| `detail-empty-id` | `INVALID_ARGUMENT`. | Nonempty input ID. |
| `play-known-permitted` | Entire stored resource for `demo-main`. | [Play](../../spec/source-api.md#play), requested origin explicitly granted. |
| `play-unknown`, `play-unknown-denied` | `NOT_FOUND`. | Missing playable lookup returns no resource whose URL could require a grant. |
| `play-known-denied` | `PERMISSION_DENIED`, not a resource or empty success. | [Returned URL enforcement](../../spec/host-api.md#network-and-returned-resources); manifest request alone is insufficient. |
| `search-undeclared`, `search-undeclared-invalid-input` | `UNSUPPORTED_OPERATION` on home-only source. | Lifecycle checks declarations before input validation. |

The known/permitted play case intentionally covers both known playable lookup and the allowed-origin result. The paired denied case changes only grants. The home-only source tests empty results and undeclared operations without mutating the example or inventing per-case source overrides.

## Structural validation and runtime execution

Use the existing [development setup](../README.md#setup), then run from the repository root:

```sh
python -m unittest discover -s conformance -p 'test_*.py' -v
```

The command includes all existing manifest tests and [`test_declarative_fixtures.py`](../test_declarative_fixtures.py). The latter checks the fixture schema, unique case IDs, named source paths, contained existing entries, manifests, stored shapes/references, grant subsets, and success snapshots against stored data. It checks declared/undeclared-operation consistency and success page labels without computing operation outcomes.

The fixture schema's data definitions cover the shapes used here; they are not complete validators for all header, URL, origin, or arbitrary source semantics. The code's URL checks inspect the simple canonical origins used by these contexts and do not implement general origin normalization. Expected error codes and lookup/pagination behavior were reviewed against the prose; these are authored expectations, not results generated by an interpreter. The validator intentionally cannot prove that every authored input/outcome pair is behaviorally correct.

**All 34 cases now execute and pass through the Rust runtime.** Run from the repository root with the [runtime toolchain](../../runtime/README.md#build-and-validate):

```sh
cargo test --manifest-path runtime/Cargo.toml --locked --test declarative_conformance -- --nocapture
```

The [harness](../../runtime/tests/declarative_conformance.rs) freshly loads each context, establishes effective grants, invokes the requested operation, validates envelope exclusivity/closed shapes, and compares the authored success or error-code expectation. Each case ID is printed; all failures, including setup failures, fail the test. Message text is not fixed. Numeric comparison preserves equal JSON values such as `1`/`1.0` without type coercion. Expected outcomes are unchanged and not generated by the dispatcher. This is bounded ready-source execution evidence, not completion of Phase 2. Runtime limits, cancellation, redirects, loading failures, and non-ready calls are outside this ready-instance corpus. The [scoped contract review](../../docs/reviews/2026-09-28-declarative-contract-review.md) records the original findings and deferred bindings; the [closure review](../../docs/reviews/2026-09-28-phase-1-closure-review.md) records their current dispositions and Phase 2 implementation boundaries, including the limits of fixture helpers as semantic validators.
