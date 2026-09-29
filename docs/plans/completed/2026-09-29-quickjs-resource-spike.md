# QuickJS resource-enforcement feasibility

Status: completed with Outcome B — RegExp compilation checkpoint blocker demonstrated. Baseline `be9120b85f7a1a7664bacb71c2efd240355f8f10`; clean starting tree, no other active plans. Phase 3 remains active.

Scope: one isolated pinned QuickJS-NG/rquickjs resource safety proof against proposed RFC 0001 §§5/7/10, covering CPU, heap/allocation, stack, RegExp and retirement. Resolve to Outcome A or preserve the first architecture-critical Outcome B counterexample and stop expanding scope. No production changes, source rewriting, dependency upgrades, engine/parser selection, ADR, RFC acceptance or publication.

1. Read governing documents, prior reviews/plans/experiments and exact pinned public APIs/source.
2. Exercise bounded resource controls and trusted termination/retirement with deterministic host state; contain every potentially hanging engine execution in an external process timeout. Prioritize native RegExp paths after source inspection; timeout containment is never positive engine evidence.
3. Record concrete observations, public API limitations, unsafe audit, uncovered cases and smallest next task in a focused review and experiment README.
4. Rerun all prior Phase 3 experiments and Phase 2 build/tests/fmt/strict Clippy, all 34 declarative cases, Python conformance and established document/link/schema/JSON/whitespace checks. Review complete diff, complete this plan and commit locally; do not push.

## Result and stopping point

The [review](../../reviews/2026-09-29-quickjs-resource-spike.md) and [experiment](../../../experiments/quickjs-resource-spike/README.md) record Outcome B. The fixed local forward-backreference RegExp fixture compiles for long periods without invoking the installed interrupt callback. Under 8 MiB engine memory / 128 KiB stack settings, 1,000, 4,000 and 16,000 references return successfully with zero callbacks; 64,000 references exceed the five-second safety guard. The compiler has repeated full-pattern scans without timeout polling. Matching's independent interrupt checks do work.

Scope expansion stopped at the blocker. Eleven existing-in-progress control tests were repaired/finalized and pass. They demonstrate CPU/module/job stops, trusted reason separation and source catch/finally suppression for CPU interruption; retained heap rejection and recoverable stack growth; direct Promise state and no final drain; and candidate rejection after host observation. Default heap errors and stack RangeErrors are catchable and lack an authenticated host reason. Complete exhaustion classification, adversarial §6 enumeration/string buffers, remaining initialization/stack/job variants and integrated delivery after each failure remain explicitly unproven.

Earlier lifecycle tests preserve late-delivery generations, logical disposal, callback/root release and fresh independent runtime evidence. No second lifecycle architecture was introduced. This task adds no unsafe Rust, custom allocator, production dependency or RFC amendment. Next: a scoped remedy review for compiler interruption/containment against this counterexample, before resuming the broader resource proof. Module/path/parser work and engine selection remain later tasks.

## Validation

Host: macOS 27.0 (26A428), arm64, Rust/Cargo 1.96.0. Build/test profile only. All regular tests/checks below pass. The separately reported compiler watchdog failure is the Outcome B evidence, not a passing resource-control test.

| Exact command | Result |
| --- | --- |
| `cargo test --manifest-path experiments/quickjs-resource-spike/Cargo.toml --locked --no-run` | Pass; precompiled before execution. |
| `cargo test --manifest-path experiments/quickjs-resource-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 11 unit tests pass; 0 failed/ignored/doc tests. External 60-second guard did not fire. |
| `cargo build --manifest-path experiments/quickjs-resource-spike/Cargo.toml --locked --bin regexp_compile` | Pass. |
| `python3 experiments/quickjs-resource-spike/probe_regexp.py` | **Exit 2**, external five-second safety kill at 64,000 references. Three smaller inputs return with zero callbacks. Not credited as successful engine interruption or retirement. |
| `cargo fmt --manifest-path experiments/quickjs-resource-spike/Cargo.toml --check` | Pass. |
| `cargo clippy --manifest-path experiments/quickjs-resource-spike/Cargo.toml --locked --all-targets -- -D warnings` | Pass. |
| `cargo test --manifest-path experiments/quickjs-complete-value-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 20 pass. |
| `cargo test --manifest-path experiments/quickjs-class-bridge-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 10 pass. |
| `cargo test --manifest-path experiments/quickjs-value-boundary-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 6 behavior + 2 expected compile-fail checks pass. |
| `cargo test --manifest-path experiments/quickjs-global-surface-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 19 pass. |
| `cargo test --manifest-path experiments/quickjs-dynamic-code-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 14 pass; 80 denial routes preserved. |
| `cargo test --manifest-path experiments/javascript-static-analysis-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 24 pass. |
| `cargo test --manifest-path experiments/quickjs-module-capture-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 14 pass. |
| `cargo test --manifest-path experiments/quickjs-lifecycle-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 15 behavior + 3 expected compile-fail checks pass. |
| `cargo test --manifest-path experiments/javascript-engine-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 10 pass. |
| `cargo test --manifest-path experiments/boa-reaction-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` | 14 pass, including expected-panic control. |
| `cargo test --manifest-path runtime/Cargo.toml --locked` | 53 pass: 18 library, 2 harness, 8 dispatch, 25 loader. |
| `cargo test --manifest-path runtime/Cargo.toml --locked --test declarative_conformance -- --nocapture` | 2 harness tests pass again; 34/34 authored cases print PASS. |
| `cargo build --manifest-path runtime/Cargo.toml --locked` | Pass. |
| `cargo fmt --manifest-path runtime/Cargo.toml --check` | Pass. |
| `cargo clippy --manifest-path runtime/Cargo.toml --locked --all-targets -- -D warnings` | Pass. |
| `.venv/bin/python -m unittest discover -s conformance -p 'test_*.py' -v` | 6 pass. |

