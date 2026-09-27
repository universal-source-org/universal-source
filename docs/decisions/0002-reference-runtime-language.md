# ADR 0002: Rust for the reference runtime

- Status: Accepted
- Date: 2026-09-28
- Scope: the reference implementation in this monorepo, starting with static declarative loading
- Related: [ADR 0001](0001-initial-architecture.md), [closure baseline](../reviews/2026-09-28-phase-1-closure-review.md), [runtime](../../runtime/README.md)

## Context

Phase 2 needs a small portable library that validates untrusted JSON, models distinguishable loading failures, resolves contained files, and retains independently owned static data. The specification and deterministic fixtures already exist; no existing runtime or host-language binding constrains this choice. The first unit needs no script engine, async scheduler, network client, or UI framework.

Future embedding on multiple host platforms matters, but no platform integration or performance benchmark has been demonstrated. ADR 0001 named Rust as a candidate, not a prior selection. Python in the conformance tooling is also not a runtime commitment.

## Decision

Select **Rust** for the reference runtime. Start with one library crate under `runtime/`, with no platform binding or public native ABI. The implementation language is **not part of the Universal Source specification or source format**. Sources still declare `declarative` or `javascript`; other conforming implementations may use other languages.

Rust's owned data and explicit error types fit immutable loaded snapshots and separate host-facing diagnostics. Its filesystem paths support component-based containment checks, and its ecosystem provides JSON decoding, Draft 2020-12 validation, and URL parsing. These satisfy current needs without embedding a managed language runtime. Potential future native embedding supports the choice, but does not establish cross-platform support or justify adding FFI now.

Use the authoritative repository manifest schema directly through the `jsonschema` crate, with HTTP/file reference retrieval features disabled. Use `serde`/`serde_json` with explicit duplicate-member rejection, `url` for parsing, and `mime` for media-type syntax; `tempfile` is test-only. Commit `runtime/Cargo.lock` and use locked builds. Parser details and finite load budgets are implementation policies documented in the runtime guide, not amendments to the standard.

## Alternatives considered

| Alternative | Advantages for this task | Reason not selected |
| --- | --- | --- |
| Python | Existing test tooling and validator; fast development and readable validation logic. | An embedded interpreter and dynamic host boundary would complicate the intended portable reference core. Python remains appropriate for independent conformance checks. |
| TypeScript/JavaScript | JSON-oriented ecosystem and potential reuse with the later JavaScript engine. | Requires a JS execution environment for the core and risks conflating the implementation with the still-unspecified source binding. Phase 2 needs neither. |
| Go | Explicit errors, standard filesystem support, simple compiled tooling. | Its managed runtime adds embedding considerations; Rust ownership better matches the desired core boundary. No benchmark claim is made. |
| C/C++ | Native embedding and broad platform tooling. | Manual lifetime/error handling would add risk and review cost to parsing untrusted input without a demonstrated benefit for this unit. |

## Consequences

Contributors need a Rust toolchain for runtime work in addition to the existing Python conformance environment. Third-party crates and their transitive dependencies require ongoing review; the lockfile identifies the tested resolution. Build cost is higher than a handwritten structural checker, but a handwritten schema substitute would be inappropriate.

Rust does not itself prove filesystem race safety, bounded execution, permission enforcement, or portability. The initial loader assumes a host-controlled directory stable during loading and documents its canonicalize/open limitation. It bounds input size and depth, validates data, and exposes no dispatch. Later invocation, grants, cancellation, embedding, and platform proofs remain separate work.

This decision completes ADR 0001's deferred implementation-language choice. Its monorepo, platform-neutral standard, five operations, initial engine targets, and adapter/WASM boundaries remain in force. No normative specification change or language requirement is introduced.
