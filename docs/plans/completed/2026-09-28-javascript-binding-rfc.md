# JavaScript execution binding design unit

Status: completed; RFC drafted and internally reviewed, not accepted or implemented. Baseline: `e3876c0aa139ff8f518b24436d6b827e47ac72dd`; clean working tree, no other active plan.

Draft and internally review one proposed RFC under `spec/rfcs/`. Define portable modules/exports, completion, initialization, values/errors, invocation-scoped capability injection, authority and resource boundaries. Choose one return model and analyze draft compatibility. Keep the five-operation Source API authoritative and preserve A1/U1/U2/P1/D1/D2 boundaries.

1. Read project guidance, specification/schema, ADRs, closure evidence, runtime and conformance guides.
2. Draft explicit binding decisions, alternatives, small examples and a future deterministic conformance matrix. No engine selection, implementation, callable service SDK, new portable types or conformance cases.
3. Self-review against each governing document and deferred issue; record findings and resolutions in the RFC. Update only necessary discovery/status links.
4. Rerun all six requested Rust/Python commands; check repository JSON/schema, Markdown, local links/anchors, whitespace and formatting. External links receive the established syntax-only check.
5. Review the complete diff, move this plan to completed with exact evidence and gaps, and make one local Conventional Commit. Do not push, tag or release.

The RFC remains proposed, not an amendment to normative prose. No ADR is planned: engine selection is the next separate design unit.

## Results and self-review

Created [RFC 0001](../../../spec/rfcs/0001-javascript-execution-binding.md) with a contained acyclic ES module model, direct named operation exports, full Source API envelope returns (Model B), synchronous evaluation-only initialization, Promise/job lifetime, structural JSON-only transfer, classified error mapping, invocation-scoped capability injection, restricted globals and disabled dynamic code generation. Its internal review records decisions and feasibility gaps against the charter, principles, architecture, Source/Host API, lifecycle, compatibility, ADRs and Phase 2 closure. No blocking normative contradiction was found. A1 remains intact; U1/U2/P1/D1/D2 remain deferred.

Model B avoids an additional author-facing expected-error throw/helper API; the runtime retains result validation, authority enforcement and terminal publication. Adoption would complete the unfinished experimental v0.1 JavaScript semantics without an automatic version bump. No normative adoption occurs here. The future conformance matrix covers loading/modules/exports, sync/async outcomes, values/errors, initialization, capabilities/permissions, cancellation/deadlines/late work, state isolation, resource limits and absence of ambient/dynamic-code authority. Examples are informative and unexecuted; capability acquisition deliberately does not invent callable service signatures.

Updated only architecture, roadmap and specification index for discovery and accurate status. No ADR: engine selection remains a separate unit. No runtime code, tests, dependency/lockfile, schema, example package or authored conformance expectation changed. The complete five-file change is this plan, the RFC, `docs/ARCHITECTURE.md`, `docs/ROADMAP.md` and `spec/README.md`.

## Validation evidence

On local Darwin arm64, Rust/Cargo 1.96.0:

| Command | Exact result |
| --- | --- |
| `cargo test --manifest-path runtime/Cargo.toml --locked` | 53 passed, 0 failed/ignored: 18 library, 2 harness/comparator, 8 dispatch, 25 loader; 0 doc tests. |
| `cargo test --manifest-path runtime/Cargo.toml --locked --test declarative_conformance -- --nocapture` | 2 tests passed; all 34 authored case IDs reported PASS, 34/34. |
| `cargo build --manifest-path runtime/Cargo.toml --locked` | Exit 0. |
| `cargo fmt --manifest-path runtime/Cargo.toml --check` | Exit 0. |
| `cargo clippy --manifest-path runtime/Cargo.toml --locked --all-targets -- -D warnings` | Exit 0, no warnings. |
| `.venv/bin/python -m unittest discover -s conformance -p 'test_*.py' -v` | 6 passed, OK. |

A temporary audit script outside the repository, invoked as `.venv/bin/python /tmp/universal-source-binding-audit.py`, passed strict parsing of 48 JSON files, both Draft 2020-12 schemas/internal references, and 3 fenced JSON blocks (including schema validation of the RFC manifest). It checked 33 Markdown files, 309 local links and 36 anchors, headings/table columns/fence closure, and 5 external URL spellings. External availability was not audited. Repository text whitespace/final-newline checks covered the 101 tracked/new file paths, excluding empty/binary contents. No repository Markdown/link checker is provided. The temporary checker's first run incorrectly flagged valid multi-level headings because of regex backtracking; correcting the checker produced the passing result without changing repository heading formatting. No JavaScript example was executed.

Reviewed the entire five-file change, including the new RFC and completed plan, for scope, authority, secrets and consistency. `git diff --check` and `git diff --cached --check` passed. No unrelated changes or active-plan residue are included.

## Remaining gaps and next task

Phase 3 remains active. RFC acceptance, restricted-realm/Promise checkpoint feasibility, callable service profiles, JavaScript implementation/conformance and cross-platform enforcement evidence remain outstanding. Structural resource boundaries and cooperative cancellation do not claim exact heap quotas or hard real-time preemption. No broader host/parser policy or dynamic-origin decision was made.

Next: evaluate/select the reference JavaScript engine in a separate scoped unit, record an ADR if justified, and perform only the smallest feasibility spike needed. No full engine or service implementation is authorized by this completed unit. Delivery is one local Conventional Commit; no push, tag, release or repository-setting change.
