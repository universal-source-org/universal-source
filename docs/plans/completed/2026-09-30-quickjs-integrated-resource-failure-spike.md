# QuickJS integrated resource/failure composition spike

Status: completed — Outcome B. Baseline `b01c9413299c6f1963a3b3e1e40accf55c4286e0`.

Scope was one experiment-local Rust path combining the pinned QuickJS source,
the existing 12-edit async patch, hardening, RegExp admission, §6 transfer,
resource probes and fresh-runtime retirement. No production dependency, pin,
RFC, ADR, engine/parser selection or fork authorization was in scope.

Implemented the isolated crate and build driver under
`experiments/quickjs-integrated-resource-failure-spike/`. It verifies patched
versus baseline source and binary linkage, exercises 15 integrated tests,
retains a negative control, and records the allocator/engine-heap failure
classification boundary. The integrated path passed its ordinary, terminal,
RegExp, lifecycle, hardening, value-boundary and fresh-runtime tests, but the
allocator diagnostic permits source catch continuation and engine heap failure
can be caught before the custom allocator latch. This is a bounded unresolved
integration issue, so the resource/failure composition gate remains open.

Validation: existing experiment suites 186 tests; integrated patched 15 tests;
baseline negative control; Python conformance 6; runtime 18; format/clippy;
async fix pass; pinned RegExp compiler blocker reproduced. New Rust ASan/UBSan
linking is unavailable on this macOS arm64 toolchain; prior C sanitizer evidence
was not changed. The user's `.gitignore` change was preserved and excluded.

Next: a focused public-API allocator/engine-heap failure-classification spike.
