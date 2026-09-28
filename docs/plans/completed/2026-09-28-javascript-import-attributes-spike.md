# Import-attribute clause presence proof

Status: completed with Outcome A — attribute-clause preflight viable. Baseline `2fea0b2bf678b775964ddc0c861a8a9a5aadc824`; clean main, no other active plans. Phase 3 remains active.

Scope: resolve only Boa AST's absent-versus-empty attribute clause gap. Preserve RFC, prior tests, production code and pinned Boa/QuickJS versions. No parser/engine selection or ADR.

1. Inspect pinned Boa public lexer/source retention boundaries; examine one alternate public syntax representation (Oxc) only if needed.
2. Extend the isolated static-analysis experiment with clause ownership/span tests, lexical negative controls, exact-byte comparison, fail-closed parser checks and pinned QuickJS compilation.
3. Record concrete Outcome A/B, limitations and next evidence gap in a focused review; minimally update current architecture/roadmap.
4. Run baseline checks and all five experiments with external timeouts, touched-crate fmt/Clippy and documentation/schema/link/whitespace checks. Complete plan, review full diff, commit locally; no publication.

## Results

The [review](../../reviews/2026-09-28-javascript-import-attributes-spike.md) records the public Boa lexer limitation and a narrow hybrid proof: unchanged Boa structural checks plus Oxc 0.152.0 optional clause nodes. Empty attributes remain invisible to Boa alone but are now rejected for imports and re-exports. All original tests remain unchanged. No private API, source rewriting, heuristic detector, parser/engine selection, ADR, RFC change or production implementation.

Nine new tests cover 24 grammar matrix rows, 40 trivia variations, 15 lexical/ASI controls, exact bytes and spans, prior false-acceptance closure, parser errors/disagreement and all 62 old corpus entries. The combined suite has 24 passing tests. Oxc accepts legacy assertions that Boa/QuickJS reject; the gate rejects them. No disagreement on tested RFC-valid sources was observed. Next is only pinned-engine dynamic-code suppression feasibility under RFC §9; full parser compatibility and resource bounds remain separate.

## Validation

All final commands below exit 0. Each experiment test command ran through the existing temporary Python runner with `subprocess.Popen(..., start_new_session=True)` and a 60-second process-group timeout; none fired. The experiment README includes the reusable single-suite wrapper. Local machine: macOS 27.0 (26A428) arm64, Rust/Cargo 1.96.0. The extended crate was test-compiled first with `cargo test --manifest-path experiments/javascript-static-analysis-spike/Cargo.toml --locked --no-run` (pass).

| Exact command | Result |
| --- | --- |
| `cargo test --manifest-path runtime/Cargo.toml --locked` | 53 pass: 18 library, 2 declarative harness, 8 dispatch, 25 loader; 0 doc tests. |
| `cargo test --manifest-path runtime/Cargo.toml --locked --test declarative_conformance -- --nocapture` | 2 pass; all 34 authored declarative cases PASS. |
| `cargo build --manifest-path runtime/Cargo.toml --locked` | Pass. |
| `cargo fmt --manifest-path runtime/Cargo.toml --check` | Pass. |
| `cargo clippy --manifest-path runtime/Cargo.toml --locked --all-targets -- -D warnings` | Pass, no warnings. |
| `.venv/bin/python -m unittest discover -s conformance -p 'test_*.py' -v` | 6 pass. |
| `cargo test --manifest-path experiments/javascript-static-analysis-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 24 pass (15 preserved + 9 new), 0 failed/ignored, 0 doc tests; original matrix unchanged, both old false acceptances rejected by supplemental gate. |
| `cargo fmt --manifest-path experiments/javascript-static-analysis-spike/Cargo.toml --check` | Pass. |
| `cargo clippy --manifest-path experiments/javascript-static-analysis-spike/Cargo.toml --locked --all-targets -- -D warnings` | Pass, no warnings. |
| `cargo test --manifest-path experiments/quickjs-module-capture-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 14 pass, 0 doc tests. |
| `cargo test --manifest-path experiments/quickjs-lifecycle-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 15 behavior tests + 3 compile-fail doc tests pass; compiler errors in those negative controls are expected. |
| `cargo test --manifest-path experiments/javascript-engine-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 10 pass, 0 doc tests. |
| `cargo test --manifest-path experiments/boa-reaction-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 14 pass, including the existing expected-panic control; 0 doc tests. |

Development probes corrected two initially assumed expectations: Oxc's clause span covers only braces/contents, and duplicate exports already produce parser diagnostics. The actual legacy-assertion grammar disagreement supplies the real fail-closed witness. No earlier test was weakened or removed. Repeated decisions in the 24-row matrix are equal.

Additional final checks:

- `.venv/bin/python /tmp/universal-source-engine-doc-audit.py`: pass; 52 Markdown files, 404 local links, 40 anchors, 74 external URL spellings, 48 strict JSON files, 2 schemas, 3 fenced JSON examples and whitespace/final-newline checks across 145 text files. External URL spelling is not a reachability claim; seven pinned upstream source fetch/byte comparisons separately passed.
- `git diff --check` and `git diff --cached --check`: pass.
- `git diff --exit-code 2fea0b2bf678b775964ddc0c861a8a9a5aadc824 -- runtime spec conformance examples docs/decisions experiments/quickjs-module-capture-spike experiments/quickjs-lifecycle-spike experiments/javascript-engine-spike experiments/boa-reaction-spike experiments/javascript-static-analysis-spike/src/analysis.rs experiments/javascript-static-analysis-spike/src/corpus.rs experiments/javascript-static-analysis-spike/src/tests.rs`: empty diff, pass.
- Lockfile package/version comparison: 71 original pairs retained, 32 added, none removed/upgraded. Full scoped diff reviewed before local commit.

## Files and dependencies

Ten files changed:

- `docs/ARCHITECTURE.md`
- `docs/ROADMAP.md`
- `docs/reviews/2026-09-28-javascript-import-attributes-spike.md`
- `docs/plans/completed/2026-09-28-javascript-import-attributes-spike.md` (moved from active)
- `experiments/javascript-static-analysis-spike/Cargo.toml`
- `experiments/javascript-static-analysis-spike/Cargo.lock`
- `experiments/javascript-static-analysis-spike/README.md`
- `experiments/javascript-static-analysis-spike/src/lib.rs`
- `experiments/javascript-static-analysis-spike/src/attributes.rs`
- `experiments/javascript-static-analysis-spike/src/attribute_tests.rs`

The four new direct Oxc dependencies (parser/AST/allocator/span) are pinned to `=0.152.0`, revision `dfbc0d1ea752f021ba4a68e4703b3a9031082a46`. The experiment lockfile adds 32 packages (103 external total), with no previous package/version removed or upgraded; shared feature edges expand locally. Existing Boa 0.22.0, rquickjs/core/sys 0.14.0 and QuickJS-NG 0.16.2 remain pinned. No production dependencies changed. No parser/engine selection, ADR, RFC status change, push, tag or release.
