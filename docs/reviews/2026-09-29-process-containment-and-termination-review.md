# Process containment and async-termination review

## Outcome and scope

**Outcome C — a bounded engine-level fix is the more plausible portable path; process containment is not universally deployable.**

Two in-process termination blockers stand on the pinned QuickJS-NG revision: native RegExp compilation performs long work without polling the interrupt handler (resource spike), and several Promise/async internals convert an uncatchable host stop into an ordinary Promise rejection so source keeps running (admission spike). The first is already neutralised **in process** by RegExp pre-admission, which works on every platform. The second is the only remaining in-process termination gap, and this review finds it is plausibly **one small, enumerable engine defect with a fix pattern that already exists in the same source file**: `promise_reaction_job` already guards the uncatchable case, and the sibling conversion sites simply omit the same guard.

Process containment (a disposable helper process the parent kills on its own deadline) would restore a hard upper bound on execution regardless of either engine defect, and its trusted-kill classification maps cleanly onto lifecycle Model B. But it cannot be deployed on iOS, iPadOS or tvOS for ordinary App Store applications, which cannot spawn arbitrary executable helper processes. The project charter lists those as target platforms. Process containment therefore cannot be the general portable answer; it remains a strong desktop/Android option and a defence-in-depth layer.

Because the remaining in-process blocker is bounded and credibly fixable, and a fix would run in process on every platform including Apple mobile/TV, the more plausible portable path is a narrowly scoped engine-fix/version feasibility experiment, not a containment architecture that excludes required platforms. This review authorises neither remedy. It selects Outcome C and names the next task.

This is a design and evidence review. No engine is selected, no engine is patched or upgraded, no RFC/ADR/production change is made, and both existing blockers still reproduce (validated below). The prior Outcome B documents are unchanged.

Baseline: `2b05e0b77e2a4a123c8c9bb8df48cb080841edc2` (`test(engine): record RegExp admission spike Outcome B`), clean tree, no other active plan. Phase 3 remains active. [RFC 0001](../../spec/rfcs/0001-javascript-execution-binding.md) remains proposed and unedited; no engine or parser is selected; no ADR exists.

### On external access

The task's top-level constraints forbid external/remote access ("Do not access external services, accounts, credentials or third-party systems"; "Do not interact with any remote service or system during this task"). Parts 2 and 4.3 conditionally invite web research "if available". The top-level prohibition is treated as binding. Consequently: platform restrictions below are recorded as **established reviewer knowledge**, explicitly flagged for confirmation against current vendor documentation before any implementation, exactly as RFC 0001 already cites ES2023 steps "as known to the reviewer, not re-fetched"; and a concrete upstream QuickJS-NG fix determination is **deferred** to a remote-permitted follow-up rather than asserted. Evidence about the pinned engine comes only from the local registry source.

## Pins and provenance

rquickjs/core/sys **0.14.0** (`d7ef5eeae702fea24c03643064de454f1c1dd4b0`), QuickJS-NG **0.16.2** (`0fdea21ff1090084e91dad812b343d92e79ba9d9`), unchanged. `rquickjs-sys-0.14.0/quickjs/quickjs.c` has SHA-256 `3a6b52a225c21709ef1fd314183efd3af862e867ce4ef8caabcae67922a745b3`, equal to the resource and admission reviews; line numbers below refer to it. Environment: macOS 27.0 (26A428), arm64, Rust/Cargo 1.96.0, debug test profile.

# Part 1 — Process-containment feasibility

Containment runs each JavaScript execution domain inside a child process. The child holds only a QuickJS runtime and the invocation's mutable state; the parent host owns all authority, all §6 validation, the deadline clock and the kill switch. Nothing here is implemented; this is a design review.

## 1.1 Process boundary models

| Model | Isolation boundary | Startup cost | Memory overhead | Teardown | Crash/kill containment | State-leakage risk | Model B fit |
| --- | --- | --- | --- | --- | --- | --- | --- |
| One child per invocation | OS process per call; strongest | Full process spawn + QuickJS init + snapshot re-parse per call | One live child at a time plus spawn churn | Kill and reap; nothing to reset | Child death cannot touch the parent | Lowest: no reuse of any mutable state | Exact: fresh realm equals fresh process |
| Reusable child, fresh Runtime/Context per invocation | Process reused; a new QuickJS runtime/queue domain per call | Amortised spawn; still full realm re-init per call | One resident child; steadier | Retire the realm; keep the process | Parent survives; but a wedged child must still be killed, losing reuse | Higher: shared process address space across calls (heap fragmentation, residual native allocations, bytecode cache) must be proven not to carry mutable JS state | Fits **only** if realm retirement in a surviving process is proven equivalent to a fresh realm, which the resource spike could not establish for a non-returning compile |
| Small worker pool, one active invocation per worker | Several reusable children; one call each at a time | Amortised across the pool | N resident children | Per-worker realm retirement; kill on wedge | Parent survives; a killed worker is replaced | Same as reusable child, times N; plus cross-worker isolation must hold | Same caveat as reusable child |
| Process per logical instance | Process lives as long as the logical instance; fresh realm per call inside it | Spawn per instance load | One child per live instance | Kill on instance disposal | Parent survives | Cross-call reuse inside one instance must still give fresh mutable JS state; long-lived process accumulates native state | Fits per-call reset only if in-process realm retirement is trustworthy |

