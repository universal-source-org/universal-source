# Dynamic JavaScript compilation suppression proof

Status: completed with Outcome A — dynamic compilation suppression viable. Baseline `de2c76972af070a1715d3db948c1cf600d050a82`; clean main, no other active plans. Phase 3 remains active.

Scope: pinned QuickJS-NG/rquickjs public-API suppression of eval and all function-family constructors, exact EvalError behavior, aliases/Reflect/mutation, pre-package hardening and host compile preservation. No complete sandbox, production execution, engine/parser selection, ADR or RFC change.

1. Read governing contracts, prior evidence and pinned intrinsic/compiler APIs/source.
2. Add isolated experiment with guarded and negative-control routes, intrinsic graph checks, mutation/reset/order/source-identity evidence; no source rewriting or private API.
3. Record concrete Outcome A/B and limitations, complete full baseline/prior experiment validation with external timeouts, fmt/strict Clippy and doc/link/schema/whitespace checks.
4. Complete plan, minimally update architecture/roadmap, review full diff and commit locally; no publication.

## Results

The [review](../../reviews/2026-09-28-quickjs-dynamic-code-suppression-spike.md) and [experiment](../../../experiments/quickjs-dynamic-code-spike/README.md) record 14 passing tests and 80 blocked paths with native EvalError prototype identity and no payload effects. Hidden standard Proxy guards for eval/four constructors preserve tested prototype relationships, redirect specialized constructor parents and lock the necessary global/constructor links. They precede package compilation, capture and evaluation. Negative controls demonstrate unsafe deletion-only, late-hardening and missed-parent strategies. Host compilation and Model B reset still work.

No production behavior/dependencies, RFC text/status, previous experiment, engine/parser decision or ADR changed. Next is only the restricted-global/ambient-authority feasibility gap, not production implementation.

## Validation

Local evidence: macOS 27.0 (26A428), arm64, Rust/Cargo 1.96.0. All final commands below pass. Every engine/parser test command uses an external 60-second process-group timeout; none fired. The README gives the reproducible wrapper. The new crate was test-compiled before execution; native build requirements and versions match the prior QuickJS crates.

| Exact command | Result |
| --- | --- |
| `cargo test --manifest-path runtime/Cargo.toml --locked` | 53 pass: 18 library, 2 declarative harness, 8 dispatch, 25 loader; 0 doc tests. |
| `cargo test --manifest-path runtime/Cargo.toml --locked --test declarative_conformance -- --nocapture` | 2 pass; all 34 authored declarative cases PASS. |
| `cargo build --manifest-path runtime/Cargo.toml --locked` | Pass. |
| `cargo fmt --manifest-path runtime/Cargo.toml --check` | Pass. |
| `cargo clippy --manifest-path runtime/Cargo.toml --locked --all-targets -- -D warnings` | Pass, no warnings. |
| `.venv/bin/python -m unittest discover -s conformance -p 'test_*.py' -v` | 6 pass. |
| `cargo test --manifest-path experiments/javascript-static-analysis-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 24 pass (15 original + 9 attribute-clause follow-up), 0 failed/ignored, 0 doc tests; original matrix unchanged, both old false acceptances rejected by supplemental gate. |
| `cargo test --manifest-path experiments/quickjs-module-capture-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 14 pass, 0 doc tests. |
| `cargo test --manifest-path experiments/quickjs-lifecycle-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 15 behavior tests + 3 compile-fail doc tests pass; compiler errors in those negative controls are expected. |
| `cargo test --manifest-path experiments/javascript-engine-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 10 pass, 0 doc tests. |
| `cargo test --manifest-path experiments/boa-reaction-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 14 pass, including the existing expected-panic control; 0 doc tests. |
| `cargo test --manifest-path experiments/quickjs-dynamic-code-spike/Cargo.toml --locked --no-run` | Test compilation pass. |
| `cargo test --manifest-path experiments/quickjs-dynamic-code-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 14 pass, 0 failed/ignored, 0 doc tests; 80 blocked routes asserted. |
| `cargo fmt --manifest-path experiments/quickjs-dynamic-code-spike/Cargo.toml --check` | Pass. |
| `cargo clippy --manifest-path experiments/quickjs-dynamic-code-spike/Cargo.toml --locked --all-targets -- -D warnings` | Pass, no warnings. |

Development corrections: unwrapped a public optional object conversion in the capture fixture; after expanding the route matrix from 74 to 80, corrected two stale count assertions and centralized the expected count. The new crate was rerun after correction; no previous test was changed. Negative controls intentionally execute harmless dynamic payloads and are explicitly separate from the guarded path.

Documentation and preservation checks:

- `.venv/bin/python /tmp/universal-source-engine-doc-audit.py`: pass; 55 Markdown files, 420 local links, 40 anchors, 80 external URL syntax checks, 48 JSON files, 2 schemas, 3 fenced JSON examples and 155 text files. This follows the established local audit procedure; external link syntax is not a remote availability claim.
- `git diff --check` and `git diff --cached --check`: pass. Complete diff reviewed, including all new files and the isolated lockfile.
- `git diff --exit-code de2c76972af070a1715d3db948c1cf600d050a82 -- runtime spec conformance examples docs/decisions experiments/javascript-static-analysis-spike experiments/quickjs-module-capture-spike experiments/quickjs-lifecycle-spike experiments/javascript-engine-spike experiments/boa-reaction-spike`: pass, empty diff.
- Exact byte comparison of the new lockfile with the prior module-capture lockfile after root-package-name replacement: pass. The six pinned upstream source files listed in the review also match the locally examined registry/vendor files byte-for-byte.

## Files and dependencies

Exactly 12 files comprise this unit:

- `docs/ARCHITECTURE.md`
- `docs/ROADMAP.md`
- `docs/reviews/2026-09-28-quickjs-dynamic-code-suppression-spike.md`
- `docs/plans/completed/2026-09-28-quickjs-dynamic-code-suppression-spike.md` (moved from active)
- `experiments/quickjs-dynamic-code-spike/.gitignore`
- `experiments/quickjs-dynamic-code-spike/Cargo.toml`
- `experiments/quickjs-dynamic-code-spike/Cargo.lock`
- `experiments/quickjs-dynamic-code-spike/README.md`
- `experiments/quickjs-dynamic-code-spike/src/lib.rs`
- `experiments/quickjs-dynamic-code-spike/src/tests.rs`
- `experiments/quickjs-dynamic-code-spike/src/harden.js`
- `experiments/quickjs-dynamic-code-spike/src/probes.js`

The sole direct dependency is rquickjs `=0.14.0` with default features off and std/loader on. Lockfile equality with the module-capture experiment after root-name replacement proves no new package/version or feature/dependency graph change. No production dependency or existing lockfile changed. Local Conventional Commit only; no push, tag or release.
