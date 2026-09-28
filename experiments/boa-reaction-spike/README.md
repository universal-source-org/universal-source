# Non-production Boa reaction ownership spike

A focused follow-up to the [QuickJS-NG F1 finding](../../docs/reviews/2026-09-28-javascript-engine-evaluation.md). **Outcome B: the reviewed Boa public hooks do not supply complete RFC 0001 reaction/job suppression.** No engine is selected. This is not a production runtime, general engine abstraction or conformance suite.

The [F1 review](../../docs/reviews/2026-09-28-boa-reaction-ownership.md) records all 14 test outcomes, exact source references, ownership boundaries and child-Promise consequences. Passing tests include deliberately reproduced failures of candidate approaches. One `should_panic` test expects an engine invariant failure from an intentionally invalid await callback substitution; the context is unwound/dropped, never reused.

## Small experiment model

- A single reusable context, fixed in-memory scripts, two host-only invocation tokens A/B and a revoked-token set. JavaScript has marker/result globals but cannot read or change ownership metadata.
- `HostHooks::make_job_callback` stores the registration token in `JobCallback::host_defined`. Supported observation calls the original callback through `call_job_callback`.
- Two explicitly nonconforming diagnostic modes replace revoked callback results with undefined or an error. They measure why callback suppression is not job discard; neither is offered as a runtime implementation.
- A local FIFO executor stores opaque `PromiseJob`s, labels enqueue time, steps one job and can drop known queued entries. Tests expose why enqueue time cannot recover future reaction registration ownership.
- Direct/nested `.then` and await, queued/future work, missing handlers, terminal observation, synthetic pending cancellation/deadline, valid B work, persistent ordinary state, species resolvers and thenable assimilation are covered. No module loader, services, real timer, CPU interruption, converter or realm hardening.

## Dependency and evidence scope

Separate unpublished crate with `boa_engine = "=0.22.0"`, default features disabled, independent Cargo.lock (134 external resolved packages including target-dependent entries). No production dependency or workspace change; no direct Boa GC/derive dependency needed for the primitive `u64` token. Optional/default features are disabled to keep the experiment narrower; this does not make the full context an RFC allowlist.

The registry VCS record identifies `337a3668a0dc86dd401ea20906e782249a64a228` (release v0.22). Five core hook/job/Promise/await/continuation files were byte-checked against that revision. No engine fork/patch, private API or altered dependency source is used.

Local evidence: macOS 27.0 build 26A428 arm64 (`aarch64-apple-darwin`), rustc 1.96.0 (ac68faa20 2026-05-25), Cargo 1.96.0 (30a34c682 2026-05-25). No other-platform claim. All 14 tests passed; zero failed/ignored, zero doc tests.

## Run from the repository root

Compile first, then use the external process-group backstop. Fixed scripts contain pending Promises, not deliberate infinite CPU loops; draining is separately limited to 100 jobs. The backstop is a test-run safeguard, not evidence of engine cancellation support.

```sh
cargo test --manifest-path experiments/boa-reaction-spike/Cargo.toml --locked --no-run
python3 - <<'PY'
import os
import signal
import subprocess

process = subprocess.Popen([
    'cargo', 'test', '--manifest-path',
    'experiments/boa-reaction-spike/Cargo.toml', '--locked',
    '--', '--test-threads=1', '--nocapture',
], start_new_session=True)
try:
    code = process.wait(timeout=60)
except subprocess.TimeoutExpired:
    os.killpg(process.pid, signal.SIGKILL)
    process.wait()
    raise SystemExit('FAIL: Boa probe exceeded 60-second external backstop')
raise SystemExit(code)
PY
cargo fmt --manifest-path experiments/boa-reaction-spike/Cargo.toml --check
cargo clippy --manifest-path experiments/boa-reaction-spike/Cargo.toml --locked --all-targets -- -D warnings
```

The wrapper uses POSIX process groups, as in the previous spike; it is not a Windows test harness. With `--nocapture`, the expected assertion panic is printed even when the suite passes. Read the result count and the named test; do not interpret that diagnostic as an unanticipated suite failure or supported exception-based cancellation.