Evidence-based reading. RFC 0001 §4/§10 require that reusing a logical instance reuse **no** mutable JavaScript state, and §10 warns that "retiring a realm may require retiring its entire engine runtime/queue domain". The resource review established a hard fact against in-process reuse of a **wedged** realm: "one cannot safely destroy a runtime while its owning thread is still executing it", and the non-returning RegExp compile "never returns before the watchdog kill", so its in-process retirement and subsequent same-process health are **not established**. That single fact is decisive for model choice under containment:

- For the deadline/cancel/resource paths that actually wedge the engine (the whole reason to add containment), a surviving process cannot be trusted to have cleanly retired the realm, because the parent had to kill a thread mid-execution. Only tearing down the **process** guarantees retirement.
- Therefore the reusable-child, pool and per-instance models can reuse a process only after **normal** returns. On any stop that required a kill, the worker must be discarded and respawned — which is the per-invocation model for exactly the cases containment exists to handle.

Conclusion: a pool is a viable **latency optimisation for normally-returning calls**, but correctness on the terminating paths requires fresh-process semantics (spawn, or kill-and-respawn). The safe default is **one fresh child per invocation**, with a warm pool permitted only when a worker that has ever been killed or interrupted is never reused. This matches Model B without weakening it.

## 1.2 Trusted stop classification and ordering

The parent must classify every terminal outcome from **its own trusted state**, never from the child's self-report. A child that is about to be killed, is wedged in a non-polling compile, or has crashed cannot be trusted to name the reason. The child may *propose* an ordinary source outcome (success envelope, source failure) over IPC; the parent still validates it and may override it with a stop it has already latched.

| Terminal reason | Parent's trusted basis |
| --- | --- |
| `CANCELLED` | Parent latched a caller cancellation before accepting any child result. |
| `TIMEOUT` | Parent's monotonic deadline (from admission, including queueing) expired before a valid child result was accepted. |
| `RESOURCE_LIMIT` | Parent observed a documented limit breach: child killed by an OS/cgroup/job memory or CPU cap, or the parent's own IPC/output/graph bound exceeded. |
| Ordinary source failure | Child delivered a well-formed §6 failure envelope (or a value the parent maps to `SOURCE_ERROR`/`INVALID_RESULT`) **and** no stop was latched first. |
| Child crash | Child exited abnormally (signal/nonzero) with no stop latched and no valid result: `SOURCE_ERROR`, logical instance disposed. |
| Protocol failure | Malformed/oversized/out-of-generation IPC frame, or a frame after the delivery gate closed: `SOURCE_ERROR` (or `RESOURCE_LIMIT` for a bound breach), instance disposed. |

Required ordering (the containment analogue of the RFC §5 terminal boundary and the lifecycle-decision gate order):

1. **Parent latches the stop reason** in trusted state (first-wins, as the resource/admission gates already do).
2. **Delivery and service authority close**: the parent stops accepting child results and stops honouring any further host-service request from that generation.
3. **Child is asked to stop, then terminated**: a cooperative stop request, then an unconditional kill after a bounded grace period. Correctness does not depend on the child obeying the cooperative request — the kill is the guarantee.
4. **No later child result is accepted**: any frame arriving after step 2, including one produced during the grace period, is discarded. This is the cross-process form of "no final drain" and "late completions must not deliver".
5. **Parent publishes exactly one host-owned outcome**, then reaps the child and (on any stop that began execution) disposes the logical instance.

This preserves RFC 0001 semantics without amendment: exactly-one terminal outcome (§5, Source API), classification from trusted host state not engine/child text (§7), cancellation as request-observe-revoke-quiesce where **process death supplies quiescence** (§5), and disposal of the logical instance after an execution-side stop (§4/§7). The parent's kill replaces the unreliable in-process cooperative quiescence that the resource spike could not guarantee.

## 1.3 §6 value transfer over IPC

