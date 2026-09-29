# QuickJS resource-enforcement spike

**Outcome B — blocker demonstrated.** RegExp matching responds to the pinned interrupt hook, but RegExp compilation contains a long native path that never calls it. A local 320,007-byte pattern exceeds the five-second safety guard despite an armed interrupt handler and an 8 MiB engine memory ceiling. The outer kill is failed engine-control evidence, never a passing interruption test.

Baseline: `be9120b85f7a1a7664bacb71c2efd240355f8f10` (`test(engine): prove complete value boundary`). Phase 3 remains active. This isolated experiment does not implement production JavaScript, select an engine/parser, accept RFC 0001, or amend its resource semantics. See the [review](../../docs/reviews/2026-09-29-quickjs-resource-spike.md) and [completed plan](../../docs/plans/completed/2026-09-29-quickjs-resource-spike.md).

## Pins and boundaries

- rquickjs/core/sys `0.14.0`, revision `d7ef5eeae702fea24c03643064de454f1c1dd4b0`.
- QuickJS-NG `0.16.2`, revision `0fdea21ff1090084e91dad812b343d92e79ba9d9`.
- Defaults disabled; `std`/`loader` enabled. Separate lockfile equals the class-bridge lockfile with only the root package renamed. No custom allocator, futures, parallel feature or new dependency.
- Local evidence: macOS 27.0 (26A428), arm64, Rust/Cargo 1.96.0. Native C toolchain as in the earlier spikes. Other platforms/build profiles are untested.
- Each realm has its own Runtime/Context, 8 MiB accounted heap limit and 128 KiB stack setting. Existing dynamic/global hardening scripts are included unchanged. No network, credentials, providers with external effects, or package source rewriting.

The library and executable both forbid unsafe Rust: **zero unsafe blocks/functions/impls**. All engine calls go through existing safe rquickjs interfaces. Limits and host counters are experiment policy, not RFC constants.

## Reproduce the bounded tests

From the repository root, compile first, then use the established process-group timeout pattern:

```sh
cargo test --manifest-path experiments/quickjs-resource-spike/Cargo.toml --locked --no-run
python3 - <<'PY'
import os, signal, subprocess
command = ['cargo', 'test', '--manifest-path',
           'experiments/quickjs-resource-spike/Cargo.toml', '--locked',
           '--', '--test-threads=1', '--nocapture']
process = subprocess.Popen(command, start_new_session=True)
try:
    result = process.wait(timeout=60)
except subprocess.TimeoutExpired:
    os.killpg(process.pid, signal.SIGKILL)
    process.wait()
    raise SystemExit('FAIL: external 60-second timeout')
raise SystemExit(result)
PY
cargo fmt --manifest-path experiments/quickjs-resource-spike/Cargo.toml --check
cargo clippy --manifest-path experiments/quickjs-resource-spike/Cargo.toml --locked --all-targets -- -D warnings
```

**11 unit tests pass**, no doctests. They cover CPU loops/finite work/recursive computation; trusted cancellation/deadline/budget reasons; catch/finally/queued-work suppression; load/call initialization stops; Promise reaction/await work; pending and self-extending jobs; retained objects/arrays/strings/nested records/closures/Promise graphs/Map/Set; recoverable stack rejection and forged RangeError; RegExp matching; forged thrown records; and candidate rejection after host observation. They are focused primitive/policy probes, not the full requested Outcome A matrix or a conformance suite.

CPU fixtures stop on the second armed callback. Memory fixtures have a secondary 100-callback CPU guard and assert that it did **not** fire: a CPU stop cannot masquerade as heap rejection. Job fixtures take at most 8 or 32 steps and never perform a final drain. All returned-control resource cases destroy the old runtime and evaluate `21 * 2` in a fresh hardened realm. Weak runtime checks additionally prove final ownership release in CPU, initialization, heap, pending-job and candidate probes.

## Reproduce the blocker separately

```sh
cargo build --manifest-path experiments/quickjs-resource-spike/Cargo.toml --locked --bin regexp_compile
python3 experiments/quickjs-resource-spike/probe_regexp.py
```

Do not invoke the binary without containment. The runner uses the same POSIX process-group watchdog mechanism, with a deliberately shorter **five-second per-process guard**. It runs 1,000, 4,000, 16,000 and 64,000 forward references in independent processes. A timeout returns **exit 2**, explicitly reporting that engine interruption and in-process retirement were not established. Other child failures also fail the runner. A fast machine may finish the last input: the binary still asserts successful compilation with **zero** interrupt callbacks; wall time is diagnostic, not a portable test constant.

The complete source fixture is [regexp_compile.js](src/regexp_compile.js):

```js
references => new RegExp("\\k<a>".repeat(references) + "(?<a>x)")
```

It performs only string construction and RegExp compilation, never matching. The host compiles that fixed function before arming the hook, then passes an integer. No host checkpoint is inserted into its source. Every armed callback would print a marker and immediately request interruption. The host deliberately enters with the request armed to measure engine polling; this is a diagnostic entry policy, not a proposed production admission path. A real host must also check before entry and before publication. Those checks cannot interrupt a request arriving during the compiler's native scan.

Final local observation: 1,000 / 4,000 / 16,000 references returned after 18 / 116 / 1,638 ms respectively, all with zero callbacks; 64,000 references entered and was killed at five seconds with no callback marker or return. `ENGINE_ACCOUNTED_BYTES` is measured **after returned temporary values were dropped**, not peak memory or RSS.

The review explains the repeated full-pattern scans and the missing compiler checkpoint. A finite heap does not establish a useful CPU checkpoint interval. Smaller patterns returning with zero callbacks independently demonstrate the blind path; the outer timeout only contains its larger instance.

## Limits and stopping point

Work stopped expanding after the RegExp blocker. Complete OOM classification, adversarial §6 native-buffer exhaustion, all initialization/job stack cases, and integrated provider delivery after every failure class are not newly proven. Default heap rejection and stack RangeError can be caught by source; neither is a trusted termination latch. Existing allocator-latch, whole-realm retirement and late-generation delivery evidence is preserved and rerun in the lifecycle suite.

The next task is a scoped review of how to close the RegExp compilation checkpoint gap, preserving this reproducer and the RFC. No patch, version change, process-containment design or source restriction is authorized by this result. Module/path and final parser work remain deferred while the resource blocker is unresolved.
