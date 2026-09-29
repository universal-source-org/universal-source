# Consolidated Phase 3 engine-feasibility review

Status: completed — **Outcome A, continue QuickJS toward selection; selection not yet justified**.
Baseline: `3c3ffa4780f09e0597e7318abe2d81941457f6ac`.
Pre-existing unrelated change: `.gitignore` adds `.idea` without a final newline; preserved and excluded from this commit and text-format validation.

## Scope and work completed

Read project governance, all ADRs, the Source/Host/lifecycle/compatibility contracts and proposed RFC, the Phase 3 reviews/completed plans and relevant experiment guides. Inspected Git status and active plans before editing. Reconstructed the full evidence chain, reviewed current primary engine/Apple sources, reran the existing fix and both historical blockers, and wrote the [consolidated review](../../reviews/2026-09-29-consolidated-engine-feasibility.md).

The review includes the qualified evidence ledger, patch ownership/rebase/security responsibilities, permanent RegExp policy cost, JSC/V8/Boa comparison, security and maintenance tradeoffs, evidence reuse/reproof scope, qualitative criteria, pre-selection gates and exact next task. It corrects the blanket Apple process exclusion with documented general ExtensionFoundation/XPC processes, including tvOS 26 availability; enhanced-security TV support and end-to-end termination remain unproven. BrowserEngineKit is not a general worker route for an ordinary media application.

Conclusions: the narrow patch is a potentially manageable compatibility obligation with real engine-semantic review responsibility. RegExp admission is acceptable as an explicit permanent policy, with route and language-version audits. Neither an authorized maintained fork nor an engine decision follows. JSC's public-system control gaps, V8's unproven terminal microtask boundary/build burden and Boa's unproven Model B integration do not justify a pivot today. Boa's old persistent-reaction blocker is not a present Model B disqualification. Switching preserves contracts/oracles but requires approximately six to eight evidence units.

Minimal ARCHITECTURE/ROADMAP changes link Outcome A, correct their current Apple summaries while retaining the historical review untouched, and replace the next-task recommendation. The exact next unit is a non-production integrated resource/failure closure spike combining the existing experiment-local patched engine through Rust with actually invoked hardening, admission, §6 transfer and lifecycle retirement. It closes only the resource-composition gate if successful. Module/path containment, parser/preflight, platform embedding and patch governance must also precede selection. Production services, conformance and optional IPC implementation remain later work.

## Validation evidence

Local environment: macOS 27 arm64, Rust/Cargo 1.96, Apple clang 21. No cross-platform execution claim.

| Command/check | Result |
| --- | --- |
| `python3 experiments/quickjs-async-termination-fix-spike/build_and_run.py` | PASS. Existing driver alone generates ignored SHA-verified baseline/patched copies; complete ordinary/uncatchable/deadline matrix, ASan+UBSan, QuickJS leak-abort and fresh-runtime health pass. No tracked engine modification. |
| Locked admission implementation test suite with README's 60-second process-group backstop | 16 pass, about 4.75 s; no timeout. Five historical Promise swallow shapes still finish 1,000 iterations with about 1,002 callbacks. |
| `python3 experiments/quickjs-resource-spike/probe_regexp.py` | Expected exit 2: 6/105/1,701 ms for 1,000/4,000/16,000 references, zero callbacks. At 64,000 references, ENTER then external five-second kill. This is preserved compiler failure, not successful interruption. |
| `cargo test --manifest-path experiments/<crate>/Cargo.toml --locked` for the other twelve crates | All pass: Boa 14, primitives 10, static analysis 24, class bridge 10, complete value 20, dynamic code 14, globals 19, lifecycle 15+3, module capture 14, RegExp routes 8, resources 11, value blocker 6+2. With admission: 186 checks. Includes historical negative controls and five expected compile-fail doctests. |
| `cargo build --manifest-path runtime/Cargo.toml --locked` | Pass. |
| `cargo test --manifest-path runtime/Cargo.toml --locked` | 53 pass: 18 library, 2 harness, 8 dispatch, 25 loader; harness executes all 34 operation cases. |
| `cargo fmt --manifest-path runtime/Cargo.toml --check` | Pass. |
| `cargo clippy --manifest-path runtime/Cargo.toml --locked --all-targets -- -D warnings` | Pass. |
| `.venv/bin/python -m unittest discover -s conformance -p 'test_*.py' -v` | Six tests pass. Initial system-Python attempt failed to import installed-only-in-venv `jsonschema`; rerun used the documented existing environment. No dependency installation or change. |
| Current upstream read-only source probe | All 12 old patch anchors still match current QuickJS-NG revision `2f0aa72a6b09cf69ff06399bcf9f4c083ac1a278`. No newer engine build, upgrade or patch performed. |

Primary-source research also inspected WebKit public/private headers at `c4c7d4d56e3e663789bbcff34edd163b45eed345`, rusty_v8 documentation/microtask APIs at `2f588046bf45a7b9adeda3ed0f0e881d38e6f3f5`, current vendor API documentation and Apple availability metadata. Links and distinctions are in the review. No tiny executable probe was added to the repository.

Temporary execution logs are `/tmp/universal-source-async-review.log`, `/tmp/universal-source-admission-review.log`, `/tmp/universal-source-regexp-review.log` and `/tmp/universal-source-consolidated-regressions.log`; they are not required project artifacts. Reproducible commands remain in the unchanged experiment guides.

## Documentation and delivery audit

The established external documentation checker was adapted only to exclude the unrelated `.gitignore` formatting change. It checks strict duplicate-free JSON, schemas/internal references, fenced JSON examples, Markdown headings/tables/fences, relative links/anchors, external URL syntax and text whitespace/final newlines. Relevant external primary sources were read separately; historical external-link availability was not comprehensively retested.

The final checker passes: 51 JSON files, two schemas, three fenced JSON examples, 83 Markdown files, 568 relative links, 46 anchors, 131 external URL spellings and 240 nonempty text files (excluding `.gitignore`).

Full four-file diff reviewed for factual scope, historical preservation, dependencies, secret/generated-file inclusion and contract consistency. `git diff --check` and `git diff --cached --check` pass. Protected production/specification/dependency/ADR/experiment and historical-review paths remain unchanged. RFC 0001 stays proposed/unimplemented; no engine/parser selection or engine ADR, no maintained-fork authorization, no production JS or module/path/IPC/platform implementation. The pre-existing `.gitignore` bytes are preserved. No active-plan residue remains.

Delivery: four documentation files, local Conventional Commit `docs(engine): consolidate Phase 3 feasibility evidence`. No push, tag, release or repository settings changes. The resulting hash is reported in the task response rather than stored self-referentially here. After the commit, the only intended dirty path is the pre-existing `.gitignore`.
