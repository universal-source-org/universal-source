# QuickJS Model B lifecycle feasibility

Status: complete; Outcome A — demonstrated Model B lifecycle viable, no engine selected. Baseline `01154b767b74afa5a0a8dd2d248c6a145ea37890`; clean starting tree, no active plans.

Scope: one isolated, unpublished QuickJS-NG/rquickjs experiment with exact versions; no production dependency/behavior changes, engine selection, ADR, RFC acceptance or full loader/service/converter implementation.

1. Read governing contracts, lifecycle decision and preserved experiments; inspect pinned public binding/engine APIs and lifetime behavior.
2. Exercise fresh runtime/context domains, fixed modules, roots, single-job Promise completion, whole-graph retirement, late native completion gates, partial initialization, interruption and non-executing structural extraction. Use bounded tests and an external process timeout; do not use invalid raw pointers or cross-runtime values.
3. Record exactly Outcome A or B for the demonstrated lifecycle requirements, with public API boundaries, tests and limits. Update only necessary current status and next evidence gap.
4. Run baseline Rust/Python/conformance checks, all three experiment suites, relevant fmt/strict Clippy and established document/link/JSON/whitespace checks. Review all changes, complete plan, commit locally; no publication.

## Outcome and limits

The [review](../../reviews/2026-09-28-quickjs-lifecycle-spike.md) records a concrete public-API path: one Runtime/Context per realm, explicit single-job pumping and direct Promise observation, no final drain, non-executing descriptor/class-gated candidate copying, removal of all JS roots before full runtime retirement, and generation-gated native events that carry no resolver. Negative controls demonstrate that Context-only destruction, same-runtime Persistent transfer, convenience property reads and generic serialization do not implement this boundary.

Fifteen behavioral tests and three compile-fail tests pass. Normal completion, queued/dormant Promise graphs, children/finally/nested await/species, roots, late native completion, setup failure/retry, cancellation, timeout, explicit allocator failure and publication-time expiry are covered. Host policy preserves logical reuse after ordinary failures but invalidates logic on executing cancellation/timeout/resource termination. Tests exercise primitives and small host-policy checks, not a production state machine or conformance harness.

No lifecycle-specific blocker remains in the demonstrated strategy. A full converter, complete sandbox/quota policy, module capture-before-evaluation and other engine-selection dimensions remain unproven. Next is only the public module instantiation/export-capture feasibility gap; post-evaluation reads in this experiment are not an alternate RFC rule. RFC 0001 and its proposed status are unchanged. No engine ADR or selection, production integration, spec maturity/version bump or service implementation.

## Provenance and dependencies

rquickjs/core/sys 0.14.0, `std` only, exact binding revision `d7ef5eeae702fea24c03643064de454f1c1dd4b0`; vendored QuickJS-NG 0.16.2 revision `0fdea21ff1090084e91dad812b343d92e79ba9d9`. Seven pinned binding files plus engine header/source fetched from upstream matched registry source byte-for-byte. The separate unpublished crate's lockfile equals the prior QuickJS lockfile except its root package name; no new external versions or production dependency changes. No private API, patch or unsafe lifetime trick. Public descriptor calls and allocator delegation have explicit safety/ownership comments.

Environment: macOS 27.0 build 26A428 arm64, target `aarch64-apple-darwin`, rustc 1.96.0 (ac68faa20 2026-05-25), Cargo 1.96.0 (30a34c682 2026-05-25), Apple clang 21.0.0. Native C compilation/static linking on this host only; no cross-platform proof. Existing Boa remains pinned at 0.22.0, unchanged.

## Validation

Final commands below all exited 0 on 2026-09-28:

