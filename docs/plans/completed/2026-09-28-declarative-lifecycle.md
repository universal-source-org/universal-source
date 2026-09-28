# Declarative lifecycle implementation

Status: completed. Baseline: `e2a3f84f8719cd2032dbfc83a509e0b79465edea`.

## Scope

Wrap the validated immutable snapshot and existing five-operation dispatcher in an instance-local lifecycle boundary. Implement serialization, independent-instance concurrency, finite deadlines, caller cancellation, exactly-one delivery, disposal/quiescence, bounded invocation input/output, and unsafe-context disposal. Use standard-library synchronization; preserve the specification, authored corpus, grants, and dispatch semantics.

U1 remains a host-facing misuse outcome, U2 remains local precedence only, and D1/D2/P1 deferrals remain open. No network, JavaScript, platform bindings, services, or generic engine abstraction.

## Steps

1. Add instance state and synchronous scheduling with cooperative cancellation/deadline checkpoints and documented policies.
2. Add deterministic synchronization tests and route the unchanged 34 cases through managed instances.
3. Update current-state documentation and assess Phase 2 gaps without closing the phase.
4. Run Rust/Python suites, the case harness, build/fmt/Clippy, documentation checks and complete diff/whitespace review; move this record to completed and commit locally.

## Validation focus

Prove same-instance exclusion, separate-instance progress, permanent disposal, cancellation/timeout and completion races, discarded late results, bounded response behavior, and panic-triggered disposal without timing-only races. Record cooperative latency and resource-accounting limitations honestly.

## Results and policies

Implemented `Instance`, `CallLimits`, `Cancellation`, and `LifecycleError` using only standard-library synchronization. The instance consumes a loaded snapshot; its mutable state is private and instance-local. The earlier public snapshot dispatch is internal now. Both the authored-case harness and focused operation tests use the managed path without changing expected outcomes.

One mutex/condition-variable execution gate serializes each instance; separate instances progress independently. No worker threads or unbounded runtime queue are created. Scheduling is not FIFO. Admitted callers retain borrowed inputs on host threads; cancellation is polled every 10 ms while queued and checked at execution/publication checkpoints. Test-only gates and a clock override establish deterministic ordering; one test also exercises the actual monotonic queue deadline.

Default deadline: 5 seconds including queue time, configurable from a positive Duration through 60 seconds. Default/hard ceilings: 65,536 compact input JSON bytes, 8,389,632 compact result-envelope bytes, 64 outstanding calls. Hosts may lower byte/admission limits to nonzero values. Input nesting is capped at 64 containers. Serialization uses a counting sink; fixed fallback diagnostics below 128 bytes are exempt so exhaustion can be reported. The loader bounds source/result construction; no precise allocator quota is claimed.

Ready/disposed is permanent instance state. Calls after disposal receive the host-only `LifecycleError::Disposed`; admitted interrupted calls receive `CANCELLED` or `TIMEOUT`. Publication chooses observed disposal/cancellation before timeout, then computed completion, under the state/token locks. The return value is the single delivery channel; later outcomes are discarded. Disposal waits for admitted work and discarded values to quiesce before returning; repeated disposal is idempotent. No cleanup callback or storage behavior is added.

Cancellation, timeout and checked quota failures keep the immutable static context healthy. An execution unwind marks it unsafe and permanently disposed, returns confidential `SOURCE_ERROR` unless interruption wins, and cancels waiters. No automatic retries occur. Abort/OOM process failures remain outside recovery.

## Validation evidence

- `cargo test --manifest-path runtime/Cargo.toml --locked`: 53 passed, 0 failed/ignored (18 library tests including 15 new lifecycle tests; 2 harness/comparator tests; 8 dispatcher integration tests; 25 unchanged loader integration tests). No doctests.
- `cargo test --manifest-path runtime/Cargo.toml --locked --test declarative_conformance -- --nocapture`: 34/34 unchanged authored cases pass individually; both integration tests pass.
- `cargo build --manifest-path runtime/Cargo.toml --locked`: passed.
- `cargo fmt --manifest-path runtime/Cargo.toml --check`: passed.
- `cargo clippy --manifest-path runtime/Cargo.toml --locked --all-targets -- -D warnings`: passed.
- `.venv/bin/python -m unittest discover -s conformance -p 'test_*.py' -v`: all six groups pass, covering manifest fixtures/example and declarative fixture consistency.
- One-off documentation/JSON scan (`.venv/bin/python /tmp/universal-source-phase-1-closure-check.py`): 30 Markdown files, 266 local links, 32 anchors, 48 JSON files, two embedded JSON blocks, two schemas and four external-link syntax checks pass. External availability was not tested.
- Complete code/test/documentation diff reviewed; `git diff --check` and staged whitespace checks pass. Specification, example, corpus expectations/contexts, Cargo dependencies/lockfile, and loader tests are unchanged from the baseline.

## Limitations and remaining work

Cooperative checkpoints are not preemptive termination or a hard real-time bound: cancellation, timeout delivery and disposal may wait for bounded cloning/validation/parsing work and OS scheduling. Input ownership, retained results, total instances/threads and process memory belong to the embedding host. The result cap is enforced before delivery, not as a precise preallocation quota. Hosts still provide a stable filesystem during loading. No networking, consumers, JavaScript, adapters, platform integration, FFI or WASM was added; no dependency was added.

No specification contradiction was found. U1/U2/P1/D1/D2 remain deferred; local policies do not close them. Phase 2 stays active: implementation/test evidence now covers loading, dispatch, static lifecycle/isolation, limits and the current corpus, but no joint closure audit has assessed that evidence against every exit criterion. The single recommended next task is that scoped Phase 2 closure audit, including host obligations and any remaining gaps. It has not begun here.
