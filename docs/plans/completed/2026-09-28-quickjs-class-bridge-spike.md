# Audited public-API object-class bridge

Status: completed with Outcome A — audited public-API class bridge viable. Baseline `d1d94be87a4c43b003a4568d8f4176034b9b3382`; clean tree, no other active plans. Phase 3 remains active.

Scope: close only the positive object-class inspection prerequisite using a minimal audited unsafe bridge over pinned public QuickJS C APIs. Capture ordinary object/array class identities from trusted host-created values; reject all other object classes before reflection. No complete §6 converter, descriptor copier, production code, dependency upgrade, RFC change, engine selection or ADR.

1. Read governing context, prior evidence and exact public header/sys/binding implementation.
2. Implement a tiny safe class API with narrowly contained unsafe FFI, explicit lifetime/context invariants and no private class constants/layout. Test native/proxy mutation, hostile hooks, host-native objects and conservative unknown-class rejection. Preserve the previous blocker tests unchanged.
3. Audit every unsafe operation; run new and all prior experiments under established external timeouts, runtime/build/conformance baseline, fmt/strict Clippy and documentation/link/JSON/schema/whitespace checks.
4. Resolve Outcome A or B, document limits and next unit, complete plan, review full diff and commit locally. No push/tag/publication.

## Results and remaining limits

The [experiment](../../../experiments/quickjs-class-bridge-spike/README.md) and [review](../../reviews/2026-09-28-quickjs-class-bridge-spike.md) record ten passing tests. The safe interface captures ordinary object/array class IDs using public native allocations and `JS_GetClassID`; all other classes fail closed. Prototype-mutated Map/Set remain non-ordinary, and all eight active/revoked proxy cases trigger zero traps. Hostile getters, constructor/tag/species/iterator/coercion/then hooks likewise never execute during classification. Labeled negative controls intentionally execute them.

Exactly one unsafe block at `experiments/quickjs-class-bridge-spike/src/bridge.rs:37`, zero unsafe functions/impls. It borrows the supported `Value::as_raw` representation for public `JS_GetClassID`; no ownership conversion or private layout/ID/API is used. The function-local unsafe exception and every safety invariant were reviewed explicitly. Context mismatches fail before querying the candidate. Miri was not run because ordinary Miri execution does not validate the native C path.

The standard native matrix contains 26 fixtures under three prototype replacements; the private native matrix contains 20 fixtures before and after two replacements. Registered Rust callable objects are rejected; an ordinary host-authored wrapper remains an ordinary engine object and requires future identity-based policy. Positive class eligibility is not transfer acceptance. No recursive copying, descriptor reader, Unicode, cycles, expanded accounting or incoming conversion was implemented. Next: the complete §6 proof using this safe gate, with all those requirements and helper identity exclusion still open.

## Validation

Local macOS arm64, Rust/Cargo 1.96.0. All final commands below pass. Engine/parser test commands used the established 60-second external process-group timeout; none fired. The existing temporary runner was adapted as `/tmp/universal-source-class-validation.py`, with logs/results in `/tmp/universal-source-class-validation/`. The experiment README contains the reproducible single-suite timeout wrapper.

