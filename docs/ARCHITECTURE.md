# Current architecture

This is a snapshot of the repository's current state, not a proposed implementation design. The [specification](../spec/README.md) remains authoritative for contract details; the [roadmap](ROADMAP.md) owns sequencing and future exit criteria.

## Present artifacts and implementation status

| Area | Current status | Evidence and limits |
| --- | --- | --- |
| Repository context | Present documentation | Charter, principles, roadmap, decisions, and contribution workflow in this monorepo. |
| Source contract | Specified as an experimental draft | [Source API](../spec/source-api.md) defines exactly five operations, results, errors, and static declarative dispatch. The Rust static profile executes independent calls through lifecycle-managed instances. |
| Manifest | Schema, prose, and validation tooling present | [Schema](../spec/schema/manifest.schema.json) covers structure; the [conformance workflow](../conformance/README.md) checks it and manifest fixtures with a real Draft 2020-12 validator. The Rust loader separately performs semantic manifest and static-entry validation. |
| Host boundary | Partially specified | [Host API](../spec/host-api.md) defines service responsibilities and permission boundaries. Exact service signatures and algorithm profiles remain open. |
| Lifecycle and versioning | Specified as draft contracts | [Lifecycle](../spec/lifecycle.md) and [compatibility](../spec/compatibility.md) define expected behavior; loading and static invocation lifecycle are implemented with documented host policies; versioning remains the specified draft contract. |
| Declarative example | Static data present | [Minimal source](../examples/json/minimal/README.md) demonstrates all five operations with a placeholder media URL. The [operation corpus](../conformance/declarative/README.md) references it directly; all 34 ready-source cases execute through the Rust harness. |
| Reference runtime | Phase 2 complete for static execution; Rust loader, dispatcher and lifecycle implemented | [Runtime and tests](../runtime/README.md) load sources, dispatch five operations, check returned origins, and enforce instance-local lifecycle/limits. The execution harness runs 34 cases; no platform binding exists. |
| JavaScript execution | Binding RFC and fresh-realm lifecycle revision internally reviewed; no engine selection; production implementation absent | [RFC 0001](../spec/rfcs/0001-javascript-execution-binding.md) proposes modules, full-envelope returns, lifecycle, value/error transfer and explicit service injection. The [F1 decision](reviews/2026-09-28-javascript-lifecycle-decision.md) proposes a fresh realm per invocation, with reusable logical snapshots but no cross-call mutable JavaScript state. It is not accepted; manifest recognition still does not imply execution support. |
| Engine feasibility experiments | Non-production, separate Rust crates | [QuickJS-NG probes](../experiments/javascript-engine-spike/README.md) and [Boa ownership probes](../experiments/boa-reaction-spike/README.md) reproduce F1 gaps. The [initial evaluation](reviews/2026-09-28-javascript-engine-evaluation.md) selects no engine; the [Boa review](reviews/2026-09-28-boa-reaction-ownership.md) finds public hooks insufficient for faithful reaction suppression. |
| Legacy adapters | Planned, not implemented | The compatibility document defines their boundary; no importer or adapter exists. |
| Platform integrations | Planned proof, not implemented | No Android, Apple, desktop, or TV runtime integration is implemented. |
| WASM | Exploratory only | No engine value, ABI, module loader, or WASM runtime architecture is defined or implemented. |

The implemented runtime is a **static declarative loader, dispatcher and instance lifecycle**, selected under [ADR 0002](decisions/0002-reference-runtime-language.md). Independent Python manifest/fixture checks remain development tooling. Recognizing an engine name in a manifest does not mean the engine runs; JavaScript execution does not exist.

## Contract boundaries that exist in the draft

A source consists of a manifest and entry file. The manifest identifies its exact specification version, engine, operations, required host services, and requested permissions. The runtime is responsible for validation, isolation, invocation, and result validation on behalf of a host. The host grants authority and consumes results. Loading and static validation now have an implementation. Invocation, returned-origin grant checks, serialization, cooperative cancellation/deadlines, and quiescent disposal now exist. Result consumers remain unimplemented; exact allocator accounting and preemptive execution are not claimed.

The initial operation vocabulary is `home`, `category`, `search`, `detail`, and `play`; individual sources may declare a subset under the existing manifest rules. The declarative profile describes static JSON lookups. It does not include a network scraping language. JavaScript is the other initial engine target; its proposed binding is in RFC 0001, pending feasibility evidence and explicit normative adoption.

The host-service vocabulary covers HTTP, cookies, storage, HTML parsing, JSON, crypto, URL utilities, and logging. Required services and requested permissions are distinct. Returned media and poster URLs remain subject to the specified network boundary. This summary does not change the normative rules.

## Implemented loader boundary

A single Rust library crate under `runtime/` reads a host-provided root into an immutable validated snapshot. It uses the authoritative manifest schema directly, strict duplicate-rejecting JSON decoding, separate origin checks, resolved file containment, and complete static shapes/references. Internal modules separate loading/diagnostics, JSON decoding, entry/result validation, URL validation, static dispatch, and instance lifecycle. These are implementation modules, not portable APIs or a plugin architecture.

The [runtime guide](../runtime/README.md) records finite input/depth limits, diagnostic precedence, parser policies, and the requirement for a host-controlled filesystem tree stable during loading. Canonicalize/open is not a concurrent hostile-writer sandbox. Stored URLs do not need effective grants to be valid data. The dispatcher checks only URLs in actual returned results against both requests and current grants; no consumer is implemented. The loaded snapshot is separate from the managed ready/call/dispose boundary.

