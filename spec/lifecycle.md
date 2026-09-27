# Lifecycle

Status: v0.1 draft. The lifecycle describes runtime responsibilities, not additional source operations. There are no public `init`, `login`, `update`, or `dispose` source methods in v0.1.

```text
Unloaded → Validate → Check services and permissions → Ready
                                                    ↓ ↑
                                                     Call
                                                    ↓
                                                  Dispose
```

Validation or loading failure returns to Unloaded. Disposal ends the instance; using it again requires a fresh load. The diagram is conceptual and does not require an application UI.

## 1. Validate without executing

The runtime MUST parse `manifest.json`, reject duplicate member names, validate it against the [schema](./schema/manifest.schema.json), and perform the [semantic checks](./README.md). It MUST verify the exact `specVersion`, engine support, and containment and existence of `entry` before executing source code. Unknown versions MUST NOT be guessed or silently downgraded.

The runtime MUST enforce finite package and entry size limits. It MUST NOT execute an invalid source to discover whether it can be made compatible. Loading a declarative entry additionally requires validating all stored data and references specified by the [Source API](./source-api.md).

Load failures MUST produce a diagnostic distinguishing an invalid manifest, unsupported version, unsupported engine, unavailable required host service, invalid entry, and resource limit. The format of these host-facing diagnostics is not a source operation result and remains implementation-defined. Diagnostics MUST NOT leak secrets.

## 2. Establish the execution context

Before entry evaluation, the runtime MUST resolve required services and establish effective permission grants under [Host API](./host-api.md) rules. It MUST create isolated source state. Any entry evaluation MUST run under the same limits and permission boundaries as ordinary calls.

A declarative entry becomes ready after validation; it has no initialization behavior. JavaScript initialization, module loading, export discovery, and host-service injection require a future execution binding. A runtime MUST NOT advertise that unfinished binding as complete v0.1 JavaScript conformance.

## 3. Invoke an operation

For each call, the runtime MUST:

1. Verify that the instance is ready and the operation is declared. An undeclared operation fails with `UNSUPPORTED_OPERATION` before source execution.
2. Validate input and apply only documented defaults. Invalid input fails with `INVALID_ARGUMENT`.
3. Execute within current grants and limits, observing cancellation.
4. Validate the returned envelope, data shape, IDs where required, and URL grants. Invalid structure fails with `INVALID_RESULT`; denied URL access fails with `PERMISSION_DENIED`.
5. Deliver exactly one result, suppressing late or duplicate completions.

Callers MUST NOT be required to invoke `home` before `search`, `detail`, or `play`. Each operation MUST accept valid IDs independently of a particular call order. Source code SHOULD NOT depend on transient instance memory for IDs to remain usable across reloads. Resources may still expire and be resolved again using `play`.

For v0.1, calls on one instance MUST be serialized. Different instances MAY execute concurrently, but MUST NOT share mutable in-memory source state. Persistent storage and cookies may be shared within the same host-managed source installation; hosts MUST isolate them from other sources and document concurrent storage behavior. Multi-call transactions are not specified.

## 4. Cancel, time out, or fail

A caller MAY cancel a call. The host MUST enforce a finite deadline. A cancelled call yields `CANCELLED`; an expired deadline yields `TIMEOUT`. When completion and cancellation race, the runtime MUST choose one terminal outcome and discard later outcomes.

Cancellation MUST stop further source access and terminate or detach outstanding work so no late result reaches the caller. It does not roll back a completed HTTP request, cookie change, or storage write. Hosts MUST NOT automatically retry a failed call: source calls may have performed external work. A caller MAY explicitly retry under its own policy.

An ordinary source failure does not require unloading a healthy instance. If an execution context cannot safely continue after cancellation, timeout, or resource exhaustion, the runtime MUST dispose it. Subsequent calls require a fresh load.

## 5. Dispose

On unload, update, or host shutdown, the runtime MUST cancel outstanding calls, release transient resources, and prevent further execution in that instance. No source cleanup callback is required. Disposal MUST NOT implicitly erase persistent storage; retention follows documented host policy.

An updated manifest MUST pass the complete loading process again. Permission changes MUST be reevaluated; an older grant MUST NOT silently authorize newly requested origins or resources. Installation, distribution, signing, and update discovery are outside this draft.
