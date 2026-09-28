# Boa retained-reaction ownership review

Date: 2026-09-28. Baseline: `c54886f4a7093de13ef911ee2559938085385183`.

## Disposition

**Outcome B — Boa public APIs are insufficient for a faithful F1 mechanism in the reviewed model.** Registration ownership is partly solved: host metadata survives on explicit `.then` callbacks and internal `await` continuations. Complete reaction/job suppression is still blocked. The executor cannot inspect that metadata before running an opaque reaction job; reactions with absent handlers have no callback record at all. Substituting a callback result is not equivalent to discarding the reaction job and violates the documented callback hook contract.

The [isolated experiment](../../experiments/boa-reaction-spike/README.md) has 14 passing tests, including counterexamples and one explicitly expected engine assertion panic. It selects no engine and creates no ADR. [RFC 0001](../../spec/rfcs/0001-javascript-execution-binding.md) remains proposed and unchanged. Phase 3 remains active. Production runtime behavior/dependencies, the previous QuickJS-NG spike and the specification are unchanged.

This concludes the focused follow-up proposed by the [initial engine evaluation](2026-09-28-javascript-engine-evaluation.md). It is not a general Boa evaluation or a proof that every possible transformation/fork is impossible. No private engine API, patch, source transformation, module loader, Source API dispatcher, service, interruption integration, structural copier or realm-hardening implementation was used. The next task is a **binding/lifecycle design decision**, not another broad engine survey or production execution slice.

## Pinned evidence and environment

Built `boa_engine = 0.22.0` from crates.io with default features disabled and an independent committed Cargo.lock. Current upstream release `v0.22` and the registry `.cargo_vcs_info.json` identify Git revision **`337a3668a0dc86dd401ea20906e782249a64a228`**. The experiment uses public APIs only. Repository source was read at that revision, not assumed from crate descriptions.

Authoritative sources:

