# Static declarative loader

Status: Completed

## Scope

Choose the reference implementation language in ADR 0002, then implement only a bounded loader for the existing static declarative contract at closure commit `e0613c5fe1e5d36f690e34c16893e45847217674`. Preserve schema/prose authority, A1 load/result separation, and D1/D2 implementation-policy boundaries.

## Work

1. Evaluate Rust and alternatives; record the scoped decision without selecting a source format or binding.
2. Add a small library with strict JSON and authoritative schema validation, semantic origin checks, contained file loading, full static shapes/references, finite load budgets, and host-facing diagnostics.
3. Add focused loader tests, including existing data fixtures, invalid input, containment/symlinks, resource boundaries, and ungranted stored URLs.
4. Run old conformance tests and Rust build/test/format/lint checks; review links and the complete diff.
5. Update current-state documentation, record limitations/results here, move the plan to completed, and make one local Conventional Commit.

## Non-goals

No operation dispatch or operation-case execution, invocation lifecycle, permission enforcement at return time, network access, host-service bindings, adapters, platform integration, WASM, specification changes, push, tags, or releases.

## Results

- Accepted [ADR 0002](../../decisions/0002-reference-runtime-language.md): Rust for this reference implementation only, after comparing Python, TypeScript/JavaScript, Go, and C/C++. The standard and source formats are unchanged.
- Added one unpublished library crate under `runtime/`, a committed Cargo lockfile, strict JSON decoding, direct authoritative schema validation, separate origin semantics, contained file loading, full static entry validation, immutable snapshots, and distinguishable host-facing diagnostic categories.
- Preserved A1: valid stored poster/media URLs need not be requested/granted at load time. `UnavailableService` is reserved and unreachable for schema-valid declarative sources; invalid service declarations are schema failures. No service implementation was added.
- Documented hard ceilings of 64 KiB manifest, 8 MiB entry, 8 MiB + 64 KiB combined source input, and 64 nested JSON containers; hosts may lower them. Package input counts only consumed manifest/entry data, not unrelated directory contents.
- Documented diagnostic precedence, parser policy, stable-filesystem precondition, and no portable permission or parsing-model changes. No normative contradiction was found. The authoritative schema already rejects trailing-dot manifest origins; resource URL parsing is a separate check.
- Updated English/Chinese introductions, contribution commands, architecture, charter/principles language-choice facts, ADR index, and roadmap. Historical ADR/review records remain intact. Phase 2 stays active.

## Validation evidence

Using Rust/Cargo 1.96.0 on the local macOS host:

- `cargo build --manifest-path runtime/Cargo.toml --locked`: passed.
- `cargo test --manifest-path runtime/Cargo.toml --locked`: 1 schema/corpus unit test and 25 loader integration tests passed; 0 failed, 0 ignored. No doctests exist. Tests cover both source contexts, all 41 schema fixtures, malformed/duplicate JSON, semantic failures, references, headers/URLs, byte/depth limits, snapshots, and Unix symlink escapes.
- `cargo fmt --manifest-path runtime/Cargo.toml --check`: passed.
- `cargo clippy --manifest-path runtime/Cargo.toml --locked --all-targets -- -D warnings`: passed after fixing four style warnings.
- `.venv/bin/python -m unittest discover -s conformance -p 'test_*.py' -v`: all 6 existing groups passed, including 41 manifest fixtures, minimal example, and structural consistency of 34 operation cases. None of those operations was executed.
- Documentation audit parsed 48 JSON files and 2 fenced JSON examples, checked both schemas, and checked all 28 Markdown files for local links/anchors, fences/tables, and whitespace. External documentation links were syntax-checked; availability was not part of that audit.
- Reviewed the complete diff, including new source/tests/docs and dependency lockfile; working-tree and staged whitespace checks passed. Specification, example, and existing conformance files are unchanged from the closure baseline. No dispatch, network consumer, binding, adapter, platform, or WASM work is present.

## Limitations and next unit

The loader assumes a host-controlled filesystem tree stable during loading; canonicalize/open is not a concurrent hostile-writer sandbox. Input/depth limits are not an allocator quota or filesystem deadline. Only local macOS execution and Unix symlink behavior were tested; other platforms are unproven. Snapshot inspection is not a permission grant or invocable lifecycle state. Full invocation isolation, deadlines/cancellation, disposition of U1/U2, and parser interoperability still need their scoped follow-up work.

Exactly one recommended next task: implement static operation dispatch plus an execution harness for the existing 34 declarative cases, including result-time permission checks, while retaining the documented deferrals. This task has not begun. No push, tag, release, or repository-setting change was performed.
