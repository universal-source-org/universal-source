# Restricted-global and ambient-authority feasibility

Status: completed with Outcome A — restricted-global surface viable. Baseline `6476e337bdf4677324b5a2f065a9f2ddd936a737`; clean tree, no other active plans. Phase 3 remains active.

Scope: one isolated pinned QuickJS-NG/rquickjs public-API proof against RFC 0001 §9. Inventory globals/intrinsics, enforce the exact global surface, suppress clocks/randomness/locale/stacks/extensions, test transitive reachability, mutation, host boundaries and fresh realms, and reuse the complete dynamic-code route corpus. No production code/dependencies, source rewriting, RFC amendment, engine/parser selection, ADR or publication.

1. Read governing context, prior experiments and pinned public/source APIs.
2. Record pristine inventory; implement fail-closed hardening and positive/negative probes. Resolve to exactly Outcome A or a precise Outcome B.
3. Run baseline, all Phase 3 experiments under external timeouts, touched-crate fmt/strict Clippy, and established documentation/link/schema/whitespace checks.
4. Record evidence/limits and next gap, complete this plan, review complete diff and commit locally.

## Results and limits

The [review](../../reviews/2026-09-28-quickjs-global-surface-spike.md) and [experiment](../../../experiments/quickjs-global-surface-spike/README.md) record 19 passing tests. The pristine snapshot has 70 global string names plus one symbol and 128 additional intrinsic/root objects. The tested RFC allowlist is exactly 38 global names, with 72 explicit intrinsic key sets. Date/performance/extensions are removed; eight random/locale methods throw native TypeError. All ordinary Error stack hooks and function location getters are absent. A bound Symbol facade avoids nonconfigurable disposal constants without exposing native Symbol. The unchanged dynamic bootstrap/corpus still denies all 80 routes. Test-private host-binding/capture and fresh-realm probes pass; bounded traversal completes with 368 nodes (372 including binding), depth 4, below depth 16/node 4096 limits.

No complete sandbox, production JS, resource policy, value converter, parser/engine selection or service profile is implemented. No RFC or ADR changed. Next: complete non-executing value-conversion feasibility against §6, as now recorded in the roadmap. Remaining resource, module/path, service, platform and conformance work is separate.

## Validation

Local macOS 27.0 arm64, Rust/Cargo 1.96.0. Every engine/parser test command used an external 60-second process timeout; the final full validation runner uses process-group termination on timeout, as reproduced in the experiment README. None fired. All final commands pass. New-crate test compilation preceded execution.