The complete value-boundary spike already fixes the portable domain: `null`, boolean, finite number, string, dense array, ordinary record; everything else rejected, no getters/`toJSON`/iterators invoked, structural non-executing copy. IPC transfers **exactly that domain** and nothing more. No new portable value type is added (RFC §6 forbids it).

- **Framing.** Length-prefixed binary frames: a fixed header (magic, protocol version, frame type, invocation generation ID, request/response ID, payload length) then the payload. The length prefix is validated against a published maximum before any read; the parent never grows a buffer to an unbounded child-declared size.
- **Encoding.** The payload is a canonical structural serialisation of the §6 tree, not `JSON.stringify` of an untrusted object graph (RFC §6 bans that path). A JSON text profile is acceptable **only** if produced by the trusted host converter on already-validated §6 values and re-parsed with duplicate-key rejection and finite-number enforcement; a compact typed binary encoding is equally acceptable. The choice is a host-policy detail, not a portable value type.
- **Size/depth limits.** Both directions carry published bounds on total bytes, nesting depth and total expanded node count (RFC §6/§10 "bound total expanded size as well as depth"; excess is `RESOURCE_LIMIT`, never truncation).
- **Malformed messages.** Any header/version/length/type error, truncated payload, or value outside §6 is a protocol failure → parent discards and classifies (`SOURCE_ERROR`/`RESOURCE_LIMIT`), never a partial or coerced success.
- **Duplicate/reserved keys.** Duplicate member names are rejected at decode (as the runtime and `json` service already require). `__proto__`, `constructor`, `toString` are ordinary string data keys and become own data properties on the receiving side, never prototype operations (RFC §6).
- **Unicode.** Strings are validated Unicode scalar text; unpaired UTF-16 surrogates are rejected, including keys (RFC §6). No trimming/normalisation/case folding. Incoming object key order is normalised to ascending scalar order on entry to JS (RFC §6); outgoing object order is not a Source API semantic; array order is preserved.
- **Numbers.** Finite binary64 only; `NaN`/±∞ rejected; negative zero normalised to zero at the JSON boundary; the encoding must round-trip the exact finite value (RFC §6/§10). Valid page inputs already fit the safe-integer range.
- **Trust direction.** Parent → child input is trusted (already validated by the host). Child → parent output is **untrusted** and validated by the trusted converter in the parent before publication. The child never sends host-native handles, functions, promises or service wrappers; those are not §6 values and cannot be represented in the frame at all.

This is exactly the existing §6 boundary relocated to a process edge; it neither widens nor narrows the domain.

## 1.4 Host services over IPC

The child owns **no ambient authority** (RFC §8/§9: no `fetch`, no globals, capabilities are invocation-scoped injected wrappers). Under containment the injected `context.services.NAME` wrappers become thin stubs in the child that marshal a request to the parent, which holds the real capability and enforces every grant.

A request/response protocol carries at least:

- **invocation generation ID** — ties the request to the currently active invocation; the parent rejects any other generation;
- **service name** and **method/operation** — validated against the declared `capabilities.host` and the (future) service profile;
- **request ID** — matches the eventual response; enables the child's realm Promise to settle;
- **§6-compatible arguments and results** — same domain and validation as §1.3; arguments copied/validated before the parent acts, results copied before fulfilment (RFC §8);
- **cancellation/revocation state** — the parent tags responses with the generation and refuses to service requests from a closed generation.

Late-response rejection: when the invocation retires (normal boundary or kill), the parent **increments the generation and closes the gate first** (§1.2 step 2). Any service response computed afterward, or any child request arriving afterward, is dropped and never settles a JavaScript resolver — in the fresh-process case there is no child left to settle, and in a surviving pool worker the stale generation is refused, exactly matching the existing `late_completions_and_old_resolvers_cannot_enter_next_generation` evidence. Authority is re-checked on every use, including revocation during an awaited call (§8). This defines the boundary only; no final service method signatures are defined here (out of scope, as in RFC §8).

## 1.5 Model B lifecycle mapping

| Model B requirement (RFC §4/§5) | Containment realisation |
| --- | --- |
| Fresh mutable JS state per invocation | Fresh child process, or a fresh Runtime/Context in a never-killed pool worker. |
| Immutable source snapshot reuse | Parent holds the validated snapshot and ships identical bytes to each child; immutable parsed-code caching is a child-local optimisation that must expose no mutable JS value (RFC §10). |
| No cross-call JS globals/module state | Guaranteed by fresh process, or by fresh runtime + proof of no residual mutable state in a reused process. |
| No final Promise drain | Parent closes the gate and kills/retires; queued jobs die with the process/realm, no handlers run (§5). |
| No source cleanup callback | Process death runs no `finally`/resolver/species code (§4/§5). |
| Late native delivery rejection | Generation gate (§1.4); stale responses dropped. |
| Full retirement after execution-side termination | Process kill is the retirement; the logical instance is disposed. |

