# Boa reaction ownership feasibility

Status: complete; Outcome B, Boa public APIs insufficient for faithful F1 suppression. Baseline: `c54886f4a7093de13ef911ee2559938085385183`; starting worktree clean, no other active plans.

Scope: test F1 only using pinned Boa public callback/job APIs in a separate unpublished experiment. Preserve QuickJS-NG evidence, production runtime, specification and existing ADRs. No engine selection, RFC acceptance, production integration, services or unrelated engine hardening.

1. Read governing documents and prior evidence; inspect current authoritative Boa hooks, reaction/resolver/await machinery and executor source.
2. Build a host-token experiment for synthetic A/B invocations. Compare observation, dequeue discard and explicitly nonconforming callback-suppression probes; inspect child Promise consequences, chains, missing handlers and valid B work.
3. Record one evidence-backed disposition, public/internal API boundaries and smallest binding/lifecycle follow-up; minimally update architecture/roadmap.
4. Run all six baseline checks, existing QuickJS-NG tests, Boa tests/fmt/strict Clippy and documentation/link/JSON/whitespace checks; review full diff, complete plan and commit locally. No publication.

## Result

The [focused review](../../reviews/2026-09-28-boa-reaction-ownership.md) records public API/source analysis and all 14 test observations. Boa 0.22.0 at `337a3668a0dc86dd401ea20906e782249a64a228` preserves host registration tokens for `.then` and internal await callbacks. Opaque jobs hide these tokens from the enqueue/dequeue boundary; empty-handler reactions have no callback token. Queued work can be dropped, but enqueue-time attribution mistakes later A reactions for B. Diagnostic callback substitution changes child settlement, can execute a source species resolver, and error substitution for await triggers a pinned engine assertion.

The positive part is durable callback metadata and physical FIFO queue control; F1 remains blocked for complete pre-effect reaction suppression. Ordinary state persists and B callbacks can run, including on the same upstream Promise, but no faithful public-hook mechanism combines all requirements. Resolving this needs a specified Promise-graph abandonment policy and an engine/upstream reaction hook, or an independently reviewed lifecycle change. No engine selected, no ADR, no normative amendment, no production implementation. The next unit is a binding/lifecycle design decision, not a broad engine survey.

Only public APIs compile into the spike; internals were inspected as evidence, not called or patched. Modes that skip callbacks explicitly violate the hook's Call requirement and exist only to measure counterexamples. No JavaScript-owned token, source `.then` interception, private API, fork, production service, converter, module or interruption integration was introduced. The 100-job guard and external timeout are test safeguards, not runtime resource guarantees.

Environment: macOS 27.0 build 26A428 arm64, Rust target `aarch64-apple-darwin`, rustc 1.96.0 (ac68faa20 2026-05-25), Cargo 1.96.0 (30a34c682 2026-05-25). Registry VCS identity and five core source files matched the pinned upstream checkout byte-for-byte. No cross-platform claim.

## Validation

Final commands all exited 0 on 2026-09-28:

| Command | Exact result |
| --- | --- |
| `cargo test --manifest-path runtime/Cargo.toml --locked` | 53 passed (18 library, 2 conformance harness, 8 dispatch, 25 loader); 0 failed/ignored; 0 doc tests. |
| `cargo test --manifest-path runtime/Cargo.toml --locked --test declarative_conformance -- --nocapture` | 2 tests passed; 34/34 authored declarative cases printed PASS. |
| `cargo build --manifest-path runtime/Cargo.toml --locked` | Passed. |
| `cargo fmt --manifest-path runtime/Cargo.toml --check` | Passed, no output. |
| `cargo clippy --manifest-path runtime/Cargo.toml --locked --all-targets -- -D warnings` | Passed, no warnings. |
| `.venv/bin/python -m unittest discover -s conformance -p 'test_*.py' -v` | 6 passed. |
| `cargo test --manifest-path experiments/javascript-engine-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | Existing QuickJS-NG tests: 10 passed, 0 failed/ignored; 0 doc tests; 0.01 s. |
| `cargo test --manifest-path experiments/boa-reaction-spike/Cargo.toml --locked --no-run` | Passed compilation. |
| `cargo test --manifest-path experiments/boa-reaction-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 14 passed, 0 failed/ignored; 0 doc tests; 0.04 s. One named should-panic test deliberately confirms the await assertion. |
| `cargo fmt --manifest-path experiments/boa-reaction-spike/Cargo.toml --check` | Passed, no output. |
| `cargo clippy --manifest-path experiments/boa-reaction-spike/Cargo.toml --locked --all-targets -- -D warnings` | Passed, no warnings. |

Both experiment suites ran under the README-style Python POSIX process-group 60-second backstop; neither backstop fired. The initial unlocked Boa build generated its isolated lockfile and failed on two incorrect import paths to public types; switching to the documented public `builtins::promise::Promise` and `object::builtins::JsFunction` paths fixed compilation. No engine patch or dependency version change was needed. Subsequent locked compilation/tests/Clippy passed. No behavioral test failed; diagnostic failures are explicit passing counterexample assertions.

Documentation check: `.venv/bin/python /tmp/universal-source-engine-doc-audit.py` (temporary external checker reused from the prior unit), covering JSON/schema/fenced examples, Markdown structure, local links/anchors, external URL syntax, whitespace and final newlines. Passed: 48 JSON files, 2 schemas, 3 fenced JSON blocks, 39 Markdown files, 333 local links, 36 anchors, 33 external URL spellings and 112 nonempty text files. `git diff --check` and `git diff --cached --check` passed. Protected baseline paths were verified unchanged. No comprehensive external-link availability audit; the relevant authoritative source/docs were retrieved separately.

## Delivery scope

Nine changed files:

- `docs/ARCHITECTURE.md`: records the separate Boa experiment, Outcome B and next design boundary.
- `docs/ROADMAP.md`: Phase 3 still active; F1 unresolved; binding/lifecycle decision next.
- `docs/reviews/2026-09-28-boa-reaction-ownership.md`: focused evidence, semantics and follow-up options.
- `docs/plans/completed/2026-09-28-boa-reaction-ownership.md`: this record.
- `experiments/boa-reaction-spike/.gitignore`: ignores only this crate's target directory.
- `experiments/boa-reaction-spike/Cargo.toml`: isolated unpublished crate, exact boa_engine 0.22.0 with defaults disabled.
- `experiments/boa-reaction-spike/Cargo.lock`: 134 external resolved packages, including target-specific dependencies; no production lockfile modification.
- `experiments/boa-reaction-spike/README.md`: scope, rerun command, versions and expected-panic explanation.
- `experiments/boa-reaction-spike/src/lib.rs`: host-token hook/queue probe and 14 tests.

The previous QuickJS-NG experiment, initial evaluation, specification, ADRs, runtime, conformance corpus and example files remain unchanged from baseline. Full diff review covers new files and lockfile, scope, sensitive data, generated artifacts and status consistency. Local Conventional Commit subject: `test(engine): probe Boa reaction ownership`. Nothing is pushed, tagged, released or changed in repository settings. No self-referential commit hash is stored here.
