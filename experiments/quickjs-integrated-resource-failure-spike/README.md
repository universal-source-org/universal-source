# Integrated QuickJS resource/failure spike

This is a non-production composition experiment for the pinned QuickJS-NG
source. It combines the already-demonstrated local async-termination patch,
RegExp admission, hardening, RFC §6 value transfer, public resource limits and
fresh-runtime retirement through one Rust binding path.

Run from the repository root:

```sh
python3 experiments/quickjs-integrated-resource-failure-spike/run.py --prepare
python3 experiments/quickjs-integrated-resource-failure-spike/run.py
python3 experiments/quickjs-integrated-resource-failure-spike/run.py --baseline
```

The driver copies the pinned source into the ignored `build/sys` directory and
never changes the committed engine or any production dependency. The patched
run passes 22 integrated tests (15 original plus 7 public-API follow-up tests);
the baseline run is a negative control for the Promise stop-swallow. Existing
experiment suites and the preserved large
RegExp compiler reproducer remain separate evidence. The result is Outcome B:
allocator rejection can still be caught by source, and an engine heap failure
can occur before the trusted allocator latch. See the [review](../../docs/reviews/2026-09-30-quickjs-integrated-resource-failure-spike.md).

The Rust ASan/UBSan link is unavailable on the local macOS arm64 toolchain;
sanitizer evidence for the earlier C patch harness is unchanged. This crate is
not a production runtime, parser, loader, engine selection or fork.

## Public-API failure-classification follow-up

**Outcome B — composition gate remains open.** The focused
[review](../../docs/reviews/2026-09-30-quickjs-public-api-failure-classification.md)
records the public API audit and evidence matrix. The same commands above run
all diagnostics; `src/failure_classification.rs` is the new test group. No engine,
patch, dependency or original test was changed.

The optional `call_heap_limit` test configuration changes the public engine
ceiling only at call entry. `None` preserves the original path; `Some(0)` disables
that ceiling while the finite 4 MiB allocator budget remains active. Paired
requests distinguish heap rejection before the allocator from trusted allocator
refusal. Both allow same-job catch/finally continuation through direct, Promise,
await, thenable and async-generator paths. Only the allocator latch suppresses
capability action and publication; it also stops later job pumping. A later
interrupt cannot undo source already executed. Ordinary/forged/stack controls
remain catchable. Public Error marking after host return is too late, and a real
OOM null fallback cannot be marked uncatchable.

All integrated cases retire without a final drain, reject late delivery, release
tracked allocations and check a healthy independent generation. The isolated
public marking probe uses public C APIs only, at host control boundaries.

Direct regression and preserved negative-control commands:

```sh
python3 experiments/quickjs-async-termination-fix-spike/build_and_run.py
cargo test --manifest-path experiments/quickjs-regexp-admission-impl-spike/Cargo.toml --offline --locked promise_machinery_swallows_gate_and_deadline_stops_in_process -- --nocapture
cargo test --manifest-path experiments/quickjs-resource-spike/Cargo.toml --offline --locked
cargo build --manifest-path experiments/quickjs-resource-spike/Cargo.toml --offline --locked --bin regexp_compile
python3 experiments/quickjs-resource-spike/probe_regexp.py
```

The last command intentionally retains the original non-polling compiler
counterexample: the large case exits 2 after an external safety kill, not an
engine stop or successful teardown. The original Promise/async counterexample
still swallows stops on the unmodified pin. No historical evidence is redefined.
