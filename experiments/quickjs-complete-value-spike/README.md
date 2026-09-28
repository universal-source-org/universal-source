# QuickJS complete value-boundary spike

**Outcome A — complete RFC §6 value boundary feasible.** Both directions are demonstrated as bounded, non-executing structural copies on the pinned public embedding path. This is an isolated feasibility experiment, not production execution, a complete sandbox or an engine selection.

Baseline: `fb7ce8e336d9910a327568ea9921876c595d4948`. Unchanged pins: rquickjs/core/sys `0.14.0`, revision `d7ef5eeae702fea24c03643064de454f1c1dd4b0`; QuickJS-NG `0.16.2`, revision `0fdea21ff1090084e91dad812b343d92e79ba9d9`. Defaults are off; `std` and `loader` are enabled. The experiment directly depends on the unchanged [class bridge](../quickjs-class-bridge-spike/README.md); no security primitive is forked and no production dependency changes.

The [review](../../docs/reviews/2026-09-28-quickjs-complete-value-spike.md) records the matrix, API provenance, safety audit and limitations. The [completed plan](../../docs/plans/completed/2026-09-28-quickjs-complete-value-spike.md) records exact full regression commands/results.

## Working interface

`Boundary::new(&Ctx, Limits)` runs only in a pristine realm. It captures the class gate, original prototypes and intrinsic identities. `exclude_helper(&Value)` registers each host-owned wrapper/registry/nested helper before source exposure. Transferable host data is intentionally not registered. Host-held references prevent identity reuse; source-visible markers are never trusted.

`outgoing(&Value)` returns an owned `Portable` tree and `Usage`, or `Failure`. Object classification precedes all candidate reflection. Only ordinary records and arrays can reach prototype/key/descriptor inspection. The descriptor wrapper never reads through getters. Strict UTF-16 decoding handles strings and keys, including embedded NUL. Active-path identity tracking rejects cycles and allows independent copies of repeated references, charging every occurrence.

`incoming(&Portable)` validates the entire Rust tree and budgets before constructing anything in JavaScript. Native allocation and own data-property definitions use original prototypes despite poisoned globals, inherited setters and iterators. Sorted Rust map iteration gives Unicode scalar key insertion order; JavaScript integer-index enumeration still applies. No serializer, iteration protocol or source callback implements either converter.

`Invalid` maps to `INVALID_RESULT` outgoing and `INVALID_ARGUMENT` incoming. `ResourceLimit` maps to `RESOURCE_LIMIT`; failures never publish a partial tree. `Engine` is an embedding failure, not a transferred exception or a new Source API code. Operation-schema validation and duplicate-key rejection in textual JSON precede this already-validated host-tree interface.

## Bounds and scope

Defaults: root depth 0, maximum depth 16, 1,024 expanded nodes, 4,096 UTF-8 bytes per string/record key, 16,384 total text bytes, and 32,768 expanded bytes. The explicit portable-size model charges 8 bytes per node plus UTF-8 value/key bytes. It is not Rust allocator usage or a normative wire encoding. Array structural keys are validated separately (at most 10 bytes needed for canonical indices/length), not transferred as record keys. All counters use checked addition. Configuration rejects depth above 64.

Pristine intrinsic capture uses a bounded graph (depth 16, 4,096 identities), with iterator and specialized-function witnesses as well as the global object. The combined intrinsic/helper exclusion list is capped at 8,192 identities. Test runtime safeguards: 32 MiB heap, 512 KiB stack, plus the external process timeout below. Native key enumeration/string extraction can allocate a whole engine buffer before the boundary can inspect its size; this does not prove CPU/heap/OOM enforcement. That separate feasibility gap is next.

Twenty tests cover all portable primitives, Unicode/surrogates/keys, records/classes, arrays, native brands, private Date/buffers, proxies, helper identities, intrinsic identities, hostile hooks/reflection, negative controls, cycles/repetition, exact limits, incoming construction/order, round trips and fresh realms. Working-path hook/trap counters remain zero. Negative controls intentionally execute source hooks. No job pumping or package rewriting occurs. Earlier module capture, lifecycle, parser, global and dynamic-code experiments remain unchanged and are rerun.

Three additional unsafe blocks live only in [inspect.rs](src/inspect.rs): descriptor ownership, UTF-16 extraction, and paired buffer release. The reused bridge has one unchanged unsafe block. There are no unsafe functions/impls or private engine accesses. Miri was not used to claim validation of native C FFI.

## Reproduce

From the repository root with Rust/Cargo 1.96 and the native C toolchain, precompile and run with the established external timeout:

```sh
cargo test --manifest-path experiments/quickjs-complete-value-spike/Cargo.toml --locked --no-run
python3 - <<'PY'
import os, signal, subprocess
command = ['cargo', 'test', '--manifest-path',
           'experiments/quickjs-complete-value-spike/Cargo.toml', '--locked',
           '--', '--test-threads=1', '--nocapture']
process = subprocess.Popen(command, start_new_session=True)
try:
    result = process.wait(timeout=60)
except subprocess.TimeoutExpired:
    os.killpg(process.pid, signal.SIGKILL)
    process.wait()
    raise SystemExit('external 60-second timeout')
raise SystemExit(result)
PY
cargo fmt --manifest-path experiments/quickjs-complete-value-spike/Cargo.toml --check
cargo clippy --manifest-path experiments/quickjs-complete-value-spike/Cargo.toml --locked --all-targets -- -D warnings
```

All 20 tests pass, zero failed/ignored; fmt and strict Clippy pass on macOS arm64. RFC 0001 remains proposed. No engine/parser is selected, ADR created or production runtime code/dependencies changed. Next: scoped CPU/allocation/memory/RegExp enforcement feasibility, before engine selection or production integration.
