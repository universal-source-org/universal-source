# QuickJS pre-evaluation module capture feasibility

Status: complete; Outcome A — pre-evaluation capture viable. Baseline `d6c18049368826e454c91ce1df06d8cc02da3701`; starting tree clean, no other active plans.

Scope: pinned rquickjs/core/sys 0.14.0 and QuickJS-NG 0.16.2 public APIs only. Investigate RFC 0001 §2/§4 capture before source evaluation in a separate unpublished experiment. Preserve Model B, production code/dependencies, prior evidence, RFC status and engine-selection boundary.

1. Read governing documents and prior experiments; inspect public compile/resolve/link/evaluate and namespace APIs against pinned upstream source.
2. Exercise fixed local modules with host-observable phase markers, direct declarations, mutation, invalid shapes, aliases/re-exports, diagnostics, dependencies and initialization jobs. Test any public host callback strategy without source rewriting or private internals; distinguish engine phase exposure from syntax metadata.
3. Record exactly Outcome A or B with executable/source evidence, limitations and the smallest next investigation. No production loader, parser adoption, engine ADR or RFC acceptance.
4. Run required runtime/conformance suites, all existing experiments and guarded new tests, touched-crate fmt/strict Clippy, document/link/schema/whitespace checks. Review diff, complete this plan and commit locally; no publication.

## Outcome and limits

The [review](../../reviews/2026-09-28-quickjs-module-capture-spike.md) records a public-API strategy and fourteen passing tests. A private host root imports a native capture module before the unchanged package entry. QuickJS links the complete graph before traversing evaluation dependencies; the native callback captures linked operation Function values before entering any package module's evaluation. It does not invoke source functions. Top-level replacement fails post-evaluation identity checks; later operation assignment leaves saved dispatch fixed. Tests cover all five names, dependencies, invalid forms, diagnostics and initialization jobs.

Direct entry `eval` alone returns too late. Public namespaces do not reveal direct-declaration syntax; passing generator/alias/re-export negative controls establish the need for separate static source analysis. No parser is adopted, no complete loader/validator is built, and unlinked namespace access is deliberately not attempted. This removes the capture-mechanism evidence gap without claiming complete RFC enforcement. Next is a narrow candidate-parser compatibility proof over immutable ES2023 source text; production execution is not the next unit.

Model B, RFC 0001 and its proposed status, engine-selection state and production behavior/dependencies remain unchanged. No ADR is created. Private host bootstrap modules add no source-visible native-module permission or initialization export.

## Versions and build scope

rquickjs/core/sys 0.14.0, revision `d7ef5eeae702fea24c03643064de454f1c1dd4b0`; vendored QuickJS-NG 0.16.2, revision `0fdea21ff1090084e91dad812b343d92e79ba9d9`. Eight pinned public binding/engine source files fetched from upstream matched registry copies byte-for-byte. No engine patch, unsafe code or private API use.

The new independent unpublished crate enables `std` and `loader`; the latter activates already locked relative-path 2.0.1. Its lockfile reuses the lifecycle resolution, changing only the root package name. No external package versions or old crate features changed; no production dependency changed. Keeping this experiment separate preserves prior lifecycle evidence under its original feature set.

Environment: macOS 27.0 build 26A428 arm64; target `aarch64-apple-darwin`; rustc 1.96.0 (`ac68faa20`, 2026-05-25); Cargo 1.96.0 (`30a34c682`, 2026-05-25); Apple clang 21.0.0. Native C build on this host only. No cross-platform, performance or complete sandbox claim.

## Validation

Final commands below all exited 0 on 2026-09-28:

