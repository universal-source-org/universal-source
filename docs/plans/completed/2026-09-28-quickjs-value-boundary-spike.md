# Complete non-executing value-boundary feasibility

Status: completed with Outcome B — blocker demonstrated. Baseline `2f4f31924ce10868ecc2bbd12d2e68c47822b8d3`; clean tree, no other active plans. Phase 3 remains active.

Scope: determine whether RFC 0001 §6 is feasible through the pinned public embedding surface with no unsafe Rust in this unit. Review the earlier C-API-backed partial copier first. Prove the ordinary-object classification/descriptor prerequisite before building recursive conversion. If that prerequisite is blocked, preserve the smallest executable counterexample and stop expansion, as requested; do not claim partial coverage as Outcome A.

1. Read governing contracts, prior reviews/plans/experiments and pinned source/public APIs.
2. Add isolated safe-API prerequisite probes, unchanged hardening reuse and explicit negative controls. Resolve to complete Outcome A or precise Outcome B; no source rewriting, dependency upgrade, production implementation, RFC change or engine/parser decision.
3. Run established baseline/all Phase 3 suites with external timeouts, new-crate fmt/strict Clippy, documentation/link/schema/whitespace and preservation checks.
4. Record outcome, untouched/unproven requirements and smallest next task; move plan to completed, review complete diff, commit locally. No push/tag/release.

## Results and scope stop

The [review](../../reviews/2026-09-28-quickjs-value-boundary-spike.md) and [experiment](../../../experiments/quickjs-value-boundary-spike/README.md) preserve the missing safe ordinary-class query and prototype-mutated Map counterexample. Six behavior tests pass: ambiguous record/native admission, structurally permitted class record, getter reads, captured native stringifier calling a source tag getter, Proxy enumeration trap, and poisoned reflection with preserved eval guard. Two compile-fail doc tests confirm E0133 for raw public `JS_GetClassID` and `JS_GetOwnProperty` calls. The crate forbids unsafe code; neither raw function executes.

No hook executes in the direct prerequisite gate; getters/traps execute only in separately labeled negative controls. The gate blocks valid records too and is not a working converter or RFC INVALID_RESULT classifier. No recursive copier, incoming conversion or transfer budgets were implemented after the blocker. Unicode/surrogates, complete brand/descriptor/array validation, cycles and repeated-reference expanded output accounting remain unproved. Test safeguards (16 MiB memory, 256 KiB stack and 60-second external timeout) are not normative or transfer constants.

The smallest next unit is a reviewed safe binding for positive ordinary-object classification and a non-executing descriptor path after that gate. Public C APIs exist; an upstream safe binding addition or separately authorized audited unsafe wrapper may close the prerequisite. No private ABI, engine patch, RFC change or different engine is shown necessary. That follow-up must precede resuming the full §6 proof. No scope expansion, dependency upgrade or source rewriting was performed.

## Validation

Local macOS arm64; Rust/Cargo 1.96.0. The established external runner was reused as `/tmp/universal-source-value-validation.py`, with logs/results under `/tmp/universal-source-value-validation/`. Every engine/parser test command below used a 60-second subprocess timeout and process-group SIGKILL on expiry, as reproduced in the experiment README. None expired. Compile-fail errors and the earlier Boa expected panic are successful negative controls, not unexplained failures.

| Exact command | Result |
| --- | --- |
| `cargo test --manifest-path experiments/quickjs-value-boundary-spike/Cargo.toml --locked --no-run` | Pass before test execution. |
| `cargo test --manifest-path experiments/quickjs-value-boundary-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 6 behavior + 2 expected E0133 compile-fail tests pass; none ignored. |
| `cargo fmt --manifest-path experiments/quickjs-value-boundary-spike/Cargo.toml --check` | Pass. |
| `cargo clippy --manifest-path experiments/quickjs-value-boundary-spike/Cargo.toml --locked --all-targets -- -D warnings` | Pass, no warnings. |
| `cargo test --manifest-path experiments/quickjs-global-surface-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 19 pass. |
| `cargo test --manifest-path experiments/quickjs-dynamic-code-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 14 pass, preserving all 80 denied routes. |
| `cargo test --manifest-path experiments/javascript-static-analysis-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 24 pass: 15 static-analysis and 9 import-attribute tests. |
| `cargo test --manifest-path experiments/quickjs-module-capture-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 14 pass. |
| `cargo test --manifest-path experiments/quickjs-lifecycle-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 15 behavior + 3 compile-fail tests pass. |
| `cargo test --manifest-path experiments/javascript-engine-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 10 pass. |
| `cargo test --manifest-path experiments/boa-reaction-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 14 pass, including preserved expected-panic control. |
| `cargo test --manifest-path runtime/Cargo.toml --locked` | 53 pass: 18 library, 2 declarative harness, 8 dispatch, 25 loader. |
| `cargo test --manifest-path runtime/Cargo.toml --locked --test declarative_conformance -- --nocapture` | 2 pass again; all 34 authored operation cases PASS. |
| `cargo build --manifest-path runtime/Cargo.toml --locked` | Pass. |
| `cargo fmt --manifest-path runtime/Cargo.toml --check` | Pass. |
| `cargo clippy --manifest-path runtime/Cargo.toml --locked --all-targets -- -D warnings` | Pass, no warnings. |
| `.venv/bin/python -m unittest discover -s conformance -p 'test_*.py' -v` | 6 pass. |

Additional checks:

- `.venv/bin/python /tmp/universal-source-engine-doc-audit.py`: established Markdown structure, local links/anchors, external URL syntax, JSON/schema/fenced examples and text whitespace audit. Pass: 61 Markdown files, 447 local links, 42 anchors, 92 external URL syntax checks, 51 JSON files, 2 schemas, 3 fenced JSON examples and 180 text files. External URL syntax is not general availability validation.
- Six pinned upstream files byte-match the local registry: rquickjs `value.rs`, `object.rs`, `object/property.rs`, `class.rs`, and QuickJS `quickjs.h`, `quickjs.c`.
- New lockfile equals the restricted-global lockfile after root-name replacement; exact versions/revisions remain unchanged.
- `git diff --check` and `git diff --cached --check`; complete diff including every new file reviewed before commit.
- `git diff --exit-code 2f4f31924ce10868ecc2bbd12d2e68c47822b8d3 -- runtime spec conformance examples docs/decisions experiments/quickjs-global-surface-spike experiments/quickjs-dynamic-code-spike experiments/javascript-static-analysis-spike experiments/quickjs-module-capture-spike experiments/quickjs-lifecycle-spike experiments/javascript-engine-spike experiments/boa-reaction-spike`: pass, empty diff; all existing production/specification/evidence paths unchanged.

## Files and delivery

Ten files comprise this unit:

- `docs/ARCHITECTURE.md`
- `docs/ROADMAP.md`
- `docs/reviews/2026-09-28-quickjs-value-boundary-spike.md`
- `docs/plans/completed/2026-09-28-quickjs-value-boundary-spike.md` (moved from active)
- `experiments/quickjs-value-boundary-spike/.gitignore`
- `experiments/quickjs-value-boundary-spike/Cargo.toml`
- `experiments/quickjs-value-boundary-spike/Cargo.lock`
- `experiments/quickjs-value-boundary-spike/README.md`
- `experiments/quickjs-value-boundary-spike/src/lib.rs`
- `experiments/quickjs-value-boundary-spike/src/tests.rs`

Local Conventional Commit only; no push, tag, release or settings changes. Production runtime/dependencies, all prior experiments, conformance corpus and RFC text are unchanged. RFC 0001 remains proposed. No engine/parser selection or ADR. Phase 3 remains active.