| Command | Exact result |
| --- | --- |
| `cargo test --manifest-path runtime/Cargo.toml --locked` | 53 passed: 18 library, 2 harness, 8 dispatch, 25 loader; 0 failed/ignored; 0 doc tests. |
| `cargo test --manifest-path runtime/Cargo.toml --locked --test declarative_conformance -- --nocapture` | 2 passed; 34/34 authored cases printed PASS. |
| `cargo build --manifest-path runtime/Cargo.toml --locked` | Passed. |
| `cargo fmt --manifest-path runtime/Cargo.toml --check` | Passed, no output. |
| `cargo clippy --manifest-path runtime/Cargo.toml --locked --all-targets -- -D warnings` | Passed, no warnings. |
| `.venv/bin/python -m unittest discover -s conformance -p 'test_*.py' -v` | 6 passed (0.037 s). |
| `cargo test --manifest-path experiments/quickjs-lifecycle-spike/Cargo.toml --locked --no-run` | Passed compilation. |
| `cargo test --manifest-path experiments/quickjs-lifecycle-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 15 passed, 3 compile-fail doctests passed; 0 failed/ignored. |
| `cargo fmt --manifest-path experiments/quickjs-lifecycle-spike/Cargo.toml --check` | Passed, no output. |
| `cargo clippy --manifest-path experiments/quickjs-lifecycle-spike/Cargo.toml --locked --all-targets -- -D warnings` | Passed, no warnings. |
| `cargo test --manifest-path experiments/javascript-engine-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 10 passed (0.01 s); 0 failed/ignored; 0 doc tests. |
| `cargo fmt --manifest-path experiments/javascript-engine-spike/Cargo.toml --check` | Passed, no output. |
| `cargo clippy --manifest-path experiments/javascript-engine-spike/Cargo.toml --locked --all-targets -- -D warnings` | Passed, no warnings. |
| `cargo test --manifest-path experiments/boa-reaction-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 14 passed (0.04 s), including the expected should-panic assertion; 0 failed/ignored; 0 doc tests. |
| `cargo fmt --manifest-path experiments/boa-reaction-spike/Cargo.toml --check` | Passed, no output. |
| `cargo clippy --manifest-path experiments/boa-reaction-spike/Cargo.toml --locked --all-targets -- -D warnings` | Passed, no warnings. |

All three experiment test commands used the README-style Python POSIX process-group 60-second backstop; none fired. Intentional loop interruption was engine-controlled. New compile-fail diagnostics and the old Boa should-panic trace are expected passing evidence. Initial compilation failed once because Atom has no public `as_raw`; the helper now creates/releases a public atom from the already inspected string key. No test execution failed. Final locked compilation/tests and strict lint pass.

Documentation/JSON/schema/link/anchor/whitespace checks use `.venv/bin/python /tmp/universal-source-engine-doc-audit.py`, the existing temporary external checker, plus `git diff --check` and `git diff --cached --check`. Passed: 48 JSON files, 2 schemas, 3 fenced JSON examples, 44 Markdown files, 360 local links, 39 anchors, 45 external URL spellings and 123 nonempty text files. Both Git whitespace checks passed; protected baseline paths and isolated lockfile identity were verified. External URL spelling is checked; the nine pinned evidence files were separately fetched and verified, not a comprehensive external-link availability audit.

## Delivery scope

Exactly eleven files:

- `experiments/quickjs-lifecycle-spike/.gitignore`: crate-local target exclusion.
- `experiments/quickjs-lifecycle-spike/Cargo.toml`: unpublished exact-version experiment dependency.
- `experiments/quickjs-lifecycle-spike/Cargo.lock`: independent existing-version resolution.
- `experiments/quickjs-lifecycle-spike/README.md`: scope, evidence limits, build and repeatable guarded validation.
- `experiments/quickjs-lifecycle-spike/src/lib.rs`: three compile-fail ownership/thread probes.
- `experiments/quickjs-lifecycle-spike/src/tests.rs`: fifteen focused lifecycle probes and tiny local host-policy helpers.
- `experiments/quickjs-lifecycle-spike/src/extraction.rs`: bounded public descriptor/class-gated copying probe; not a production converter.
- `docs/reviews/2026-09-28-quickjs-lifecycle-spike.md`: Outcome A, pinned APIs, requirement/test matrix, limits and next gap.
- `docs/plans/completed/2026-09-28-quickjs-lifecycle-spike.md`: this plan and validation record.
- `docs/ARCHITECTURE.md`: records lifecycle evidence with no production or engine-selection claim.
- `docs/ROADMAP.md`: records Outcome A and next module-capture evidence gap.

Runtime, production tests/dependencies/lockfile, prior experiments/reviews/plans, specification/RFC, ADRs, examples and conformance corpus/expectations are unchanged. Full diff review covers source ownership/FFI, lifecycle scope, doc consistency, independent lockfile and protected paths. Local Conventional Commit subject: `test(engine): validate QuickJS fresh-realm lifecycle`. No push, tag, release or repository-settings change; no self-referential commit hash is stored here.