- [Release v0.22](https://github.com/boa-dev/boa/releases/tag/v0.22) and [public HostHooks documentation](https://docs.rs/boa_engine/0.22.0/boa_engine/context/trait.HostHooks.html).
- [HostHooks source](https://github.com/boa-dev/boa/blob/337a3668a0dc86dd401ea20906e782249a64a228/core/engine/src/context/hooks.rs): `make_job_callback`, `call_job_callback`, `promise_rejection_tracker`, and their supported contracts.
- [Job API source](https://github.com/boa-dev/boa/blob/337a3668a0dc86dd401ea20906e782249a64a228/core/engine/src/job.rs): `JobCallback`, `host_defined`, `PromiseJob`, `NativeJob`, `JobExecutor::enqueue_job`/`run_jobs` and FIFO requirements.
- [Promise source](https://github.com/boa-dev/boa/blob/337a3668a0dc86dd401ea20906e782249a64a228/core/engine/src/builtins/promise/mod.rs): `ReactionRecord`, `perform_promise_then`, `create_resolving_functions`, fulfillment/rejection and `trigger_promise_reactions`, `new_promise_reaction_job`, `new_promise_resolve_thenable_job`.
- [Await opcode](https://github.com/boa-dev/boa/blob/337a3668a0dc86dd401ea20906e782249a64a228/core/engine/src/vm/opcode/await/mod.rs), [GeneratorContext continuation machinery](https://github.com/boa-dev/boa/blob/337a3668a0dc86dd401ea20906e782249a64a228/core/engine/src/builtins/generator/mod.rs), [VM Promise capability storage](https://github.com/boa-dev/boa/blob/337a3668a0dc86dd401ea20906e782249a64a228/core/engine/src/vm/mod.rs), and [public JsPromise state inspection](https://github.com/boa-dev/boa/blob/337a3668a0dc86dd401ea20906e782249a64a228/core/engine/src/object/builtins/jspromise.rs).

The compiled registry sources for the five central files were byte-compared with the pinned checkout:

| Engine source path | SHA-256 |
| --- | --- |
| `context/hooks.rs` | `38c0dbc626f55761d9d276a483aefe7e1265310727bff6627e1c408a8a97bf18` |
| `job.rs` | `fd3c5ed9d25d9621f96de0fd2d93938ee5214720597c65a62a7650a2b2df7411` |
| `builtins/promise/mod.rs` | `187f8ed892e2de68b6cd5168602fc16264b3e32e630847cba46318eb6088e1da` |
| `vm/opcode/await/mod.rs` | `d861ea7ea49f500fbe2a4a1b473f8c59b829dc253198ed02cc8f191538e39486` |
| `builtins/generator/mod.rs` | `da4fd678da37e75c1ee09a53b1eca632baf4057d02085537711733471f2a27b9` |

Local environment: macOS **27.0**, build **26A428**, **arm64**; Rust host `aarch64-apple-darwin`; rustc **1.96.0 (ac68faa20 2026-05-25)**, Cargo **1.96.0 (30a34c682 2026-05-25)**. Only this host was tested. Neither engine portability nor production readiness follows from these results.

## Experiment model

One reused Boa `Context` runs fixed in-memory scripts in two synthetic invocations. The host holds `current: Option<u64>` and a revoked-token set; A and B are Rust-only tokens. No JavaScript-visible ownership variable is used. `make_job_callback` preserves the original callback and adds its registration token to `JobCallback::host_defined`. Hook logs record registration owner, current executing invocation and whether the diagnostic probe skipped the callback.

A local `JobExecutor` holds a FIFO `VecDeque` of opaque `PromiseJob`s tagged with the current token **at enqueue time**. The harness can run one job, inspect an intrinsic `JsPromise::state`, discard a known enqueue token, or return immediately when no job exists. It asserts at most 100 jobs per diagnostic drain; an external 60-second process-group timeout bounds the test run. No asynchronous host provider, real deadline clock, operation result validation or production scheduler is implemented.

Three test modes distinguish supported observation from attempted suppression:

1. `Observe` calls the original callback and returns its actual result, as the public hook contract requires. Host registration metadata survives, but stale work executes.
2. `SkipWithUndefined` deliberately returns `Ok(undefined)` for a revoked callback without calling it. This is an explicitly nonconforming counterexample, not a proposed implementation.
3. `SkipWithError` deliberately substitutes a rejection value (`99`) to test whether throwing could abandon old work. It is also nonconforming and exposes an internal await invariant.

At synthetic terminal selection the harness revokes A and drops already queued entries tagged A without executing them. This models the RFC's proposed termination policy, not unrestricted ECMAScript queue progress: Boa documents ordinary jobs as eventually executed and FIFO. The API physically allows the host to withhold/drop queued jobs; that alone neither supplies reaction ownership nor standardizes the observable cancellation semantics. Diagnostic drains after B begins intentionally exercise stale jobs to expose these gaps, and are not RFC-compliant scheduling.

## Where metadata exists and where suppression fails

| Stage | Public surface and source evidence | Consequence |
| --- | --- | --- |
| Callback creation/registration | `make_job_callback` runs for callable fulfillment/rejection handlers, preserving the original `JsFunction` and adding host-defined data. `perform_promise_then` creates records for user `.then` and internal await handlers. | A token survives while the Promise is pending; host-only tagging does not execute the callback. It records **registration**, not when the function object was originally created. |
| Missing handler registration | `perform_promise_then` maps only present callable handlers to callback records; `retained.then()` creates empty-handler reactions. | No `make_job_callback` call, hence no host registration token. Even with a rejection callback present, the fulfillment side can be empty. Function-object maps or `.then` wrappers cannot cover this internal reaction model faithfully. |
| Retention | `ReactionRecord` stores optional handler, capability and reaction type in Promise reaction lists. It is `pub(crate)` and its fields are private. | Explicit handlers retain metadata, but the public embedding cannot inspect/revoke the whole reaction list or stamp all reactions. |
| Future job creation | Resolving functions trigger reactions only when the retained Promise settles. `new_promise_reaction_job` captures the reaction and argument in a closure. | A reaction registered under A can become a job while B is current. Ownership is not lost from the callback record; it is hidden inside the job. |
| Enqueue/dequeue | `JobExecutor` receives `Job::PromiseJob`; public `PromiseJob` exposes construction, realm and consuming `call`, not the enclosed reaction/handler/capability. | A same-realm job's owner cannot be read before execution. Enqueue-time tagging produces B for both an old A reaction and a valid B reaction on the **same Promise**. Dropping all such jobs would also suppress B. |
| Callback invocation | `call_job_callback` receives the original callback token just before `Call`. Its documented requirement is to call that callback and return its completion. | This is late enough to identify a callback, but there is no supported `DiscardReaction` completion. `Ok` means a normal result, `Err` means a throw, not abandonment. |
| After callback | Reaction job resolves/rejects its result capability; empty-handler jobs propagate directly without invoking the callback hook. | Substitution changes child Promise state, can enqueue more work, and can call source-provided species resolver functions outside this hook. |
| Await continuation | Await creates native fulfill/reject closures containing a suspended `GeneratorContext`, then calls `perform_promise_then` with **no result capability**. The closure resumes the async frame. | Both await handlers are tagged. Skipping the resume leaves async results pending. Returning an error violates the job's normal-completion assertion for a missing capability. |
| Resolving functions / thenable assimilation | Resolve/reject functions are created by internal machinery without a registration-owner hook. On resolving an object, the resolver reads `then` and creates a callback/job for callable `then` at resolution time. | A resolver saved in A and invoked with a thenable in B creates a B-tagged assimilation callback. This is a distinct ownership-policy question, not evidence that all work belongs to the Promise's creation invocation. The synchronous `then` lookup can itself execute source. |

The earliest useful complete enforcement point would be **reaction registration**, with metadata independent of the optional callback, propagated into the job and exposed before dequeue execution (or checked before any reaction effects). This does not exist in the reviewed supported surface. Intercepting a callback and setting a side channel at that point cannot undo source calls or child settlement performed by the surrounding job. Panicking to unwind an opaque job is neither a supported discard mechanism nor a healthy-context recovery policy.

## Local observations

All 14 tests passed with no ignored tests and zero doc tests. A pass in this table often means an incompatibility was reproduced.

| Test | Observation |
| --- | --- |
| `supported_observation_preserves_registration_tags_but_executes_old_then_and_await` | A makes three callback records: one `.then` and two await handlers. No job exists until B resolves the Promise. Both executing callbacks still report A, both enqueues report B, and both old bodies run (`marker = 2`). |
| `skip_then_blocks_body_but_fulfills_child_and_b_observes_it` | Diagnostic skip prevents old body (`marker = 0`) but fulfills child with undefined. B's observer runs once and sees undefined. Upstream remains fulfilled with 7; ordinary state persists from 40 to 42; no rejection notification. |
| `queued_a_jobs_can_be_dropped_before_execution_without_dropping_b_jobs` | An already queued A job is removed before execution. Its child stays pending; A body never runs; independent B job runs once. |
| `enqueue_attribution_loses_future_registration_owner` | A's pending reaction is enqueued with B's current token. Removing enqueue-tagged A entries removes nothing; old body runs once. The callback hook still sees registration A. |
| `same_retained_promise_can_hold_a_and_b_owned_reactions` | Both jobs enqueue under B. Callback metadata distinguishes A/B; diagnostic skip blocks A's body, B adds 7 and returns 42. Old child nevertheless becomes fulfilled undefined. Promise identity or enqueue token alone cannot separate them. |
| `skipped_promise_chain_settles_both_children_without_running_old_bodies` | Both chain registrations retain A; both bodies are skipped. Both child and tail become fulfilled undefined instead of remaining pending; each reaction job still runs. |
| `skipped_nested_await_leaves_inner_and_outer_pending_without_finally` | Inner and outer register four A callback records. Skipping inner's stale resume leaves **both** async results pending, old markers/finally counter zero. B observers on them remain waiting; an unrelated B job runs once. No rejection notification. |
| `terminal_promise_stops_queued_work_but_not_future_reactions` | One A job settles terminal Promise to 42 and queues later work plus a pending reaction. Direct state inspection permits immediate terminal selection; queued `+10` work is dropped. B later resolves retained Promise and old `+100` work still executes. |
| `pending_return_keeps_host_in_control_for_synthetic_cancel_and_deadline` | In each controlled cancellation/deadline scenario the returned Promise stays pending, an empty step immediately returns false, host records terminal state and revokes A. No blocking wait, source timer or real CPU interruption is involved. |
| `absent_handlers_have_no_registration_tag_and_propagate_under_b` | `retained.then()` triggers zero make/call hooks. When B resolves retained to 7, its child fulfills with 7 through an opaque B-enqueued job. Callback-based suppression misses it entirely. |
| `error_substitution_rejects_child_and_reports_unhandled_rejection` | Old body stays unexecuted, child rejects with substituted 99, host receives one Reject notification. A B-owned catch runs and observes 99. This is observable rejection, not discard. |
| `error_substitution_for_await_violates_engine_invariant` | Returning Err for stale await handler triggers the pinned engine's `handlerResult is not an abrupt completion` assertion. This one test explicitly expects that panic; its context is dropped on unwind and never reused. No supported recovery is claimed. |
| `skipped_callback_still_calls_source_species_resolver` | Despite skipping the old handler (`marker = 0`), the reaction job calls an A-created, source-provided species resolver once. This proves callback omission alone cannot guarantee no source execution. |
| `thenable_resolver_hook_tags_assimilation_at_resolution_not_resolver_creation` | Resolver and thenable retained from A; invoking resolver in B creates a B-tagged callback/job and calls thenable body once. Registration/assimilation time differs from object creation time. |

## Observable Promise semantics

| Attempt | Child / upstream consequence | Source execution and observability |
| --- | --- | --- |
| Drop a known queued reaction job | Ordinary child stays pending. Upstream settlement already occurred and is not undone. Retained child object is not deleted. | No reaction callback or JS cleanup runs from dropping this job. B can attach a reaction that remains pending, or distinguish this state with controlled Promise scheduling. There is no automatic rejection. |
| Substitute undefined for `.then` callback | Ordinary child fulfills undefined; chained children also settle as jobs run. Upstream stays settled. | B observes fulfilled undefined; custom species resolve can run source synchronously even though the old handler was skipped. This is not equivalent to job discard. |
| Substitute undefined for await handler | Suspended async frame does not resume; async function's Promise stays pending, as does a nested outer await in the test. | No tested finally block executes. This differs from `.then` child fulfillment under the same substitution and gives no uniform Promise-graph rule. |
| Substitute error for `.then` callback | Child rejects with the invented reason; unhandled rejection tracking fires if no handler yet. | Later B catch observes that reason. Calling this cancellation would introduce a source-visible result not specified by the RFC. |
| Substitute error for await handler | No result capability exists for this reaction; engine invariant fails. | Expected test panic only; no reusable-context solution. |
| Ignore empty-handler reactions | Default value/throw propagation continues. | No callback hook is available to suppress propagation; downstream B observers can see the settled child. |

No test uses FinalizationRegistry or measures garbage collection. Dropping Rust job handles releases roots; actual GC/native resource reclamation timing is not a deterministic source cleanup promise. The async `finally` observation is source control flow, not proof about GC finalizers. The proposed RFC excludes GC/finalization observability anyway.

The RFC already deliberately abandons queued work at terminal selection, departing from ordinary eventual job progress. Its retained-state model therefore needs an explicit portable consequence for abandoned reaction capabilities: for example, leaving a child pending rather than inventing fulfillment/rejection. That example is **a review question, not an amendment adopted here**. It must also define what B may do with a retained pending child or upstream Promise, and distinguish A's reaction from B's new reaction on the same object. A single Promise-level owner flag cannot express both registrations. Inert promises may originate during initialization; their permitted later use also needs to fit any chosen rule.

## Missing mechanism and next design decision

The public API faithfully preserves tokens for present callbacks and physically permits custom queue control, but does not provide complete pre-execution ownership and revocation. Retrofitting it would require a **new upstream hook or engine patch** to stamp every reaction (including empty handlers), carry the token through reaction/thenable/async machinery, and discard a revoked reaction before invoking its handler **or capability resolve/reject**. The capability consequence and legal future observations must be specified first. Such machinery lies behind private reaction fields and opaque closures today; this experiment neither accesses nor patches it. A callback hook that returns a never-settling Promise would still perform result assimilation/capability calls, still miss empty handlers, and still violate the hook's Call contract; it is not established as a solution.

This review does not conclude Outcome C: two focused experiments show important incompatibilities, but do not quantify whether an engine extension's maintenance cost is disproportionate to the benefit. Do not force an RFC weakening or engine choice from that uncertainty. The next coherent task is to decide the binding/lifecycle semantics, with concrete A/B expected outcomes, before any further engine implementation.

Options for that decision (not adopted here):

| Model | State and author impact | Portability, safety and conformance tradeoff |
| --- | --- | --- |
| Current persistent instance plus invocation-owned reactions | Preserves healthy mutable state and allows ordinary Promise helpers, but authors must await needed work. Source API call-order independence still applies. | Requires complete registration tracking and pre-effect discard; must specify retained-child outcomes. Host services revoke per invocation; executing cancellation still disposes. Engine extensions may be needed. |
| Fresh realm per invocation | Loses all transient state/caches and changes RFC initialization frequency; IDs must still work without prior calls. Simple author lifecycle but repeated initialization. | Removes cross-call Promise graphs and simplifies job/service isolation; still requires CPU interruption, safe disposal and bounded initialization. Deterministic fresh-state fixtures are simpler. Requires explicit RFC lifecycle change. |
| Dispose after any invocation leaving unresolved async work | Preserves state only on provably clean calls, making persistence conditional. Inert pending promises complicate the criterion. | Detecting all unresolved/internal work needs hooks and is not the same as empty queue. Cancellation/service isolation improves on disposal, but deterministic liveness criteria and reuse rules must be specified. |
| Prohibit retention of unresolved Promise state | Keeps ordinary synchronous state; authors must avoid pending chains across completion, including internal await state. | A source prohibition alone is not enforcement: reliably detecting reachability/hidden continuations can need deep engine support. Define failure behavior and service revocation; do not silently trust source compliance. |
| Persistent instance, revoke service authority only and permit old pure JS reactions | Keeps ordinary ECMAScript Promise progress and caches, but old source may mutate later-call state and consume B's resources. Call-order independence remains an author obligation and is harder to test. | More conventional engines could schedule it, but it explicitly relaxes the current no-background-work rule. Stale service handles must still fail; cancellation and deterministic cross-call ordering need a new contract. Not a workaround inside current RFC code. |

The decision should compare these models against actual needed state persistence and write expected results for direct/await chains, missing handlers, species functions, B reactions on retained objects and termination. Preserve Model B envelopes, JSON-only transfer, module restrictions, explicit capabilities, code-generation ban and A1/U1/U2/P1/D1/D2. No normative file changed here and no alternative is accepted. Interruption, Proxy/copying, dynamic compilation, globals, module preflight, resources and platforms remain separate engine-selection blockers regardless of the eventual F1 decision.

Full checks and delivery scope are in the [completed plan](../plans/completed/2026-09-28-boa-reaction-ownership.md).