Pooled vs fresh (the decisive question the task asks): a pooled child can satisfy Model B **only for calls that return normally**, where a fresh Runtime/Context provably resets mutable state. For any call the parent had to **kill or interrupt**, the resource review's rule applies — the realm cannot be trusted to have retired cleanly in a surviving process — so that worker must be destroyed, not reused. Hence: pool for throughput on the happy path; fresh-process (spawn or respawn) semantics on every terminating path. Fresh processes are required exactly where in-process retirement is unprovable.

## 1.6 The RegExp blocker under containment

Containment does **not** make native RegExp compilation interruptible; `lre_compile` still has no poll site. What it changes:

- the parent's deadline/cancel/resource state is authoritative;
- a child stuck in a non-returning compile is **terminated** by the parent;
- the parent process survives;
- no in-process QuickJS cleanup is needed for that invocation, because the whole address space is discarded.

Is that sufficient for RFC §§5/7/10? Yes, on platforms where the kill is available. §5 requires eventual interruption under a documented policy and explicitly disclaims hard preemption and precise instruction interruption; a bounded-latency parent kill is a stronger guarantee than the cooperative in-process checkpoint the RFC already accepts. §7 gets a trustworthy `TIMEOUT`/`RESOURCE_LIMIT` from the parent's own reason. §10 gets address-space/CPU control at the process/OS layer. Pre-admission remains valuable defence-in-depth (it turns most oversized patterns into a clean in-process `RESOURCE_LIMIT` without paying a spawn+kill), but containment alone bounds even an unknown non-polling native path. This does **not** claim the engine itself is fixed.

## 1.7 Promise stop-swallowing under containment

The admission spike showed that an uncatchable host stop (gate stop *or* engine deadline interrupt) can be converted into an ordinary rejection by Promise/async internals, after which source continues; a 1,000-iteration loop completed after ~1,002 interrupt callbacks. Under containment this in-process defect is **bypassed rather than fixed**: the parent does not rely on the interrupt being honoured. When its deadline/cancel/resource state latches, it closes the gate and kills the child. Source in the child may keep running until the kill lands, but it cannot produce a delivered result (the gate is closed), cannot acquire authority (services refuse the closed generation), and cannot outlive the process. So containment restores a **hard upper bound on wall-clock execution and a trustworthy terminal classification**, which is what §§5/7 require.

What remains unavailable with containment: **in-process** cooperative termination. On a host that cannot spawn a helper process, containment offers nothing, and the swallow defect is fully exposed. That is precisely why the engine-level remedy matters for portability (Part 4).

## 1.8 Resource accounting by layer

| Resource | QuickJS runtime | Child process | Parent |
| --- | --- | --- | --- |
| CPU time | Cooperative interrupt only (and not during non-polling native work) | OS/cgroup CPU cap; parent wall-clock kill | Authoritative deadline from admission |
| Wall-clock deadline | Not self-enforced | — | Monotonic clock, kill on expiry |
| Address space / RSS | `JS_SetMemoryLimit` (accounted bytes, not RSS; caught OOM can mask it) | OS rlimit/cgroup/Job-Object memory cap → kill = trustworthy `RESOURCE_LIMIT` | Aggregate across children |
| QuickJS heap | `JS_SetMemoryLimit` (8 MiB in probes) | Backstops the engine cap | Policy owner |
| Stack | `JS_SetMaxStackSize` (128 KiB); ordinary `RangeError`, no trusted latch | OS stack limit → kill | Policy owner |
| IPC volume | — | — | Published per-invocation frame/byte caps; breach = `RESOURCE_LIMIT` |
| Output/result size | — | — | §6 size/depth/node bounds before publication |
| File descriptors | — | Per-process fd limit; child needs only its IPC endpoints | Policy owner |

Containment's real gain over in-process is at the **child-process/OS layer**: memory and CPU caps become enforceable with a trustworthy kill reason, and the address space is reclaimed by process exit rather than by in-process GC during a wedge. On the reviewed macOS host these controls exist; the matrix below records where they do not.

# Part 2 — Platform viability

Classification of whether an ordinary third-party application on each platform can spawn and rely on disposable executable helper processes for per-invocation containment. **Basis: established platform knowledge, flagged for confirmation against current vendor documentation before implementation.** Mobile support is not inferred from desktop.