| Exact command | Result |
| --- | --- |
| `cargo test --manifest-path runtime/Cargo.toml --locked` | 53 pass: 18 library, 2 declarative harness, 8 dispatch, 25 loader; 0 doc tests. |
| `cargo test --manifest-path runtime/Cargo.toml --locked --test declarative_conformance -- --nocapture` | 2 pass; all 34 authored operation cases PASS. |
| `cargo build --manifest-path runtime/Cargo.toml --locked` | Pass. |
| `cargo fmt --manifest-path runtime/Cargo.toml --check` | Pass. |
| `cargo clippy --manifest-path runtime/Cargo.toml --locked --all-targets -- -D warnings` | Pass, no warnings. |
| `.venv/bin/python -m unittest discover -s conformance -p 'test_*.py' -v` | 6 pass. |
| `cargo test --manifest-path experiments/quickjs-global-surface-spike/Cargo.toml --locked --no-run` | Pass. |
| `cargo test --manifest-path experiments/quickjs-global-surface-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 19 pass; 0 failed/ignored, 0 doc tests. |
| `cargo fmt --manifest-path experiments/quickjs-global-surface-spike/Cargo.toml --check` | Pass. |
| `cargo clippy --manifest-path experiments/quickjs-global-surface-spike/Cargo.toml --locked --all-targets -- -D warnings` | Pass, no warnings. |
| `cargo test --manifest-path experiments/quickjs-dynamic-code-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 14 pass; 0 doc tests. Original guards/tests unchanged. |
| `cargo test --manifest-path experiments/javascript-static-analysis-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 24 pass; 0 doc tests. |
| `cargo test --manifest-path experiments/quickjs-module-capture-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 14 pass; 0 doc tests. |
| `cargo test --manifest-path experiments/quickjs-lifecycle-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 15 behavior + 3 expected compile-fail doc tests pass. |
| `cargo test --manifest-path experiments/javascript-engine-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 10 pass; 0 doc tests. |
| `cargo test --manifest-path experiments/boa-reaction-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 14 pass, including preserved expected-panic control; 0 doc tests. |

The new experiment's complete test command was additionally run with child environment `TZ=UTC` and `TZ=Asia/Shanghai` under the same 60-second bound: 19 pass each. Negative-control epoch offsets are 0 and -480 respectively; the restricted behavior is identical. This is same-host timezone variation, not platform proof or full locale testing.

Development corrections: stack overflow was initially assumed to yield InternalError; pinned execution showed RangeError, so the test now asserts the actual standard family. Hidden extension-family prototype handling has a separate private-injection test. Expanded inventory covers specialized function prototypes and array unscopables. BigInt's missing own locale method receives the RFC-required locked guard, and the intrinsic expectation includes it. No old tests were weakened.

Additional checks:

- `.venv/bin/python /tmp/universal-source-engine-doc-audit.py`: established Markdown structure/local-link/anchor, external URL syntax, JSON/schema/fenced-example and text whitespace audit passes: 58 Markdown files, 436 local links, 41 anchors, 86 external URL syntax checks, 51 JSON files, 2 schemas, 3 fenced JSON examples and 172 text files. External URL syntax is not availability validation.
- Five pinned upstream API/engine files byte-match local registry sources; official ES2023 edition retrieved for property-table review.
- Independent Python comparison of RFC §9 names with `allowed-globals.json`: exact match. New lockfile equals the prior dynamic-code lockfile after root-package-name replacement.
- `git diff --check` and `git diff --cached --check`: pass. Complete diff, including every new file, reviewed.
- `git diff --exit-code 6476e337bdf4677324b5a2f065a9f2ddd936a737 -- runtime spec conformance examples docs/decisions experiments/quickjs-dynamic-code-spike experiments/javascript-static-analysis-spike experiments/quickjs-module-capture-spike experiments/quickjs-lifecycle-spike experiments/javascript-engine-spike experiments/boa-reaction-spike`: pass, empty diff.

## Files and delivery

Exactly 19 files comprise this unit:

- `docs/ARCHITECTURE.md`
- `docs/ROADMAP.md`
- `docs/reviews/2026-09-28-quickjs-global-surface-spike.md`
- `docs/plans/completed/2026-09-28-quickjs-global-surface-spike.md` (moved from active)
- `experiments/quickjs-global-surface-spike/.gitignore`
- `experiments/quickjs-global-surface-spike/Cargo.toml`
- `experiments/quickjs-global-surface-spike/Cargo.lock`
- `experiments/quickjs-global-surface-spike/README.md`
- `experiments/quickjs-global-surface-spike/src/lib.rs`
- `experiments/quickjs-global-surface-spike/src/tests.rs`
- `experiments/quickjs-global-surface-spike/src/inventory.js`
- `experiments/quickjs-global-surface-spike/src/pristine-inventory.json`
- `experiments/quickjs-global-surface-spike/src/allowed-globals.json`
- `experiments/quickjs-global-surface-spike/src/allowed-intrinsics.json`
- `experiments/quickjs-global-surface-spike/src/harden.js`
- `experiments/quickjs-global-surface-spike/src/probes.js`
- `experiments/quickjs-global-surface-spike/src/traverse.js`
- `experiments/quickjs-global-surface-spike/src/binding.js`
- `experiments/quickjs-global-surface-spike/src/entry.js`

Local Conventional Commit only; no push, tag, release, settings change or ADR. Production runtime/dependencies, all earlier experiments, specification/RFC status and conformance corpus are unchanged.
