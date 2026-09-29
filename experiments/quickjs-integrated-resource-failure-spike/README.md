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
run passes 15 integrated tests; the baseline run is a negative control for the
Promise stop-swallow. Existing experiment suites and the preserved large
RegExp compiler reproducer remain separate evidence. The result is Outcome B:
allocator rejection can still be caught by source, and an engine heap failure
can occur before the trusted allocator latch. See the [review](../../docs/reviews/2026-09-30-quickjs-integrated-resource-failure-spike.md).

The Rust ASan/UBSan link is unavailable on the local macOS arm64 toolchain;
sanitizer evidence for the earlier C patch harness is unchanged. This crate is
not a production runtime, parser, loader, engine selection or fork.