## Implemented call and lifecycle boundary

`Instance::new` consumes a validated snapshot and creates independent ready/disposed state. `Instance::invoke` accepts an operation name, JSON input, separate `EffectiveGrants`, and a cancellation token. It serializes execution per instance, applies finite queue/execution deadlines and input/result/admission budgets, and delegates declaration/input/lookup/result/origin behavior to the existing dispatcher. Calls return owned data and do not mutate the snapshot. There is no preceding-call requirement or public snapshot-dispatch bypass.

A mutex/condition-variable gate permits independent instances to progress concurrently without sharing mutable source or scheduler state. Calls run synchronously on host threads; cancellation/deadlines use checkpoints over finite static work. No runtime worker pool or async dependency exists. Disposal cancels admitted calls, waits for quiescence, and permanently rejects new calls with a host-specific `LifecycleError::Disposed`. This does not settle U1 portably. Checked limits and interruption leave the immutable context reusable; an execution panic disposes it as unsafe. The [runtime policies](../runtime/README.md#lifecycle-and-invocation-policy) specify latency, race ordering, budgets, and the lack of hard real-time or precise process-memory guarantees.

The [Rust harness](../runtime/tests/declarative_conformance.rs) freshly loads each source and creates a lifecycle-managed instance for every authored case, then checks the actual envelope against independent expected data. All 34 cases pass under their documented preconditions. Separate deterministic lifecycle tests cover scheduling, races, disposal and isolation. These do not prove consumer conformance or cross-platform support. U1/U2 remain deferred; local structure-before-permission result checks and conservative trailing-dot origin comparison are implementation policies documented in the [runtime guide](../runtime/README.md).

## Repository layout

| Path | Current role |
| --- | --- |
| `spec/` | Draft contracts, manifest schema, RFC template and proposed JavaScript binding RFC. |
| `examples/json/minimal/` | The one existing declarative example. |
| `runtime/` | One Rust library crate, Cargo manifest/lockfile, loader/dispatcher/lifecycle modules, focused tests, execution harness, and policy/build documentation. |
| `experiments/` | Independent unpublished QuickJS-NG primitive and Boa reaction-ownership crates with separate lockfiles; neither is a production runtime dependency or JavaScript conformance implementation. |
| `conformance/` | Manifest fixtures and validation, declarative input/expected-result cases, fixture-consistency tests, and local command documentation. The Rust execution harness consumes the unchanged 34-case corpus. |
| `docs/` | Durable project context, decisions, task plans, and scoped contract reviews. |

Local scaffolding may contain empty runtime, language-binding, or conformance directories. Git does not preserve empty directories by itself; their names are neither implementations nor decisions to adopt a language or ABI. The plan directories have explicit `.gitkeep` files so their workflow locations survive cloning.

## Decision boundary

[ADR 0001](decisions/0001-initial-architecture.md) records the monorepo, platform-neutral contract, initial engines, and adapter boundary. [ADR 0002](decisions/0002-reference-runtime-language.md) selects Rust for the reference implementation; Rust remains outside the standard and source format. WASM has no current architecture beyond being deferred research. The loader and its dependencies are documented; further runtime structure and platform bindings still need evidence and scoped decisions.

The [declarative contract review](reviews/2026-09-28-declarative-contract-review.md) records example agreement and the Phase 3 binding handoff. The [Phase 1 closure review](reviews/2026-09-28-phase-1-closure-review.md) completes the joint contract audit, identifies the reviewed Git baseline, and explicitly defers non-ready reporting, combined result-error precedence, dynamic-origin expansion, and remaining host-policy/parser questions. The [Phase 2 closure review](reviews/2026-09-28-phase-2-closure-review.md) accepts loading, dispatch and static lifecycle enforcement against all static-profile exit criteria, with explicit host policies and evidence limits. Phases 1 and 2 are complete; Phase 3 is active for JavaScript binding design. JavaScript execution and network consumers remain unimplemented. Closure does not establish complete v0.1 conformance or a stable release. These reviews do not amend the contract.

The [RFC self-review](../spec/rfcs/0001-javascript-execution-binding.md#internal-review-and-decision) and [completed design plan](plans/completed/2026-09-28-javascript-binding-rfc.md) record the first Phase 3 unit. No engine, service signatures or new portable types were adopted. The [engine evaluation](reviews/2026-09-28-javascript-engine-evaluation.md) and [completed feasibility plan](plans/completed/2026-09-28-javascript-engine-evaluation.md) record the second unit: no engine selected and no engine ADR. A QuickJS-NG spike reproduces retained Promise reactions crossing the proposed invocation boundary. The [focused Boa review](reviews/2026-09-28-boa-reaction-ownership.md) now records Outcome B: callback tokens survive, but opaque jobs, missing-handler reactions and observable child settlement prevent faithful suppression through the reviewed public hooks. The subsequent [lifecycle decision](reviews/2026-09-28-javascript-lifecycle-decision.md) resolves F1 at the proposed-contract level by choosing mandatory fresh realms (lifecycle Model B), whole-graph retirement and independent host capability revocation. This removes the selective reaction-revocation requirement; it does not invalidate the preserved counterexamples or prove the revised binding implemented. The [completed plan](plans/completed/2026-09-28-javascript-lifecycle-decision.md) records documentation and unchanged-suite validation. Next is a focused review/proof of fresh-realm lifetime and initialization through one realistic embedding path before production execution; engine selection, RFC acceptance, conformance and services remain separate work.