| Platform | Class | Basis |
| --- | --- | --- |
| Linux | Straightforward | `fork`/`exec`, `posix_spawn`; cgroups v2 for CPU/memory caps, rlimits, optional seccomp/namespaces; SIGKILL for a hard stop. Full control. |
| macOS | Straightforward | `posix_spawn`/`NSTask`, XPC helper services in an app bundle, `setrlimit`; App Sandbox + Hardened Runtime add entitlement requirements but permit bundled helpers. |
| Windows | Straightforward | `CreateProcess`; **Job Objects** give per-job memory and CPU/time limits and atomic kill-on-close; separate process = crash isolation. |
| Android | Possible with platform-specific architecture | A normal app cannot `fork`/`exec` arbitrary binaries (W^X; executing app-writable files is blocked on modern API levels). The supported mechanism is a **service with `android:isolatedProcess="true"`**: a separate, heavily sandboxed process with no ambient permissions, well matched to running untrusted JS. The OS controls its resources and can kill it. Requires designing around the isolated-service model, not `fork`. |
| iOS | Generally unavailable for third-party App Store use | App Store apps run one sandboxed process and **cannot spawn arbitrary child processes** (`fork`/`exec`/`posix_spawn` of app binaries are not permitted for third-party apps). App Extensions are system-launched, purpose-limited and not general per-invocation helpers. XPC on iOS is for system services, not third-party spawnable helpers. Executable-memory/JIT restrictions are a separate constraint (interpreted QuickJS is fine; process spawning is the blocker). |
| iPadOS | Generally unavailable for third-party App Store use | Same application model and restrictions as iOS. |
| tvOS | Generally unavailable for third-party App Store use | Same restrictions as iOS, plus tighter resource limits; no third-party helper-process facility. |

Apple mobile/TV specifics investigated (reviewer knowledge, confirm before implementing):

- **Spawning arbitrary helper processes:** not available to third-party App Store apps; process creation is a system privilege.
- **Executable child processes:** an app cannot ship and exec a separate executable it launches on demand.
- **Dynamic code / runtime restrictions:** third-party apps cannot map writable-executable memory (no JIT) except the system WebView's special entitlement. QuickJS runs as a bytecode interpreter, so *running JS in-process* is permitted; this constraint is about JIT/exec, orthogonal to the process-spawn blocker.
- **Extension / XPC applicability:** App Extensions and XPC do not provide a general "spawn a disposable helper per invocation" capability on iOS/iPadOS/tvOS; they are system-managed and purpose-scoped.
- **Can a normal App Store app rely on this design?** No. A containment architecture that depends on helper processes cannot ship in a standard iOS/iPadOS/tvOS App Store application.

Conclusion: process containment is **straightforward on Linux/macOS/Windows, achievable on Android via isolated-process services, and generally unavailable on iOS/iPadOS/tvOS**. It cannot be a universal solution for the charter's platform set.

# Part 3 — Cross-platform architectural consequence

If containment is unavailable on Apple mobile/TV, the project-level options are:

| Model | Description | Compatibility with charter / principles / roadmap |
| --- | --- | --- |
| **P — process-contained QuickJS on capable platforms only** | Desktop/Android get full JS via containment; Apple mobile/TV get no JS (or declarative only). | Conflicts with the charter's explicit iOS/iPadOS/tvOS targets for the intended JS scope. "Portability over platform convenience" disfavours a design that structurally excludes required platforms. Acceptable only if JS support is openly scoped by host profile (see S). |
| **H — heterogeneous containment** | Same Source contract, different isolation per platform (process on desktop/Android, in-process engine on Apple). | Compatible with the contract *only if* the in-process path on Apple can itself meet §§5/7/10 termination — which today it cannot, because of the swallow defect. So H reduces to "need an in-process termination story anyway", i.e. it depends on the Part 4 fix. Consistent with the principle that the contract is behavioural, not mechanism-specific, but not yet supportable without the engine fix. |
| **E — choose an engine that meets in-process termination everywhere** | QuickJS stays a candidate only if a bounded in-process fix exists; otherwise select an engine that terminates in-process on all platforms. | Directly consistent with "proof before standardization" and the roadmap's deferred engine selection. This is the portable-first framing and is what Outcome C feeds. |
| **S — reduce supported JS/platform claims** | JS becomes optional by host profile; some platforms claim declarative-only. | Compatible with charter honesty ("report unsupported behavior and narrower platform coverage honestly") and with minimal-abstraction principles, but it is a **contract-scope decision**, not an engineering fix, and would need an explicit decision record. It is the fallback if neither containment nor an engine fix serves Apple mobile/TV. |