Prior Phase 3 total: **151** tests/checks, unchanged. Their ten experiment commands use the established external 60-second POSIX process-group guard. All finish without watchdog intervention. The unchanged existing runner `/tmp/universal-source-complete-value-validation.py` executes the regression commands above, plus complete-value fmt/strict Clippy (both pass); its 18 command results are all exit 0. Captured summary: `/tmp/universal-source-resource-regression.log`. These temporary paths are execution records, not required new project-wide tooling. Reproducible new-crate commands and guards are committed in the experiment README.

Final compiler observations are in `/tmp/universal-source-resource-regexp.log` and durably summarized in the review: 18 / 116 / 1,638 ms for 1,000 / 4,000 / 16,000 references, then the external five-second kill at 64,000. The committed runner requires ENTER before interpreting a timeout and rejects unexpected callback output. Its exit 2 remains a failed engine bound. Wall times are observations, not portable expected values.

Development corrections and guard accounting:

- An initial depth-40 Fibonacci fixture hit the deliberately small stack cap before the CPU callback threshold. It was changed to repeated depth-12 recursion to isolate CPU behavior; dedicated recursion tests retain stack-limit evidence. The initial CPU assertion failed twice before correction.
- An initial unrooted unresolved Promise-chain fixture did not guarantee live heap growth. It reached a 25-second suite guard and an 8-second isolating guard. The corrected retained graph asserts no CPU fallback fired and reaches heap rejection. No system OOM was involved.
- The first compiler sweep constructed the same pattern on the host: smaller cases returned with zero callbacks; the 64,000 case reached a five-second guard. The final preserved fixture constructs the pattern in JavaScript with permitted String operations and reproduces the same missing hook. Its own five-second guard fired once. Total observed outer kills during this task: **four**, two development heap-fixture kills and two compiler counterexample kills. None is counted as a passing engine stop.
- New tests/final compiler binary are independently compiled and strict-Clippy clean. No prior test was edited or weakened.

## Audit and delivery

The new crate uses only safe rquickjs APIs. **Zero unsafe blocks/functions/impls**, enforced in both `src/lib.rs` and `src/bin/regexp_compile.rs`; tests inherit the library prohibition. The review records public APIs, local source fingerprints, allocator/stack limitations and native scan diagnosis. No private engine memory/API access, engine patch, version upgrade or code instrumentation.

The lockfile matches the unchanged class-bridge resolution after the root-name substitution. Production `runtime/`, dependencies, specification/RFC, decisions, conformance, examples and all prior experiments remain byte-for-byte unchanged against the baseline. The existing dynamic/global bootstrap files are included by reference. RFC 0001 remains proposed; no engine or parser is selected and no ADR is created.

Thirteen changed files: four documentation files (`ARCHITECTURE.md`, `ROADMAP.md`, focused review, this completed plan) and nine experiment files (`.gitignore`, `Cargo.toml`, `Cargo.lock`, `README.md`, `probe_regexp.py`, `src/lib.rs`, `src/tests.rs`, `src/regexp_compile.js`, `src/bin/regexp_compile.rs`). The active plan was moved here; `.gitkeep` files are preserved.

Local Conventional Commit subject: `test(engine): expose RegExp compilation resource blocker`. No push, tag, release or settings change. Final full diff, documentation/links/schema/JSON/whitespace, protected-path and clean-after-commit checks are recorded below.

## Final pre-commit verification

The staged result was independently re-verified before committing rather than accepted from the earlier session's record:

- Local pinned source fingerprints for `quickjs.h`, `quickjs.c`, `libregexp.h` and `libregexp.c` match the review table. Rereading `libregexp.c` confirms that an unresolved named backreference calls `re_parse_captures` (full `buf_start`..`buf_end` scan) once to count and once to emit indexes. `lre_compile` and the parser contain no `lre_check_timeout` call; the only polling sites are `lre_poll_timeout` uses in the matcher.
- The sandboxed build cannot copy vendored engine sources (`rquickjs-sys` build script, permission denied); all Cargo commands were rerun unsandboxed with the crate-local target directory, without network access or dependency changes.
- New crate: 11 tests pass under the 60-second process-group guard (0.7 s, guard not fired); fmt and strict Clippy pass; the lockfile equals the class-bridge lockfile after root-name substitution.
- Compiler reproducer rerun: 1,000 / 4,000 / 16,000 references returned in 11 / 115 / 1,664 ms with zero callbacks; 64,000 references printed `ENTER`, then reached the external five-second kill with no callback; runner exit **2**. Timings differ slightly from the earlier run; the zero-callback observation and blocker are unchanged. One further outer kill, again not counted as an engine stop.
- The existing regression runner's 18 commands all exit 0 with the counts in the validation table; the declarative harness prints 34/34 `PASS` lines; Python conformance runs 6 tests OK.
- `git diff --cached --check` passes; 104 relative links/anchors in the five changed Markdown files resolve; no JSON, schema or example file changed. `runtime/`, `spec/`, `conformance/`, `examples/`, `docs/decisions/` and every prior experiment are unchanged against the baseline. `unsafe` appears only in the two `forbid(unsafe_code)` attributes.