| Exact command | Result |
| --- | --- |
| `cargo test --manifest-path experiments/quickjs-class-bridge-spike/Cargo.toml --locked --no-run` | Pass before first execution after compile fixes. |
| `cargo test --manifest-path experiments/quickjs-class-bridge-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 10 pass; 0 failed/ignored, 0 doc tests. |
| `cargo fmt --manifest-path experiments/quickjs-class-bridge-spike/Cargo.toml --check` | Pass. |
| `cargo clippy --manifest-path experiments/quickjs-class-bridge-spike/Cargo.toml --locked --all-targets -- -D warnings` | Pass; no warnings. |
| `cargo test --manifest-path experiments/quickjs-value-boundary-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | Preserved 6 behavior + 2 expected compile-fail tests pass. |
| `cargo test --manifest-path experiments/quickjs-global-surface-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 19 pass. |
| `cargo test --manifest-path experiments/quickjs-dynamic-code-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 14 pass; original 80-route denial corpus preserved. |
| `cargo test --manifest-path experiments/javascript-static-analysis-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 24 pass, including all 9 import-attribute follow-up tests. |
| `cargo test --manifest-path experiments/quickjs-module-capture-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 14 pass. |
| `cargo test --manifest-path experiments/quickjs-lifecycle-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 15 behavior + 3 expected compile-fail tests pass. |
| `cargo test --manifest-path experiments/javascript-engine-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 10 pass. |
| `cargo test --manifest-path experiments/boa-reaction-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 14 pass, including preserved expected-panic control. |
| `cargo test --manifest-path runtime/Cargo.toml --locked` | 53 pass: 18 library, 2 declarative harness, 8 dispatch, 25 loader. |
| `cargo test --manifest-path runtime/Cargo.toml --locked --test declarative_conformance -- --nocapture` | 2 pass again; all 34 authored operation cases PASS. |
| `cargo build --manifest-path runtime/Cargo.toml --locked` | Pass. |
| `cargo fmt --manifest-path runtime/Cargo.toml --check` | Pass. |
| `cargo clippy --manifest-path runtime/Cargo.toml --locked --all-targets -- -D warnings` | Pass; no warnings. |
| `.venv/bin/python -m unittest discover -s conformance -p 'test_*.py' -v` | 6 pass. |

Development corrections: initial test code used an unavailable Ctx job query and unnecessarily unified invariant context lifetimes; these were corrected without unsafe casts. Nesting `Context::with` on the same runtime raised the binding's RefCell lock panic. The final mismatch test uses supported Persistent save/restore in sequential same-runtime scopes, and separate runtime scopes for the other case. Final suites pass; no prior test was changed.

Additional validation:

- `.venv/bin/python /tmp/universal-source-engine-doc-audit.py`: established documentation/Markdown/local-link/anchor, JSON/schema/fenced-example and whitespace audit; pass: 64 Markdown files, 458 local links, 41 anchors, 100 external URL syntax checks, 51 JSON files, 2 schemas, 3 fenced JSON examples and 189 text files. External URL syntax checking does not imply general URL availability.
- Six pinned upstream files byte-match the registry: QuickJS header/implementation; rquickjs Value/Object/Array; target sys bindings. No manual ABI declaration.
- New lockfile equals the previous value-boundary lockfile after only root-name replacement.
- `rg -n '\bunsafe\b' experiments/quickjs-class-bridge-spike/src`: one unsafe expression, explicitly reviewed against borrowed lifetime, context identity, ownership and public query behavior.
- `git diff --check` and `git diff --cached --check`; full diff including new files reviewed before commit.
- `git diff --exit-code d1d94be87a4c43b003a4568d8f4176034b9b3382 -- runtime spec conformance examples docs/decisions experiments/quickjs-value-boundary-spike experiments/quickjs-global-surface-spike experiments/quickjs-dynamic-code-spike experiments/javascript-static-analysis-spike experiments/quickjs-module-capture-spike experiments/quickjs-lifecycle-spike experiments/javascript-engine-spike experiments/boa-reaction-spike`: pass, empty diff for all prior evidence, production and normative contracts.

## Files and delivery

Eleven files comprise this unit: `docs/ARCHITECTURE.md`, `docs/ROADMAP.md`, this completed plan (moved from active), the focused review, and seven experiment files (`.gitignore`, `Cargo.toml`, `Cargo.lock`, `README.md`, `src/lib.rs`, `src/bridge.rs`, `src/tests.rs`).

Local Conventional Commit only. No push/tag/release, production runtime/dependency change, RFC modification/acceptance, engine/parser selection or ADR. Pinned versions remain unchanged. Phase 3 remains active; this is class inspection feasibility, not complete §6 conversion or production sandboxing.