No model is edited into the governing documents here. The analysis shows H and E both ultimately require an in-process termination path on Apple platforms, which is exactly what the Part 4 fix would provide; P and S are contract-scope retreats. This is why Outcome C (pursue the bounded engine fix next) dominates a containment-only architecture.

# Part 4 — QuickJS stop-swallowing root-cause review

The engine represents uncatchability as a flag on the Error object: `JS_IsUncatchableError` (`quickjs.c:12049`) and `JS_SetUncatchableError` (12068). `JS_ThrowInterrupted` (8642) — raised by `__js_poll_interrupts` (8648) when the installed handler returns true — creates an InternalError and marks it uncatchable, so an engine deadline/cancel interrupt is an uncatchable error, exactly like the admission gate's stop.

Decisive observation: **`JS_IsUncatchableError` is referenced in only five places in the entire file** — its definition (12049), and four call sites: the interpreter unwind (20901), async-function resume (21482, 21491) and `promise_reaction_job` (55584). Every other place that turns a pending exception into a Promise rejection omits the check. The engine already knows the correct pattern; a bounded set of sibling sites simply lack it.

`promise_reaction_job` (55558) is the template. Before converting an exception into a rejection it does:

```c
is_reject = JS_IsException(res);
if (is_reject) {
    if (unlikely(JS_IsUncatchableError(ctx->rt->current_exception)))
        return JS_EXCEPTION;          /* propagate; do not convert */
    res = JS_GetException(ctx);        /* only now convert to a value */
}
```

The conversion sites below do the `JS_GetException` step **without** the guard.

| Site | Line(s) | Native call that sets the pending exception | Converts to rejection immediately? | Checks uncatchable first? | Expected if it did |
| --- | --- | --- | --- | --- | --- |
| `js_promise_constructor` executor | 55900; conv. 55916 | `JS_Call(executor, …)` (55913) | Yes — calls the reject function with `JS_GetException` | No | Propagate `JS_EXCEPTION`; the Promise is left without a fabricated rejection and the uncatchable stop unwinds |
| `js_promise_resolve_function_call` thenable `then` lookup | 55766; `then` read 55801, `fail_reject` 55805 | `JS_GetProperty(resolution, then)` (55801) | Yes — `fail_reject` rejects with `JS_GetException` | No | Propagate; `await` and `Promise.resolve(thenable)` stop instead of resuming |
| `js_promise_resolve` wrapper (reached via `Promise.resolve`, `js_async_function_resume` await at 21516) | 56032; conv. 56109 | `JS_GetProperty(argv[0], constructor)` etc. | Yes | No | Propagate the stop out of the await path |
| `js_promise_all` iteration | 56218; `fail_reject` 56241 | `JS_GetIterator` / iterator step raising the stop | Yes | No | Propagate; `Promise.all` stops |
| `js_async_generator_resume_next` | 21837; conv. 21891 | `async_func_resume` returns exception (21890) | Yes — `js_async_generator_reject` with `JS_GetException` | No | Propagate; async generator body stop is not swallowed |
| `js_async_generator_completed_return` | 21799; conv. 21812 | `JS_GetException` at 21812 | Yes | No | Same |
| `js_promise_race` | 56361; conv. 56382 | iterator/`next` raising the stop | Yes | No | Propagate |
| `js_promise_try` | 56095; conv. 56109 region | `JS_Call(argv[0], …)` raising the stop | Yes | No | Propagate |

For each, propagation would **not** bypass legitimate user rejection handlers: an *ordinary* (catchable) exception still takes the `JS_GetException` path and becomes a rejection exactly as today; only the uncatchable stop (host interrupt or gate stop) would unwind. Resource cleanup is the manageable risk: each site frees temporaries (`error`, `resolving_funcs`, iterator/next handles, the in-flight promise) before returning, and an early `return JS_EXCEPTION` must free the same handles — which is precisely what `promise_reaction_job`'s early return already does correctly. So the guard is safe if each site frees its locals on the propagation path, the standard C-cleanup discipline already used in the file.

Bypass of user handlers is therefore **not** a concern for correctness of ordinary Promises; the change is deliberately invisible unless the exception is uncatchable, which only the host can create.

## 4.1 Minimal-patch hypothesis

Conceptually, at each conversion site:

```c
if (JS_IsException(x)) {
    if (unlikely(JS_IsUncatchableError(ctx->rt->current_exception)))
        goto propagate;   /* free locals, return JS_EXCEPTION */
    err = JS_GetException(ctx);
    /* ...existing convert-to-rejection... */
}
```

This is the exact guard `promise_reaction_job` already contains; the hypothesis is only that replicating it (or centralising it) at the sibling sites closes the swallow. Determinations:

- **How many sites:** on the order of six to eight in this revision (the table above). Three are confirmed by the admission spike's deterministic shapes (executor, thenable resolve reached by `await`/`Promise.resolve`, `Promise.all`); the rest are by source reading.
- **Shared helper:** yes — a small inline helper such as `promise_convert_or_propagate(ctx, &err)` returning "propagate" when the current exception is uncatchable would centralise the check and the free discipline, reducing the change to one call per site. This is the cleaner form and shrinks the audit surface.
- **Confident enumeration:** *moderately*. All conversion sites are reachable from the Promise/async-generator implementation region and from `JS_GetException` uses near a reject call; the file is self-contained (no macro indirection hides these). But "confident" requires an exhaustive pass over every `JS_GetException`→reject pairing plus the async-function/await paths, which is the core deliverable of the follow-up experiment, not this review.
- **Regression categories required:** every Promise/async construction and settlement path must show (a) ordinary exceptions still become catchable rejections and (b) an uncatchable stop propagates and terminates. See §4.2.

This is a **design-level hypothesis**, deliberately not a production patch. The project does not patch or upgrade the engine in this task.

## 4.2 Regression matrix for a future fix

Minimum matrix, each in two variants — *ordinary exception must stay a catchable rejection* and *uncatchable stop (gate stop, deadline, cancel, resource) must propagate and terminate with no source continuation*:

- Promise constructor executor throw;
- thenable `then` getter throw (direct and via `Promise.resolve`);
- `await` of a thenable;
- `Promise.all` iterator/element;
- `Promise.race` (if applicable in the revision);
- `Promise.try` (if applicable);
- async function body throw and `await` point;
- async generator resumption and completed-return;
- reaction jobs (regression guard: the already-correct site must stay correct);
- nested Promise resolution / chained `.then`;
- the RegExp admission gate stop specifically;
- each host stop reason (deadline, cancel, resource) delivered inside each of the above.

Two invariants gate acceptance: **no ordinary catchable exception becomes uncatchable** (no over-propagation), and **no uncatchable engine stop becomes a rejection** (no under-propagation). The admission spike's `promise_machinery_swallows_*` and `stop_is_uncatchable_*` tests are the seed oracles. This task does not implement the matrix; the existing probes already answer the design uncertainty (the guard exists and works in the reaction job; the siblings lack it).

## 4.3 Upstream / version evidence

Under the local-only constraint no remote upstream history, release notes or newer source were consulted. Locally available evidence:

- Only the pinned QuickJS-NG 0.16.2 (`0fdea21…`) source is present. No newer version is in the local registry to diff against.
- Within the pinned source, the fix pattern is **already present** at `promise_reaction_job` (55584) and the async-function/interpreter sites, and **absent** at the conversion siblings. This asymmetry suggests the guard was added incrementally and the sibling sites are a plausible, narrowly-scoped upstream target — but whether a **published** later revision already adds it cannot be asserted without remote history.

