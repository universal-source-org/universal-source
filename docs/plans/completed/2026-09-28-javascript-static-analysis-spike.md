# JavaScript source-static-analysis compatibility proof

Status: completed with Outcome B — static preflight blocker. Baseline `9307dc4be4e3c9bfa27ae9127a011b6abe499279`; clean starting tree, no other active plans. Phase 3 remains active.

Scope: one isolated unpublished Boa Parser 0.22.0 experiment against unchanged rquickjs/core/sys 0.14.0 / QuickJS-NG 0.16.2. No production dependencies, source rewriting, parser/engine selection, ADR, RFC acceptance or lifecycle change.

1. Read governing contracts and previous evidence; inspect pinned parser/AST/source APIs and current upstream provenance.
2. Build a small deterministic module/export corpus and AST visitor, with exact-byte identity, acceptance matrix, decoded names, nested syntax, import/TLA, location and structural-bound probes. Distinguish runtime code-generation denial from syntax rejection.
3. Record concrete Outcome A or B, all disagreements, limitations and the smallest next evidence gap. Do not hide valid-source rejection or lost syntax metadata.
4. Run baseline Rust/Python/conformance checks, all four previous experiments, guarded new tests, touched-crate fmt/strict Clippy and established docs/link/schema/whitespace checks. Review complete diff, complete plan and commit locally; no publication.

## Results and remaining gap

[Review](../../reviews/2026-09-28-javascript-static-analysis-spike.md) and [experiment](../../../experiments/javascript-static-analysis-spike/README.md): 15 passing tests, 62-case acceptance matrix (58 both accept, 1 Boa-only outside-profile import-source syntax, 0 QuickJS-only, 3 both reject). Original bytes are preserved. Direct/forbidden operation forms, escaped/Unicode names, nested syntax and TLA are distinguishable in the tested AST surface. Empty import/re-export `with {}` clauses are not: the public module-item AST equals the no-clause form. Passing negative controls reproduce false acceptance, not RFC compliance.

Outcome B requires extra syntax-aware source validation or a lossless parser representation; it does not require changing RFC semantics or the previous capture strategy. Next is a bounded clause-presence proof with lexical negative controls, not production implementation. No parser/engine/ADR decision, RFC amendment, production dependency or conformance change was made.

## Validation

All final commands below exit 0. Each experiment test command was run by a Python `subprocess.Popen(..., start_new_session=True)` wrapper with a 60-second timeout and process-group kill on expiry; no timeout fired. The reproducible wrapper is in the experiment README. Results are from macOS 27.0 (26A428) arm64, Rust/Cargo 1.96.0. Experiment tests are local evidence, not portable conformance.

| Exact command | Result |
| --- | --- |
| `cargo test --manifest-path runtime/Cargo.toml --locked` | 53 pass: 18 library, 2 declarative harness, 8 dispatch, 25 loader; 0 doc tests. |
| `cargo test --manifest-path runtime/Cargo.toml --locked --test declarative_conformance -- --nocapture` | 2 pass; all 34 authored declarative cases PASS. |
| `cargo build --manifest-path runtime/Cargo.toml --locked` | Pass. |
| `cargo fmt --manifest-path runtime/Cargo.toml --check` | Pass. |
| `cargo clippy --manifest-path runtime/Cargo.toml --locked --all-targets -- -D warnings` | Pass, no warnings. |
| `.venv/bin/python -m unittest discover -s conformance -p 'test_*.py' -v` | 6 pass. |
| `cargo test --manifest-path experiments/javascript-static-analysis-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 15 pass, 0 failed/ignored, 0 doc tests; asserted matrix totals 3/0/1/58 in reject/reject, reject/accept, accept/reject, accept/accept order. |
| `cargo fmt --manifest-path experiments/javascript-static-analysis-spike/Cargo.toml --check` | Pass. |
| `cargo clippy --manifest-path experiments/javascript-static-analysis-spike/Cargo.toml --locked --all-targets -- -D warnings` | Pass, no warnings. |
| `cargo test --manifest-path experiments/quickjs-module-capture-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 14 pass, 0 doc tests. |
| `cargo test --manifest-path experiments/quickjs-lifecycle-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 15 behavior tests + 3 compile-fail doc tests pass; compiler errors in those negative controls are expected. |
| `cargo test --manifest-path experiments/javascript-engine-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 10 pass, 0 doc tests. |
| `cargo test --manifest-path experiments/boa-reaction-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 14 pass, including the existing expected-panic control; 0 doc tests. |

The new crate was initially built/test-compiled offline with the prior experiment versions; strict final checks use the committed lockfile. During development, the matrix revealed that both parsers accept newer `using` syntax; its explicit AST variant is now an outside-ES2023 rejection. A duplicate visitor count for `new Function` was fixed because the AST already delegates its visit to `Call`. No previous test expectation was weakened.

Documentation and preservation checks:

- `.venv/bin/python /tmp/universal-source-engine-doc-audit.py`: pass using the established temporary audit from prior units; 50 Markdown files, 390 local links, 39 anchors, 67 external URL spellings, 48 strict JSON files, 2 Draft 2020-12 schemas, 3 fenced JSON examples and whitespace/final-newline checks across 141 text files. External URL syntax is not a reachability claim; the eight pinned Boa source fetch/comparisons separately passed.
- `git diff --check` and `git diff --cached --check`: pass.
- `git diff --exit-code 9307dc4be4e3c9bfa27ae9127a011b6abe499279 -- runtime spec conformance examples docs/decisions experiments/quickjs-module-capture-spike experiments/quickjs-lifecycle-spike experiments/javascript-engine-spike experiments/boa-reaction-spike`: empty diff, pass.
- Lockfile package/version comparison against all prior experiment locks: 71 external pairs, no new version pair, no boa_engine. Reviewed the complete scoped diff including new files before local commit.

## Files and dependency scope

Exactly these 12 files comprise this unit:

- `docs/ARCHITECTURE.md`
- `docs/ROADMAP.md`
- `docs/reviews/2026-09-28-javascript-static-analysis-spike.md`
- `docs/plans/completed/2026-09-28-javascript-static-analysis-spike.md` (moved from active)
- `experiments/javascript-static-analysis-spike/.gitignore`
- `experiments/javascript-static-analysis-spike/Cargo.toml`
- `experiments/javascript-static-analysis-spike/Cargo.lock`
- `experiments/javascript-static-analysis-spike/README.md`
- `experiments/javascript-static-analysis-spike/src/lib.rs`
- `experiments/javascript-static-analysis-spike/src/analysis.rs`
- `experiments/javascript-static-analysis-spike/src/corpus.rs`
- `experiments/javascript-static-analysis-spike/src/tests.rs`

Direct experiment dependencies: boa_parser/boa_ast/boa_interner `=0.22.0`, rquickjs `=0.14.0`; 71 external locked packages, all exact package/version pairs already present in prior experiment locks. No boa_engine dependency or production dependency changes. Baseline runtime/spec/conformance/ADRs/old experiments remain unchanged. Local Conventional Commit only; no push, tag or release.