| Command | Exact result |
| --- | --- |
| `cargo test --manifest-path runtime/Cargo.toml --locked` | 53 passed: 18 library, 2 harness, 8 dispatch, 25 loader; 0 failed/ignored; 0 doc tests. |
| `cargo test --manifest-path runtime/Cargo.toml --locked --test declarative_conformance -- --nocapture` | 2 passed; 34/34 authored cases PASS. |
| `cargo build --manifest-path runtime/Cargo.toml --locked` | Passed. |
| `cargo fmt --manifest-path runtime/Cargo.toml --check` | Passed, no output. |
| `cargo clippy --manifest-path runtime/Cargo.toml --locked --all-targets -- -D warnings` | Passed, no warnings. |
| `.venv/bin/python -m unittest discover -s conformance -p 'test_*.py' -v` | 6 passed (0.031 s). |
| `cargo test --manifest-path experiments/quickjs-module-capture-spike/Cargo.toml --locked --no-run` | Passed compilation. |
| `cargo test --manifest-path experiments/quickjs-module-capture-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 14 passed (0.02 s); 0 failed/ignored; 0 doc tests. |
| `cargo fmt --manifest-path experiments/quickjs-module-capture-spike/Cargo.toml --check` | Passed, no output. |
| `cargo clippy --manifest-path experiments/quickjs-module-capture-spike/Cargo.toml --locked --all-targets -- -D warnings` | Passed, no warnings. |
| `cargo test --manifest-path experiments/quickjs-lifecycle-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 15 behavior tests and 3 compile-fail doctests passed; 0 failed/ignored. |
| `cargo test --manifest-path experiments/javascript-engine-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 10 passed; 0 failed/ignored; 0 doc tests. |
| `cargo test --manifest-path experiments/boa-reaction-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 14 passed, including expected should-panic evidence; 0 failed/ignored; 0 doc tests. |

All four experiment suites used Python `Popen(start_new_session=True)` with a 60-second timeout and process-group SIGKILL on timeout, as reproduced in the new README. No backstop fired. The old lifecycle compile-fail diagnostics and Boa should-panic trace are expected passing evidence. Only the new experiment is touched; fmt and strict Clippy run on it and the baseline runtime.

During development the first proof ran its queue query through Runtime inside `Context::with`, causing the binding's RefCell borrow panic. Moving that host query outside `with` fixed the test; capture/order assertions had already succeeded. One subsequent compile failed on two independently inferred invariant lifetimes in the identity helper; an explicit shared scoped lifetime fixed it. Final tests/lints above pass; no unsafe workaround or weakened assertion was used.

Documentation/schema/link/whitespace validation uses the established temporary checker `.venv/bin/python /tmp/universal-source-engine-doc-audit.py`, plus `git diff --check` and `git diff --cached --check`. All passed: 47 Markdown files, 371 local links, 39 anchors, 55 external URL spellings, 48 JSON files, 2 schemas, 3 fenced JSON examples and 131 nonempty text files. External URL spelling is not a comprehensive external-link availability audit; the eight pinned source files were separately fetched and verified. Both Git whitespace checks passed. The protected baseline paths are unchanged, and independent lockfile identity was verified. Complete diff review included new files, phase/ownership logic, private-name normalization, documentation consistency and dependency scope.

## File inventory and delivery

Exactly ten changed files:

- `experiments/quickjs-module-capture-spike/.gitignore`: isolated target directory exclusion.
- `experiments/quickjs-module-capture-spike/Cargo.toml`: unpublished exact-version dependency with public loader API enabled.
- `experiments/quickjs-module-capture-spike/Cargo.lock`: independent unchanged-version resolution.
- `experiments/quickjs-module-capture-spike/README.md`: mechanism, limits, dependency/build scope and guarded reproduction.
- `experiments/quickjs-module-capture-spike/src/lib.rs`: non-production crate boundary.
- `experiments/quickjs-module-capture-spike/src/tests.rs`: fourteen focused tests and small private fixture helpers.
- `docs/reviews/2026-09-28-quickjs-module-capture-spike.md`: Outcome A, exact phase/API/source evidence, negative controls and next gap.
- `docs/plans/completed/2026-09-28-quickjs-module-capture-spike.md`: completed plan and validation record.
- `docs/ARCHITECTURE.md`: records proven capture strategy and remaining syntax-analysis gap.
- `docs/ROADMAP.md`: records scoped evidence and next feasibility task without advancing Phase 3.

No production runtime, old experiment, specification/RFC, ADR, example or conformance file changed. Local Conventional Commit subject: `test(engine): prove QuickJS pre-evaluation module capture`. No push, tag, release or repository-settings change; no self-referential commit hash is stored in this plan.
