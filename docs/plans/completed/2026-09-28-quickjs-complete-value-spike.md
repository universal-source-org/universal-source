# Complete RFC §6 value-boundary feasibility

Status: completed with Outcome A — complete RFC §6 value boundary feasible. Baseline `fb7ce8e336d9910a327568ea9921876c595d4948`; clean tree, no other active plans. Phase 3 remains active.

Scope: isolated bidirectional bounded non-executing structural conversion using the unchanged audited class bridge as an experiment-only path dependency. Add only narrowly audited public descriptor/UTF-16 wrappers as needed. Preserve all earlier evidence. No production execution/dependencies, RFC changes, selection/ADR, parser or resource-execution work.

1. Read contracts, earlier reviews/plans, exact pinned APIs and safety/ownership requirements.
2. Implement class-gated descriptors, Unicode, helper identity exclusion, records/arrays, path-cycle rejection and expanded-copy accounting; prevalidate incoming portable trees and construct using native own-property definitions/original prototypes.
3. Test complete domain, hostile hooks/mutations, negative controls, round trips and each bound. Resolve exact Outcome A or a new precise Outcome B without weakening §6.
4. Audit unsafe code, run all prior/new experiments under external timeouts and runtime/Python/docs/schema/link/whitespace validation, record outcome/limits, complete plan, review diff and commit locally. No push/tag/release.

## Results and remaining limits

The [experiment](../../../experiments/quickjs-complete-value-spike/README.md) and [review](../../reviews/2026-09-28-quickjs-complete-value-spike.md) record 20 passing tests. The complete outgoing path rejects prototype-mutated native Map/Set, active/nested/revoked proxies, helper/intrinsic identities and invalid shapes without any source hook/trap. Strings/keys use strict public UTF-16 extraction; cycles fail and repeated references consume expanded-copy budgets. Incoming values are prevalidated before native allocation and own data definitions, using original prototypes despite source poisoning. Fresh realms remain independent.

The unchanged class bridge is a path dependency. Three new unsafe blocks in `src/inspect.rs` (lines 56, 93, 108) handle public descriptor ownership, UTF-16 buffer release/extraction; the bridge retains its one block at `src/bridge.rs:37`. Zero unsafe functions/impls. Every block was audited for public provenance, class/context preconditions, initialized data and exact release/ownership transfer. No private engine access or manual extern declaration. Miri not run: native C FFI is not meaningfully validated by ordinary Miri execution.

Default budgets: depth 16 (root 0), 1,024 expanded nodes, 4,096 bytes per text/key, 16,384 total UTF-8 text/key bytes, 32,768 expanded bytes (8 bytes per node plus text). Depth configuration caps at 64. Pristine intrinsic graph: depth 16 / 4,096 identities; combined helper list: 8,192. Native full-key/string buffers can be materialized before budget checks; test heap/stack caps and the external timeout are scaffolding, not production resource enforcement. Next unit: CPU/allocation/memory/RegExp controls and trustworthy failure/retirement evidence. No production service, module containment, parser selection or platform/conformance work was added.

## Validation

Local macOS arm64, Rust/Cargo 1.96.0. All final commands below pass. Every engine/parser test command used the established external 60-second process-group timeout; none fired. The previous runner was adapted at `/tmp/universal-source-complete-value-validation.py`; logs/results are under `/tmp/universal-source-complete-value-validation/`. A reproducible timeout wrapper is in the new experiment README.

