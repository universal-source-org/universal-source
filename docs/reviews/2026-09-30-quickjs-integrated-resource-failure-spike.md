# QuickJS integrated resource/failure composition spike

Status: completed — **Outcome B: bounded integration issue remains**. This spike
started from `b01c9413299c6f1963a3b3e1e40accf55c4286e0`, on `main`, with the
pre-existing `.gitignore` change preserved and excluded. It is evidence only;
RFC 0001 remains proposed, no engine/parser/fork is selected or authorized, and
production `runtime/` remains declarative.

## Question and exercised architecture

The experiment asks whether the existing 12-edit async-termination patch and
the existing RegExp admission policy compose through one Rust binding path. The
new `experiments/quickjs-integrated-resource-failure-spike/` crate creates a
fresh Runtime/Context for every invocation, sets the public heap/stack limits,
installs the public interrupt handler and allocator adapter, applies the
unchanged dynamic-code and global hardeners before module evaluation, captures
the export through the native module hook, and uses the existing admission and
complete value-boundary crates. It pumps jobs only while the invocation is live,
closes the resolver and active flag before conversion, performs no final drain,
destroys the context/runtime, and then proves an independent generation works.

The build driver copies and SHA-verifies the pinned QuickJS source, applies only
the already-demonstrated 12 async guard edits to the experiment-local copy, and
patches only this crate's `rquickjs-sys`. The patched source SHA is
`9a926c4ed02517c84ecfbc9d925b6ceb782b3119f6d05a3d58280c66363e1f3f`; the
unmodified pin is `3a6b52a225c21709ef1fd314183efd3af862e867ce4ef8caabcae67922a745b3`.
Cargo metadata, the generated C source, archive/executable hashes and exported
QuickJS symbols are checked. The baseline Rust executable is a negative control:
the same Promise/RegExp stop publishes marker `1`, while the patched executable
does not.

## Directly exercised results

The integrated patched suite has 15 tests passing. It covers ordinary
initialization, direct-call and Promise/job exceptions (including catchable
exceptions), cancellation, deadline and admission stops through direct,
Promise, await, reaction, async-generator and for-await paths, no continuation
after those trusted stops, no final drain, resolver invalidation and stale
generation rejection, and a healthy fresh runtime after every case.

Hardening is invoked in the integrated path. The 80-case dynamic-code denial
corpus returns 80 and the global surface has no ambient OS/network/filesystem
objects. The integrated native reachability witnesses report clean walks
(`368/1482` and `357/1457` nodes/edges); the native RegExp constructor and
wrapped operations are retained only by host setup witnesses and are not
reachable from the package. Boundary tests cover helper identity rejection,
accessors/proxies/coercion hooks, cycles, Unicode, dense arrays and finite
depth/node/text budgets. Inbound and outbound values remain non-executing.

The accepted RegExp policy is exercised at the bound, with matching subject
sizes and deadline around matching. The 64,000-reference forward-backreference
case is rejected by literal/dynamic admission before native compilation. The
original pinned compiler reproducer still enters the non-polling path and is
contained only by the historical external watchdog; it remains outside the
admitted profile. A finite native-work probe also shows that `Array.join` can
perform work between interrupt checkpoints; the policy must continue to admit
only bounded operations and must not promise a real-time interval.

## Failure and resource boundary

Controlled allocator failpoints around own-property enumeration, property-name
and string copying, array/object traversal, and inbound construction fail closed
with a trusted ResourceLimit latch. The adapter is conservative accounting, not
an RSS bound; Rust/host allocations, arena slack and process memory remain
outside that accounting. Ordinary stack exhaustion and forged RangeError values
remain ordinary SourceError and can be caught.

The integrated diagnostic exposes the unresolved boundary. A trusted allocator
rejection returns a terminal host outcome and prevents capability action/result
publication, but source can still execute a catch continuation (`notes: [77]`);
in one Promise case an already synchronous marker also executes (`[77, 78]`).
The existing async patch fixes swallowed uncatchable Promise/async stops, but it
does not turn every allocator callback refusal into an uncatchable engine stop.
Separately, QuickJS may reject against its engine heap ceiling before invoking
the custom allocator: a large `String.repeat` was caught by source and returned
42 with no allocator rejection. Lowering the public engine limit during inbound
or outbound transfer likewise produces an ordinary engine error rather than a
trusted host reason. These observations are not converted into a production
classification by this spike.

This is a narrow integration issue, not a new engine patch request: closing it
would require deciding how the RFC treats allocator/heap failures and proving
that policy through the public binding or a separately scoped engine change.
The experiment did not broaden the patch, call private QuickJS APIs, drain jobs
after terminal completion, reuse a runtime, execute source getters/coercions, or
weaken the RegExp admission gate.

## Evidence inherited rather than reproved here

The complete parser/import-attribute strategy, complete module/path containment,
all historical class/descriptor ownership audits, platform builds and Apple
embedding remain prior evidence or open work. The integrated crate reuses their
fixtures and oracles; it does not claim that a Rust fixture loader is production
module containment. The historical Promise-swallow and RegExp compiler reviews
remain unchanged and both blockers reproduce on the pinned unmodified engine.

## Validation

- Integrated patched Rust tests: 15 passed; `cargo fmt --check` and clippy with
  `-D warnings` passed.
- Baseline negative control: one test passed and reproduced the swallowed marker;
  source SHA and patched SHA were distinct and positively recorded.
- Existing QuickJS/Boa/static suites: 186 tests passed across 13 experiment
  crates. Declarative Python conformance: 6 tests passed. Reference runtime:
  build, 18 tests, formatting and clippy passed.
- Async-fix regression: pass, including sanitizer/leak-clean C harnesses from
  the unchanged prior experiment. The pinned RegExp compiler reproducer still
  timed out under its external five-second containment guard with zero interrupt
  callbacks.
- The new Rust sanitizer link was attempted on macOS arm64 but cannot link
  compiler-rt ASan/UBSan symbols into this Rust target; no sanitizer claim is
  made for the new crate. The prior C sanitizer evidence remains valid.
- Documentation/link/JSON/schema/whitespace audit passed (568 links, 44 anchors,
  51 JSON files); generated build/target files are ignored.

## Decision

**Outcome B.** QuickJS remains the strongest-evidenced candidate, but this
composition gate does not pass. The smallest next task is a focused public-API
failure-classification spike: determine whether engine-heap and allocator
failures can be made a trusted, no-continuation terminal class without another
substantive QuickJS change, while preserving the baseline swallow and RegExp
compiler reproducers. Do not begin V8/JSC evaluation, production integration,
or engine selection from this result.

Engine selection is still not justified. Remaining gates include module/path
containment, final parser/preflight strategy, multi-platform/Apple embedding,
and patch ownership/update strategy. No production dependency/pin, RFC status,
engine/parser selection, ADR, or maintained fork changed here.
