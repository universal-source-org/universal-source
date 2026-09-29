# RegExp compilation remedy/architecture review

Status: completed with **Outcome A — pre-admission viable at design level**. Baseline `8f6cafa67b7e53d95d54d60200acac852fd78ac1` (`test(engine): expose RegExp compilation resource blocker`); clean tree, no other active plans. Phase 3 remains active.

Scope: exactly one design/evidence review determining whether the demonstrated forward-named-backreference RegExp **compilation** checkpoint blocker can be closed while keeping RFC 0001 §§5/7/9/10 semantics intact. Candidates: (A) static literal pre-admission, (B) dynamic construction pre-admission, (C) process containment, (D) an upstream/version change, plus an engine patch. Resolve to exactly one outcome and name the smallest next implementation task. Preserve the original Outcome B.

Non-goals (unchanged restrictions): no remedy implementation, production runtime change, RFC edit, new normative RegExp restriction, QuickJS patch, dependency upgrade, second engine, engine/parser selection, ADR, module/path completion, IPC/services, push, tag or release. Pins fixed: rquickjs `=0.14.0` (`d7ef5ee…`), QuickJS-NG `0.16.2` (`0fdea21…`).

## Steps

1. Read the governing docs, RFC §§1/4/5/7/9/10, the prior static-analysis/import-attribute/dynamic-code/global-surface reviews and the resource spike. Read the pinned RegExp source: lexer literal compile, `js_regexp_constructor`, `js_compile_regexp`, `js_regexp_compile`, the `String.prototype` match/search/split/matchAll routes, species/`ctx->regexp_ctor`, and interrupt poll sites.
2. Build one tiny `experiments/quickjs-regexp-admission-spike` probe answering the pivotal questions: (a) literal compile on the non-polling path; (b) replaced-global bypass routes; (c) coercion before compile. Contain everything with the established external watchdog; never credit a kill as engine support.
3. Rerun the original blocker reproducer and the 11 resource tests. Run the new probe under containment, touched-crate fmt/strict Clippy, the regression suite, and doc/link/whitespace checks.
4. Write the review, this plan and minimal ROADMAP/ARCHITECTURE updates; commit locally; do not push.

## Result

The [review](../../reviews/2026-09-29-quickjs-regexp-admission-review.md) records Outcome A. RFC 0001 requires finite published bounds and eventual interruption under a documented checkpoint policy, not mid-compile interruptibility. A host-published UTF-16 pattern length bound, applied before the non-polling compiler and paired with a trusted deadline check at every admission gate, keeps each compile a bounded slice. The pinned audit finds only O(L²) compile paths.

The [probe](../../../experiments/quickjs-regexp-admission-spike/README.md) (8 tests) confirms:

- literals compile at parse time in every position with zero callbacks, so a byte-level preflight before `Module::declare` is required;
- Boa's visitor enumerates every tested literal, including under division/RegExp ambiguity;
- a replaced global is bypassed by `String.prototype.match`/`matchAll`/`search`, `RegExp.prototype.compile` and generic-receiver `@@split`/`@@matchAll`, so the wrap set is finite and known;
- all source-observable coercion precedes native compile, so a single-coercion facade can measure without restricting inputs to primitive strings;
- global matching loops reach the interrupt hook.

New risks found:

- Boa's `regress` literal validation is superlinear on the reproducer family;
- pinned QuickJS deviates from ES2023 on `newTarget.prototype` read order and on `RegExpCreate`;
- facade uncatchability needs `JS_SetUncatchableError`, a narrow audited unsafe call, or an explicitly reviewed fallback.

Process containment is viable but not required, and is non-portable to iOS/tvOS-class hosts; it is the ranked fallback. No local upstream remedy exists, so Outcome C is not reached. An engine patch would create a fork.

Next: **RegExp admission feasibility implementation spike**, as specified in the review. The original resource Outcome B is preserved; the reproducer still exits 2.

## Validation

Host: macOS 27.0 (26A428), arm64, Rust/Cargo 1.96.0, debug test profile. The sandbox cannot build `rquickjs-sys`, so Cargo commands ran unsandboxed with `--offline`, the crate-local target directory and no dependency changes.

| Exact command | Result |
| --- | --- |
| `cargo test --manifest-path experiments/quickjs-regexp-admission-spike/Cargo.toml --locked --no-run` | Pass. |
| `cargo test --manifest-path experiments/quickjs-regexp-admission-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` (60-second process-group guard) | 8 pass; guard not fired (about 1 s). |
| `cargo fmt --manifest-path experiments/quickjs-regexp-admission-spike/Cargo.toml --check` | Pass. |
| `cargo clippy --manifest-path experiments/quickjs-regexp-admission-spike/Cargo.toml --locked --all-targets -- -D warnings` | Pass. |
| `cargo test --manifest-path experiments/quickjs-resource-spike/Cargo.toml --locked -- --test-threads=1 --nocapture` (guarded) | 11 pass, unchanged. |
| `python3 experiments/quickjs-resource-spike/probe_regexp.py` | **Exit 2** as preserved: 17 / 143 / 1,682 ms, zero callbacks; 64,000 references killed at five seconds with no callback. Not engine evidence. |
| Ten prior Phase 3 experiment `cargo test … -- --test-threads=1 --nocapture` commands (guarded) | 151 pass: complete-value 20, class-bridge 10, value-boundary 6+2, global 19, dynamic 14, static-analysis 24, module-capture 14, lifecycle 15+3, engine 10, Boa reaction 14. |
| `cargo test --manifest-path runtime/Cargo.toml --locked` | 53 pass. |
| `cargo test --manifest-path runtime/Cargo.toml --locked --test declarative_conformance -- --nocapture` | 2 pass; 34/34 `PASS`. |
| `cargo build` / `cargo fmt --check` / `cargo clippy --all-targets -- -D warnings` on `runtime/Cargo.toml` | Pass. |
| `.venv/bin/python -m unittest discover -s conformance -p 'test_*.py' -v` | 6 pass. |

The prior `/tmp` regression runner was not executed directly; the automated guard refused an unreviewed temporary script. Its identical local commands were run inline instead, with the same 60-second process-group guard on experiment suites. Captured summary: `/tmp/us-regexp-admission-regression.txt` (execution record only).

Development corrections:

- Pedantic Clippy lints were initially tried and dropped for the project's established `-D warnings` gate.
- The first matching-loop test predicted zero callbacks and **failed**: the handler fired. Source then showed per-call polling in `JS_CallInternal`. The test now asserts the observed interruption per operation.
- A 64,000-reference literal parse was added to measure preflight scaling after the 16,000 case.
- Draft source-line citations were rechecked against the pinned files and corrected before writing.
- No external kill occurred during this task apart from the preserved reproducer's own guard.

## Audit and delivery

Zero unsafe code (`forbid(unsafe_code)`); no private API, patch or dependency change. The new lockfile equals the static-analysis spike lockfile after renaming the root package. Production `runtime/`, `spec/`, `conformance/`, `examples/`, `docs/decisions/` and every prior experiment are unchanged against the baseline. RFC 0001 remains proposed; no engine or parser is selected; no ADR exists.

Changed files: `docs/ARCHITECTURE.md`, `docs/ROADMAP.md`, the review, this plan, and `experiments/quickjs-regexp-admission-spike/` (`.gitignore`, `Cargo.toml`, `Cargo.lock`, `README.md`, `src/lib.rs`, `src/tests.rs`). Local Conventional Commit only; no push, tag, release or settings change.