Therefore: **no concrete upstream fix is evidenced here.** Determining commit/version, exact sites changed, coverage of all swallowing routes, and whether it precedes or follows `0fdea21…` requires a remote-permitted review. If such a fix exists and is complete, adopting it would be a **version-evaluation** task (still gated by the project's no-upgrade rule until authorised); if it does not exist upstream, restoring in-process termination would require a **maintained local fork** of the vendored C, with the maintenance and re-audit cost the resource review already flagged. Which of the two applies is the first question of the next task.

# Part 5 — Option comparison

| Option | RFC change? | Engine fork? | Desktop | Android | iOS/iPadOS/tvOS | Hard termination | Main cost / risk |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Current pinned QuickJS in-process | No | No | Partial | Partial | Partial | **No** — RegExp compile non-polling; Promise swallow lets source continue after a latched stop | Cannot meet §§5/7/10 for these paths; unsafe to claim JS execution |
| QuickJS + process containment | No | No | Yes | Yes (isolated-process) | **No** (no spawnable helpers) | Yes, where available (parent kill) | IPC for every §6 value and service call; per-invocation process/pool lifecycle and latency; **excludes Apple mobile/TV** |
| QuickJS + upstream/fixed version | No | No (if released) | Yes | Yes | **Yes** (in-process, interpreter) | Yes, if the fix covers all swallow sites and pre-admission covers compile | No local evidence such a version exists; needs remote review; upgrade not authorised |
| QuickJS + maintained local patch | No | **Yes** (vendored C fork) | Yes | Yes | **Yes** (in-process) | Yes, if the patch is complete | Fork maintenance and re-audit on every engine change; must enumerate all conversion sites; patch not authorised |
| Another engine | No | n/a | Depends | Depends | Depends | Depends | Full re-evaluation; discards accumulated QuickJS feasibility evidence; premature to select |

Conclusion the table supports: containment is the only option that needs **no engine change** yet delivers hard termination — but it structurally fails the Apple mobile/TV requirement. The two in-process options that *would* serve all platforms (upstream version or local patch) both rest on the **same bounded fix** identified in Part 4, whose portable pay-off is exactly why it is worth a focused feasibility experiment before any engine selection. This makes the next consolidated engine review evidence-based rather than speculative.

# Part 6 — Decision

**Outcome C — an engine-level fix is the more plausible portable path.**

- Process containment restores hard termination and maps cleanly onto Model B, §6 transfer, host-service injection and trusted classification, but it is **not universally deployable**: it cannot ship in ordinary iOS/iPadOS/tvOS App Store applications, which the charter targets. So Outcome A is excluded and a containment-only architecture (Outcome B's framing) would force dropping or differentiating Apple mobile/TV JS claims.
- The remaining in-process termination blocker (Promise/async stop-swallowing) is plausibly **one small, enumerable engine defect** whose fix pattern already exists in the same file (`promise_reaction_job`). Combined with the already-working in-process RegExp pre-admission, a bounded fix would restore §§5/7/10 termination **in process on every target platform**, including Apple mobile/TV where containment is impossible.
- Therefore the more plausible portable path is a narrowly scoped engine-fix/version feasibility experiment, not a containment build. This review authorises neither remedy and selects no engine.

Outcome D is not reached: a credible bounded remedy is identified, so QuickJS need not enter selection review as a blocked candidate yet. Outcome B is not selected because conceding the Apple mobile/TV scope is a larger charter/contract compromise than first testing a bounded fix that the source itself suggests.

## Smallest next task

A **narrowly scoped engine-fix / version feasibility experiment** on the pinned revision:

1. Exhaustively enumerate the Promise/async exception-to-rejection conversion sites in `quickjs.c` and confirm which omit the `JS_IsUncatchableError` guard.
2. As a throwaway local experiment only (not a production patch, not committed to the engine), test the hypothesis that replicating/centralising the existing reaction-job guard makes each admission-spike swallow shape terminate, while ordinary exceptions stay catchable — driven by the existing seed oracles and the §4.2 matrix.
3. Under a remote-permitted follow-up, determine whether a published QuickJS-NG version already carries the fix (commit/version, sites, coverage, order relative to `0fdea21…`), deciding between a version evaluation and a maintained fork.

It must not select an engine, patch or upgrade the pinned dependency in the tree, edit the RFC, implement IPC, or ship platform code. If the fix proves unbounded or incomplete, escalate to a consolidated JavaScript engine-selection review (with containment as the desktop/Android fallback and a host-profile scope decision for Apple platforms).

## Remaining uncertainties

- Confident enumeration of **all** conversion sites needs the exhaustive pass of the next task; the table here is the reachable set from source reading plus three spike-confirmed sites.
- Whether a bounded fix yields "hard" termination depends on there being no *other* non-polling native path beyond RegExp compile; none is demonstrated, but none is disproven.
- Upstream availability of the fix is undetermined under the local-only constraint.
- Platform classifications are established reviewer knowledge pending authoritative vendor confirmation; Android's isolated-process path and Windows Job Objects need a small architecture spike before being claimed.
- Containment's IPC latency, spawn cost and pool-reuse safety on the terminating paths are unquantified (no implementation in scope).

## Validation and audit

Both blockers still reproduce, and the swallow finding re-confirms:

- `python3 experiments/quickjs-resource-spike/probe_regexp.py` → **exit 2** (1,000 / 4,000 / 16,000 refs returned with zero callbacks; 64,000 refs stopped only by the external five-second safety kill, which is not engine evidence).
- The 11 resource tests pass unchanged.
- The 16 admission-spike tests pass unchanged, including `promise_machinery_swallows_gate_and_deadline_stops_in_process` (1,000 iterations complete after ~1,002 callbacks) and `await_resolution_ignores_the_global_promise_binding`.

Documentation-only change: no crate was created or modified, so there is no new fmt/Clippy surface; the runtime and Python suites and the full Phase 3 experiment set are unaffected and were spot-checked. Exact commands and results are in the [completed plan](../plans/completed/2026-09-29-process-containment-and-termination-review.md).

**Audit:** no production `runtime/`, `spec/`, `conformance/`, `examples/`, `docs/decisions/` or experiment change; pins unchanged; RFC 0001 remains proposed; no engine or parser selected; no ADR. Nothing pushed or published.
