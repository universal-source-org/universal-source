# JavaScript engine feasibility evaluation

Status: complete; Outcome B, no engine selected. Baseline: `d6da15425d0aa4354641d643e28cbe35ca9e17db`; clean worktree, no other active plan.

Evaluate QuickJS/QuickJS-NG, Boa, V8 and JavaScriptCore against proposed RFC 0001 using upstream source/docs and one isolated Rust spike. Distinguish native engine support, Rust API exposure, local evidence and platform assumptions. Select only with defensible evidence; otherwise record blockers without an ADR. RFC acceptance is separate.

1. Review governing documents, dependency policy, RFC and Phase 2 evidence.
2. Desk-evaluate pinned upstream candidates, emphasizing jobs/reaction ownership, CPU interruption, structural inspection/Proxy, dynamic compilation and isolation.
3. Exercise only the strongest candidate's critical primitives in a separate unpublished spike crate. No production dependencies, loader/dispatcher/services, conformance corpus or engine abstraction.
4. Record evaluation, exact versions/results, RFC feedback and selection disposition. Update minimal current-state links.
5. Run all six baseline checks plus spike tests/fmt/clippy, document/link/JSON/whitespace checks; review diff, complete plan and commit locally. No push/tag/release.

## Results and scope review

Reviewed all requested governing/specification documents, both existing ADRs, Phase 2 closure, prior binding plan, runtime/conformance guides, Git history/status, repository structure and dependency workflow before editing. No separate dependency policy beyond the contribution/runtime/decision guidance was found. The starting worktree was clean and there were no other active plans.

The [evaluation](../../reviews/2026-09-28-javascript-engine-evaluation.md) compares Bellard QuickJS 2026-06-04, QuickJS-NG 0.16.2 with rquickjs 0.14.0, Boa 0.22.0, V8 15.2.124.1 with rusty_v8 152.2.0, and JavaScriptCore public/private APIs with rust_jsc 0.5.0 and its distinct WebKit fork. It records exact revisions, authoritative sources, Rust API exposure, build/platform/license constraints and local evidence limits. Only QuickJS-NG was built and exercised, on macOS 27.0 arm64 with Rust/Cargo 1.96 and Apple clang 21.0.0.

The [separate spike](../../../experiments/javascript-engine-spike/README.md) has ten small primitive tests. It proves local isolation/reuse/disposal, synchronous function calls, intrinsic Promise inspection and explicit one-job execution, loop/regexp interruption, accessor-free descriptor inspection, non-executing Proxy detection, twelve constructor-lockdown probes, and absence of thirteen ambient host-global names. It also reproduces delayed Promise reactions crossing synthetic invocation boundaries and an await path bypassing a source `.then` wrapper. All ten passing tests include these expected incompatibilities; they are not conformance claims.

No engine selection or ADR is justified. RFC 0001 is unchanged and remains proposed. The main blocker is supported ownership/revocation of queued and not-yet-queued reactions while retaining healthy instance state (evaluation F1). Exact deterministic-global hardening, comprehensive non-executing copying, module preflight and other-platform evidence remain incomplete (F2/F3). The next unit is a focused Boa reaction-hook experiment and RFC feasibility review, before production integration. Boa hooks are not assumed to allow silently suppressing callbacks contrary to their documented contract.

## Validation

All commands below exited 0 on 2026-09-28:

| Command | Result |
| --- | --- |
| `cargo test --manifest-path runtime/Cargo.toml --locked` | 53 Rust tests passed: 18 library, 2 conformance harness, 8 dispatch, 25 loader; 0 failed/ignored; 0 doc tests. Harness includes the 34 authored operation cases. |
| `cargo test --manifest-path runtime/Cargo.toml --locked --test declarative_conformance -- --nocapture` | 2 tests passed; printed every case ID, **34/34 authored declarative cases passed**. |
| `cargo build --manifest-path runtime/Cargo.toml --locked` | Passed. |
| `cargo fmt --manifest-path runtime/Cargo.toml --check` | Passed, no output. |
| `cargo clippy --manifest-path runtime/Cargo.toml --locked --all-targets -- -D warnings` | Passed, no warnings. |
| `.venv/bin/python -m unittest discover -s conformance -p 'test_*.py' -v` | 6 tests passed. |
| `cargo test --manifest-path experiments/javascript-engine-spike/Cargo.toml --locked --no-run` | Passed native engine/spike compilation. |
| `cargo test --manifest-path experiments/javascript-engine-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 10 passed, 0 failed/ignored, 0 doc tests; final test execution 0.01 s. Run under the README's Python process-group 60-second backstop; backstop did not fire. |
| `cargo fmt --manifest-path experiments/javascript-engine-spike/Cargo.toml --check` | Passed, no output. |
| `cargo clippy --manifest-path experiments/javascript-engine-spike/Cargo.toml --locked --all-targets -- -D warnings` | Passed, no warnings. |

Documentation validation uses a temporary external script, `.venv/bin/python /tmp/universal-source-engine-doc-audit.py`, since the repository has no supplied Markdown/link checker. It strictly parses repository JSON and fenced JSON, validates both Draft 2020-12 schemas and internal schema references, checks the RFC manifest example, Markdown headings/fences/table widths, local links/anchors, external URL syntax and text whitespace/final newlines. External link availability is not comprehensively audited; authoritative evidence pages/source were separately retrieved during evaluation. Passed: 48 JSON files, 2 schemas, 3 fenced JSON blocks, 36 Markdown files, 321 local links, 36 anchors, 24 external URL spellings and 105 nonempty text files. `git diff --check` and `git diff --cached --check` passed. The complete staged diff (including all new files) was reviewed for scope, consistency, dependencies, accidental generated artifacts and sensitive data.

## Delivery and remaining work

Complete changed-file set (nine files):

- `docs/ARCHITECTURE.md`: current evaluation/spike state and next boundary.
- `docs/ROADMAP.md`: Phase 3 active, no selection, targeted F1 follow-up.
- `docs/reviews/2026-09-28-javascript-engine-evaluation.md`: comparison, pinned evidence, local results and RFC feedback.
- `docs/plans/completed/2026-09-28-javascript-engine-evaluation.md`: this completed plan and validation record.
- `experiments/javascript-engine-spike/.gitignore`: local target output.
- `experiments/javascript-engine-spike/Cargo.toml`: unpublished isolated test crate, rquickjs exact dependency with only std.
- `experiments/javascript-engine-spike/Cargo.lock`: independent locked experiment graph; 18 external locked packages, 10 in the active build graph.
- `experiments/javascript-engine-spike/README.md`: scope, dependencies, bounded rerun instructions and limitations.
- `experiments/javascript-engine-spike/src/lib.rs`: ten fixed-source primitive probes and a local descriptor helper.

No tracked file under `runtime/`, `conformance/`, `spec/`, `examples/` or `docs/decisions/` changes. No production dependency, Source API behavior, host service, platform integration or conformance expectation changes. No push, tag, release or repository settings change is authorized or performed. The final commit is discoverable in Git as `test(engine): evaluate JavaScript binding feasibility`; no self-referential commit hash is stored in this plan.
