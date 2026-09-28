# JavaScript lifecycle and F1 decision

Status: complete; lifecycle Model B chosen in the proposed RFC, no engine selected. Baseline: `f4029bf64f74ea8ee94cf0fba2c8cd0d5985a016`; starting worktree clean, no other active plans.

Scope: documentation-only comparison of lifecycle Models A–E using the completed QuickJS-NG and Boa evidence. Choose one portable lifecycle, narrowly revise proposed RFC 0001, and distinguish logical instance reuse from mutable JavaScript persistence. Preserve the experiments, runtime, dependencies, ADRs and conformance corpus. No engine selection, RFC acceptance, new experiment or production execution.

1. Read governing contracts and completed evidence; identify which state/lifecycle guarantees are normative and which exist only in the proposed binding.
2. Compare A–E, define terminal timing, Promise graphs, capability revocation and instance health; record the decision and engine evidence limits in one review.
3. Amend only affected RFC sections and architecture/roadmap status. Check all persistence, initialization and stale-reference wording together.
4. Rerun six baseline commands and both existing experiment suites; run documentation/JSON/schema/link/anchor/whitespace checks, review the complete diff, complete this plan and commit locally. Do not publish.

## Result and review

The [decision review](../../reviews/2026-09-28-javascript-lifecycle-decision.md) compares A–E and selects mandatory fresh realms for each executing invocation. Mutable JavaScript persistence was a proposed RFC convenience, not an existing normative Source API guarantee. Logical instances retain validated immutable snapshots and remain reusable after ordinary outcomes; every realm is retired, with no Promise graph or wrapper entering a later call. Execution cancellation/deadline/resource termination still disposes the logical instance. Merely queued/pre-execution failures leave healthy instances intact.

RFC 0001 now specifies a disposable synchronous load-validation realm and fresh per-call evaluation, errors during ready-call initialization, terminal delivery gating, non-executing copying and teardown before final interruption check/publication, and exact abandonment of queued/future reactions, children and await graphs without synthetic Promise settlement. Private same-invocation error classification survives authority revocation for candidate processing. Future conformance assertions now require state reset and late native delivery suppression. Return Model B, module restrictions, transfer/authority/code-generation rules, error vocabulary and A1/U1/U2/P1/D1/D2 remain intact. The experimental v0.1/no-bump conclusion remains, with the incompatible change to the earlier unaccepted persistence proposal explicitly recorded.

F1 is resolved as a proposed lifecycle decision, not an engine implementation claim. The next unit reviews/proves this lifetime through the existing QuickJS-NG/rquickjs embedding path: roots/queues/resolvers, repeated initialization and capture-before-evaluation, no stale reentry and mandatory reset. Other engine feasibility and platform evidence remain open. No production execution is authorized by this document, and the RFC is not accepted.

## Validation

Rerun environment: macOS 27.0 build 26A428 arm64; rustc 1.96.0 (ac68faa20 2026-05-25), Cargo 1.96.0 (30a34c682 2026-05-25), target `aarch64-apple-darwin`. Existing pinned dependencies unchanged: rquickjs 0.14.0 / QuickJS-NG 0.16.2 (`0fdea21ff1090084e91dad812b343d92e79ba9d9`); Boa 0.22.0 (`337a3668a0dc86dd401ea20906e782249a64a228`). No new engine behavior or platform support claimed.

All commands below exited 0 on 2026-09-28:

| Command | Exact result |
| --- | --- |
| `cargo test --manifest-path runtime/Cargo.toml --locked` | 53 passed: 18 library, 2 conformance harness, 8 dispatch, 25 loader; 0 failed/ignored; 0 doc tests. |
| `cargo test --manifest-path runtime/Cargo.toml --locked --test declarative_conformance -- --nocapture` | 2 tests passed; all 34/34 authored cases printed PASS. |
| `cargo build --manifest-path runtime/Cargo.toml --locked` | Passed. |
| `cargo fmt --manifest-path runtime/Cargo.toml --check` | Passed, no output. |
| `cargo clippy --manifest-path runtime/Cargo.toml --locked --all-targets -- -D warnings` | Passed, no warnings. |
| `.venv/bin/python -m unittest discover -s conformance -p 'test_*.py' -v` | 6 passed, 0 failures (0.038 s). |
| `cargo test --manifest-path experiments/javascript-engine-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 10 passed, 0 failed/ignored; 0 doc tests (0.01 s). |
| `cargo test --manifest-path experiments/boa-reaction-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 14 passed, 0 failed/ignored; 0 doc tests (0.04 s). The named should-panic await probe printed its expected assertion and passed. |

Both experiment commands ran using the existing README-style Python POSIX process-group 60-second backstop; neither backstop fired. No experiment/test/source/dependency modifications were necessary. These tests preserve counterexamples to the prior persistent-realm proposal; they are not conformance tests of the revised fresh-realm binding.

Documentation validation uses the established temporary external checker: `.venv/bin/python /tmp/universal-source-engine-doc-audit.py`. It checks strict/duplicate-free JSON, schemas and fenced examples, Markdown heading/fence/table structure, local links/anchors, external URL syntax, whitespace and final newlines. Passed: 48 JSON files, 2 schemas, 3 fenced JSON examples, 41 Markdown files, 350 local links, 39 anchors, 33 external URL spellings and 114 nonempty text files. `git diff --check` and `git diff --cached --check` passed; protected baseline paths were verified unchanged. External-link availability is not audited; no new upstream capability assertion or new engine survey requires it in this unit.

## Delivery scope

Exactly five documentation files:

- `docs/reviews/2026-09-28-javascript-lifecycle-decision.md`: comparison, selected lifecycle, Promise/capability/terminal rules, compatibility and next embedding proof.
- `docs/plans/completed/2026-09-28-javascript-lifecycle-decision.md`: this completed plan, results and validation.
- `spec/rfcs/0001-javascript-execution-binding.md`: narrow lifecycle revision and affected cross-references/future conformance/internal review; proposed status preserved.
- `docs/ARCHITECTURE.md`: current proposed lifecycle and implementation boundary.
- `docs/ROADMAP.md`: Phase 3 status and focused embedding proof next.

Runtime, tests, dependencies/lockfiles, existing experiments/evidence, examples, normative specification documents outside the RFC, conformance corpus/expected results and ADRs are unchanged. Only future proposed JavaScript conformance requirements change. Full diff review includes both new documents, lifecycle/error consistency, status wording and protected paths. Local Conventional Commit subject: `docs(spec): choose per-invocation JavaScript realms`. No push, tag, release or repository-setting change; no self-referential commit hash is stored here.