| Exact command | Result |
| --- | --- |
| `cargo test --manifest-path experiments/quickjs-complete-value-spike/Cargo.toml --locked --no-run` | Pass before executable validation. |
| `cargo test --manifest-path experiments/quickjs-complete-value-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 20 pass; zero failures/ignored/doc tests. |
| `cargo fmt --manifest-path experiments/quickjs-complete-value-spike/Cargo.toml --check` | Pass. |
| `cargo clippy --manifest-path experiments/quickjs-complete-value-spike/Cargo.toml --locked --all-targets -- -D warnings` | Pass. |
| `cargo test --manifest-path experiments/quickjs-class-bridge-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 10 pass, unchanged bridge. |
| `cargo test --manifest-path experiments/quickjs-value-boundary-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 6 behavior + 2 expected compile-fail checks pass. |
| `cargo test --manifest-path experiments/quickjs-global-surface-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 19 pass. |
| `cargo test --manifest-path experiments/quickjs-dynamic-code-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 14 pass; original 80-route denial corpus preserved. |
| `cargo test --manifest-path experiments/javascript-static-analysis-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 24 pass, including nine import-attribute follow-up tests. |
| `cargo test --manifest-path experiments/quickjs-module-capture-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 14 pass. |
| `cargo test --manifest-path experiments/quickjs-lifecycle-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 15 behavior + 3 expected compile-fail checks pass. |
| `cargo test --manifest-path experiments/javascript-engine-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 10 pass. |
| `cargo test --manifest-path experiments/boa-reaction-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 14 pass, including the preserved expected-panic control. |
| `cargo test --manifest-path runtime/Cargo.toml --locked` | 53 pass: 18 library, 2 declarative harness, 8 dispatch, 25 loader. |
| `cargo test --manifest-path runtime/Cargo.toml --locked --test declarative_conformance -- --nocapture` | 2 harness tests pass again; all 34 operation cases PASS. |
| `cargo build --manifest-path runtime/Cargo.toml --locked` | Pass. |
| `cargo fmt --manifest-path runtime/Cargo.toml --check` | Pass. |
| `cargo clippy --manifest-path runtime/Cargo.toml --locked --all-targets -- -D warnings` | Pass. |
| `.venv/bin/python -m unittest discover -s conformance -p 'test_*.py' -v` | 6 pass. |

Prior Phase 3 total: 131 tests/checks, unchanged. Development corrections: the UTF-16 length requires checked conversion from the public sys size type to usize; strict Clippy requested a collapsed test conditional. Both were fixed, with final checks passing. Pristine roots were expanded to include shared iterator/specialized-function intrinsics, then tested after stripping their properties and prototypes.

Additional validation:

- `.venv/bin/python /tmp/universal-source-engine-doc-audit.py`: established Markdown/local-link/anchor, external URL syntax, JSON/schema/fenced-example and whitespace audit. Pass: 67 Markdown files, 470 local links, 41 anchors, 102 external URL syntax checks, 51 JSON files, 2 schemas, 3 fenced JSON examples and 200 text files. External URL syntax checks do not imply general availability.
- Nine exact pinned upstream files byte-match the local registry: QuickJS header/source; rquickjs Value/Object/Property/Atom/String/Array; aarch64 Apple sys binding. No engine source changed.
- Lock comparison confirms every external package/version/checksum matches the class-bridge baseline. The two local experiment packages are the only root/dependency differences.
- `rg -n '\bunsafe\b' experiments/quickjs-complete-value-spike/src experiments/quickjs-class-bridge-spike/src/bridge.rs`: all four unsafe expressions explicitly reviewed; zero unsafe functions/impls.
- `git diff --check` and `git diff --cached --check`; complete diff including all new files reviewed before commit.
- `git diff --exit-code fb7ce8e336d9910a327568ea9921876c595d4948 -- runtime spec conformance examples docs/decisions experiments/quickjs-class-bridge-spike experiments/quickjs-value-boundary-spike experiments/quickjs-global-surface-spike experiments/quickjs-dynamic-code-spike experiments/javascript-static-analysis-spike experiments/quickjs-module-capture-spike experiments/quickjs-lifecycle-spike experiments/javascript-engine-spike experiments/boa-reaction-spike`: empty; all prior evidence, production code/dependencies and normative contracts preserved.

## Files and delivery

Thirteen files: minimal `ARCHITECTURE.md`/`ROADMAP.md` evidence updates, review, this completed plan moved from active, and nine isolated experiment files (`.gitignore`, `Cargo.toml`, `Cargo.lock`, `README.md`, `src/lib.rs`, `src/inspect.rs`, `src/transfer.rs`, `src/roots.js`, `src/tests.rs`).

Local Conventional Commit only. No push/tag/release. Pinned versions unchanged; RFC 0001 unchanged/proposed; no engine/parser selection or ADR. JavaScript remains unimplemented in production. Clean final tree checked after local commit.
