# Declarative dispatch and execution harness

Status: Completed

## Scope

Implement independent ready-source static calls for the five existing operations, result validation, and returned-origin checks. Run the 34 authored cases unchanged through production loading and dispatch. Preserve loader policies, A1, and the U1/U2 deferrals.

## Work

1. Add a minimal Rust call boundary, closed input validation, exact lookup/page rules, envelopes, and explicit effective grants.
2. Validate constructed data before checking only returned URL-bearing fields; require both requested and effective authority.
3. Add a corpus execution harness with independent expected JSON comparison and focused boundary tests.
4. Run Rust/Python checks, document actual coverage and limitations, review the complete diff and links, and commit locally.

## Non-goals

No lifecycle state machine, deadlines/cancellation/scheduling/disposal, mutable source state, network consumers, JavaScript, services, adapters, platform/FFI/WASM work, normative contract changes, or publishing.

## Implemented scope and policies

- Added `LoadedSource::invoke`, a synchronous five-operation dispatcher over immutable loaded data, and separate `EffectiveGrants` host context with a host-only setup diagnostic. No portable binding or new Source API errors were defined.
- Implemented vocabulary/declaration-before-input precedence, closed inputs, nonempty IDs, exact query handling, page defaults/range, category-before-page lookup, independent detail/play calls, and closed success/failure envelopes. Unknown playable lookup precedes grants.
- Reused stored-type validation for constructed results and the loader's normalized origin formatter for returned URLs. Only typed poster/media fields are checked; both requested and effective authority are necessary. A1 remains intact and results are owned copies.
- Local result validation precedes origin denial without standardizing U2. U1 remains deferred. Trailing-dot resource origins remain distinct from dotless requests under the documented conservative D2 policy. No parser or loader acceptance rules changed.
- Added the corpus execution harness and independent JSON/envelope comparison helper. Each case uses a fresh production load, validated grants, actual dispatch, and unchanged authored expectations; setup errors fail the test. All case IDs are reported with `--nocapture`.
- Updated runtime, architecture, roadmap, contributor, root README and conformance documentation. Phase 2 stays active. No normative contradiction or fixture/specification conflict was found.

## Validation evidence

- `cargo test --manifest-path runtime/Cargo.toml --locked`: **38 passed, 0 failed, 0 ignored** (3 unit tests, 2 harness/comparator tests, 8 dispatcher integration tests, 25 loader integration tests); no doctests exist. This adds 12 tests to the existing 26, including the corpus runner.
- `cargo test --manifest-path runtime/Cargo.toml --locked --test declarative_conformance -- --nocapture`: **34/34 authored cases executed and passed**, with each ID printed; both integration tests passed.
- `.venv/bin/python -m unittest discover -s conformance -p 'test_*.py' -v`: all **6 existing groups passed**, covering manifest schema/fixtures/example and authored operation-data consistency.
- `cargo build --manifest-path runtime/Cargo.toml --locked`, `cargo fmt --manifest-path runtime/Cargo.toml --check`, and `cargo clippy --manifest-path runtime/Cargo.toml --locked --all-targets -- -D warnings`: passed.
- The local documentation audit checked 29 Markdown files, relative links/anchors, fences/tables/whitespace; parsed 48 JSON files and two fenced examples; and checked both schema documents. External link availability was not tested. No JSON data files were edited.
- Reviewed the complete diff including new source, tests, and docs; working-tree/staged whitespace checks passed. Normative specification, authored cases/expectations, source contexts, dependencies/lockfile, and loader tests remain unchanged from the loader baseline.

## Limitations and remaining Phase 2 work

This is independent ready-source execution under adequate-resource/no-cancellation preconditions, tested locally on macOS. There is no serialized scheduler, deadline/cancellation support, late-completion suppression, disposal, instance-health management, or full invocation resource accounting. The host must supply current grants and serialize calls; passing the ready-source corpus does not establish complete lifecycle conformance. Actual network access and redirect/consumer checks remain absent. Loader stable-filesystem and parser policy limitations remain documented.

Exactly one next coherent task: implement and test the remaining Phase 2 lifecycle/isolation boundary around the loader and dispatcher, retaining the existing ready-source cases and explicit U1/U2 deferrals. That work has not started here. No JavaScript, adapter, platform, FFI, or WASM work was added, and nothing was pushed, tagged, or released.
