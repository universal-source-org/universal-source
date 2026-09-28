# RFC 0001: Portable JavaScript execution binding

- Status: **Draft; internally reviewed, not accepted or implemented**
- Authors: Universal Source contributors
- Created: 2026-09-28
- Target specification: experimental v0.1 draft
- Discussion: local design unit; no issue or PR created
- Supersedes / superseded by: none
- Initial draft baseline: `e3876c0aa139ff8f518b24436d6b827e47ac72dd`
- Lifecycle revision baseline: `f4029bf64f74ea8ee94cf0fba2c8cd0d5985a016`; [F1 decision review](../../docs/reviews/2026-09-28-javascript-lifecycle-decision.md)

## Summary

Propose a restricted ES module binding for the existing five Source API operations. The entry exports named functions, receives copied input and an explicit invocation context, and returns the **complete Source API envelope (Model B)** synchronously or through a Promise. Initialization is synchronous module evaluation, with no initialization export. Each executing call receives a fresh realm and module state; the logical loaded instance retains only its snapshot and host-side configuration/lifecycle state. Capabilities are invocation-scoped objects passed in the context, never ambient globals.

Requirement words below define the proposed binding if adopted. This RFC does **not** amend the [Source API](../source-api.md), [Host API](../host-api.md), [lifecycle](../lifecycle.md), [compatibility rules](../compatibility.md), or [manifest schema](../schema/manifest.schema.json) merely by existing. A later acceptance change must integrate the binding into those documents and coordinate conformance material. The current reference runtime still rejects JavaScript execution. This document chooses portable behavior; remaining feasibility evidence is not permission for implementations to choose different binding semantics.

## Motivation and scope

The manifest already recognizes `javascript`, but independent runtimes cannot infer entry exports, return wrapping, or Promise behavior from that name. The existing minimal declarative source demonstrates the actual data needs: home lists, paged queries, independent detail lookup and resource resolution. JavaScript should compute those same values without changing their meaning. No live-site requirement justifies an SDK, package manager, extra operation or new portable value type here.

This unit specifies the execution boundary only. Engine choice, engine integration, callable host-service signatures, HTTP/DNS/redirect algorithms, cookie/storage profiles, selectors, crypto profiles, media consumers, distribution, adapters, platform proof, dynamic-origin expansion and WASM remain outside scope. The [Phase 2 closure](../../docs/reviews/2026-09-28-phase-2-closure-review.md) remains valid for the unchanged static runtime.

## Proposed contract

### 1. Package and module model

A JavaScript source package is the existing host-supplied source root containing `manifest.json`, its declared `.js` entry, and the transitively imported local `.js` modules. Only the manifest and reachable module bytes are consumed by this binding; unrelated files are not automatically imported. Archive/distribution formats and installation are deferred. Source code MUST NOT read filesystem paths or enumerate the root.

1. The runtime MUST validate the manifest, exact version/engine support, required services, and entry containment before evaluation. `manifest.entry` is the sole entry selector. No `package.json`, `main`, index file, extension inference, shebang execution, script tag, or command-line argument participates.
2. Modules MUST be UTF-8 ECMAScript modules using the [ECMAScript 2023](https://262.ecma-international.org/14.0/) language baseline, restricted below. There is no CommonJS (`require`, `module`, `exports`), npm resolution, browser import map, bundler transform, TypeScript transform, or platform-specific module wrapper. Strict module semantics apply. A source using syntax outside the baseline is not portable and MUST be refused as an invalid entry by this profile, even if an engine supports it.
3. Static imports and named re-exports in helper modules MAY use only `./` followed by a path matching the manifest's segment grammar and ending in `.js`. For example, `./helpers/ids.js` resolves relative to the importing module's resolved, contained package-relative directory. Helper modules may export named declarations or named export lists; default and star exports are excluded throughout the graph. Parent segments (`../`), additional dot segments, absolute paths, backslashes, percent escapes, query/fragment parts, URL schemes, bare specifiers and remote imports are forbidden. The deliberate absence of parent traversal keeps this first model small; shared helpers can be located below their importer.
4. Resolution MUST use exact case-sensitive package path spelling, with no platform case folding or implicit normalization. Each resolved regular file, including symlinks, MUST remain within the source root. Contained symlinks are allowed; module identity is its canonical contained package-relative path within the loaded snapshot, so symlink aliases do not evaluate twice. Distinct canonical paths, including hard links, are distinct modules; no portable inode identity is assumed. Filesystem names that cannot satisfy exact spelling are invalid entries. Package preparation must supply a stable tree or snapshot; this does not claim a hostile concurrent-writer filesystem sandbox.
5. The runtime MUST resolve and parse the complete reachable graph before evaluating any module, apply finite byte/module-count/depth budgets, and reject missing files, forbidden syntax/specifiers, unresolved/ambiguous exports, and cycles as invalid entries. Cycle detection uses resolved module identity, including symlink aliases. Valid acyclic dependencies evaluate once per realm in ECMAScript dependency order, both in load validation and in each executing invocation (§4). No recursive lazy discovery at invocation time is permitted.
6. Dynamic `import()` and `import.meta` are forbidden syntax anywhere in the graph, including unreachable branches. Top-level `await` is forbidden; `await` inside an async function is allowed. Import attributes/assertions, JSON modules and native modules are not supported. Entry re-exports and default exports are excluded by the export contract below.
7. Module text is snapshotted for the instance. Updating files cannot change an existing instance; a replacement package requires the complete load/grant process again. Immutable parsed code may be cached across instances and invocations, but module environments, functions, intrinsics and mutable values MUST NOT be shared.

This adds no imports that grant host authority. It intentionally defers remote code, package dependencies, cycle support and richer module resolution. Supporting those later requires an explicit binding/versioning review, not an engine option.

### 2. Export and invocation contract

The entry module's export names MUST exactly equal `capabilities.operations`, a nonempty subset of `home`, `category`, `search`, `detail`, `play`. Each MUST be declared directly as `export function NAME(input, context)` or `export async function NAME(input, context)`; parameter names and unused parameters are not significant. Destructuring/default/rest parameters use ordinary language semantics; the runtime always supplies exactly two arguments. Generator functions, async generators, classes, exported variables, default/object APIs, re-exports, aliases and additional exports are invalid entries. Helpers can be unexported entry declarations or named exports in imported helper modules; helper exports do not become source operations.

There is no `init`, `initialize`, `dispose`, default factory or secondary operation object. Duplicate exports are an invalid-entry parse/link failure. A missing declared operation or invalid export form/type prevents loading, rather than producing an operation error later. An operation omitted both from the manifest and entry is valid; attempting to call it yields `UNSUPPORTED_OPERATION` before source execution or input validation. An exported undeclared operation prevents loading. This avoids two competing discovery surfaces.

At module instantiation in each realm, the runtime MUST capture the directly declared operation functions. After evaluation, each entry export MUST still hold that same function value; this catches top-level replacement with a nonfunction or another API surface. Failure prevents loading in the load-validation realm; per-call initialization failure follows §4 and §7. Later assignment during an invocation does not change captured dispatch. At each ready call it MUST check declarations, validate input under the Source API, apply only the specified page default, initialize a fresh realm under §4, copy the input into that realm, and call its captured function with `this` equal to `undefined` and arguments `(input, context)`. No arity-based overloading is defined. Source code cannot invoke the host dispatcher through the context.

Every existing operation retains its exact input, success shape, page/ID constraints and errors. JavaScript computes a page, including its `page` and `hasMore`; static declarative lookup/pagination rules do not apply to JavaScript. The runtime validates all actual results, including returned-origin grants, but does not enumerate all possible dynamic IDs by executing operations during loading. Source authors still owe the Source API's cross-operation ID consistency and call-order independence.

### 3. Return model decision

**Choose Model B: full envelope return. Model A MUST NOT be offered as an alternative mode, guessed from result shape, or selected by individual runtimes.**

| Criterion | Model A: data only | Model B: complete envelope (selected) |
| --- | --- | --- |
| Existing Source API | Runtime wraps success; expected failures need another binding mechanism. Mirrors static stored data, not the logical call boundary. | Directly expresses the existing logical success/failure result. Declarative storage stays data-only. |
| Error propagation | `NOT_FOUND` and other intentional failures require a new branded throw/helper or separate return exception. | Intentional failure uses the existing error envelope; unexpected throws/rejections use the error boundary below. |
| Invalid return detection | Schema-check data, then wrap; accepting envelopes as well would be ambiguous. | Validate one closed envelope, then its data/error. Data-only returns are `INVALID_RESULT`. |
| Portability | Portable if an additional explicit failure mechanism is standardized. | One existing vocabulary and one transfer rule; no author-visible exception constructor is needed. |
| Implementation complexity | Adds wrapping and intentional-failure bridging. | Reuses envelope validation; requires no success wrapping heuristic. Both need validation and interruption override. |
| Author ergonomics | Shorter success paths, but a different idiom for expected failures. | Slightly more success boilerplate; success and expected failure remain plain objects. Helpers may be local functions. |
| Ownership | Runtime constructs successes and controls validation/publication. | Source proposes an outcome; runtime still owns validation, grants, limits, confidentiality and exactly-one publication. |

The runtime MUST NOT trust `ok: true` as validation or authority. It MUST reject unknown envelope/data/error fields and invalid codes with `INVALID_RESULT`. A valid source failure envelope is an intentional outcome, not a thrown exception. The runtime MUST enforce host denials independently: returning success cannot authorize a denied action. A source can recover from a failed service call and return unrelated permitted data; it cannot turn the denied action into a performed action. Observed invocation cancellation/deadline/resource termination cannot be caught and converted into a successful publication.

### 4. Initialization, state and disposal

**Choose lifecycle Model B: fresh realm per invocation.** This is separate from §3's full-envelope return Model B. A *logical loaded instance* holds the immutable module snapshot, declarations, host configuration and serialization/lifecycle state. An *execution realm* holds JavaScript module/global/intrinsic state, captured functions and its source job queue. Reusing the logical instance MUST NOT reuse any mutable JavaScript state.

Loading follows manifest/complete-graph validation and service/grant setup. The runtime MUST create a disposable load-validation realm, instantiate/capture the operation declarations, evaluate the snapshot and verify export identity. Evaluation of the graph is **the entire initialization step**; there is no separate source initialization callback. Evaluation receives no arguments, context, manifest object, filesystem path or capabilities. Its completion value is ignored, not transferred as Source API data. After successful validation, this realm MUST be destroyed before the logical instance becomes ready; no JavaScript value or captured function survives from it.

Initialization MUST NOT be asynchronous. Top-level await is rejected before execution; if evaluation queues any source Promise job (excluding the engine's internal module-loader bookkeeping), initialization MUST fail without running that job. Creating an inert Promise that schedules no job is not itself asynchronous initialization. In the load-validation realm, evaluation exceptions, queued initialization jobs and export-identity failures are invalid-entry load diagnostics. Loading deadlines/cancellation and resource failures remain host-facing diagnostics, never Source API envelopes; no instance becomes ready after a failed load. Evaluation MUST be bounded under the same execution/authority policy as calls.

For each declared, input-valid ready call that reaches execution, the runtime MUST create a **fresh realm**, instantiate/capture fresh operation functions and synchronously evaluate the same snapshot under the initialization rules above before calling the operation. No files are reread or graph discovery deferred. Per-call initialization consumes the invocation deadline and execution/resource budget. An ordinary initialization exception, queued initialization job or export-identity failure yields sanitized `SOURCE_ERROR` without calling the operation; observed interruption and checked resource exhaustion retain their classified outcomes. This is a ready-call failure, not a second loading diagnostic. Invalid input, undeclared operations and queued/pre-execution rejection execute no initialization.

Mutable module variables, source-added globals and intrinsic mutations last only within that invocation, including its allowed async continuations. They MUST reset for the next call, even after synchronous success or ordinary failure. A module-scope `Map` can cache within a call but never supplies a portable cross-call cache. Immutable source/parsed-code caches are permitted only without mutable source values. Inputs/results are copied; no Promise, function, wrapper or engine handle may cross calls. Persistent host storage is separate, requires its own profile/permission, and is neither supplied nor erased by realm lifetime. No source may require `home` first, and usable IDs must not depend on an earlier call.

Every execution realm MUST be retired on every terminal outcome as specified in §5. The logical instance remains reusable after ordinary success, returned failure, invalid result or source exception, but the next call always gets new JavaScript state. Only one invocation may execute per logical instance, including initialization, async continuations, non-executing transfer and cleanup through publication. The binding conservatively still requires **logical instance disposal** after an executing invocation is cancelled, expires or hits an execution resource limit; initialization counts as execution. Waiting calls cannot then execute (U1 reporting remains host-defined). Cancellation/expiration while merely queued does not disturb the active invocation or dispose the logical instance. Checked pre-execution input/admission limit rejection also leaves it healthy. This does not change Phase 2's reusable immutable static contexts.

Realm retirement closes scheduling/delivery, revokes scopes and releases or permanently neutralizes all source-executable roots, jobs, resolvers and transient service handles. It MUST NOT invoke source cleanup, Promise handlers/resolvers or `finally` code. Physical reclamation may be deferred only when no source execution or later source observation remains possible. Disposal cannot be reported complete while source code can still access the realm; interrupted execution may need cooperative quiescence. Logical instance disposal additionally prevents future calls; the host may retain a diagnostic object only. Reload requires a new instance and complete validation/grant setup. Neither form of disposal erases persistent storage.

### 5. Completion, jobs, cancellation and deadlines

An operation MAY return a synchronous value or an intrinsic Promise from its invocation realm. A fulfilled operation Promise supplies its fulfillment value as the proposed full envelope; rejection follows the error table. The return slot accepts a Promise with the invocation realm's original `Promise.prototype`, including an async function's Promise; Promises with a different prototype (including ordinary subclass instances) and cross-realm/native handles are not accepted. This is a current-prototype check, not a claim to track construction history; source mutations cannot forge the engine's actual Promise identity. Internal settlement observation MUST NOT call an overridden `then` method. Arbitrary thenables are not assimilated by the runtime: a directly returned thenable is an invalid result and its `then` MUST NOT be called by boundary conversion. Ordinary ECMAScript Promise resolution inside source code may itself assimilate a thenable; that is source execution under the same limits. Nested Promises in result data are invalid portable values.

The runtime MUST schedule Promise jobs FIFO within an invocation, preserving language run-to-completion and dependency ordering. There are no timers, event loop APIs, workers or source callbacks from arbitrary host threads. Host service settlements enqueue work only onto their still-active owning invocation; a closed delivery gate MUST NOT invoke a JavaScript resolver or enqueue executable work. Relative arrival order of independent external service completions is not deterministic; sources needing an order must explicitly await it. Hosts MUST NOT pump another invocation's jobs in the same instance while awaiting this one.

A synchronous return becomes a candidate after the function unwinds. For a returned Promise, settlement becomes a candidate when the Promise settles, but never before the exported function itself has unwound. At the next source-job boundary, **before another queued source job**, the runtime MUST close source-job/service-delivery gates and revoke the scope, perform bounded non-executing candidate copying/validation or failure classification, and retire the entire execution realm under §4. Revoking authority does not erase the private classification of an already produced same-invocation host rejection. After cleanup and an immediate pre-publication interruption check, it selects and publishes exactly one host-owned envelope. The invocation is complete only after source quiescence, revocation and realm retirement are established and publication occurs; serialization covers all these steps. Terminal selection is not an irrevocable success decision made before cleanup/deadline checks. Observed interruption without a candidate also requires source quiescence and retirement before completion.

Jobs already queued but not started receive **no final drain**, including during copying/cleanup. Settlement observation MUST NOT introduce an extra `.then()` turn. Neither conversion nor cleanup may invoke source code. A Promise settling during a running job does not cut short that job; if execution continues indefinitely, cooperative interruption can still win. If termination was observed, the candidate is discarded. Rejection of an unrelated, unreturned Promise does not replace the operation's outcome; it may produce only confidential host diagnostics, not Node/browser unhandled-rejection behavior.

A never-settling returned Promise is pending until caller cancellation or the finite host deadline; it MUST NOT be interpreted as `null`, success, or a missing return. Pending Promise waits must leave the scheduler able to observe cancellation/deadlines even if no source job arrives. Deadlines use host monotonic time starting at invocation admission, including queueing. The host checks before initialization/entry, during initialization under its checkpoint policy, between jobs, during bounded transfer/validation and host cleanup, at service boundaries and immediately before publication; an expired result MUST NOT be accepted. No source clock is supplied.

Cancellation is **cooperative**: request, observation at a checkpoint, authority revocation, then quiescence. No source-visible cancellation callback or `AbortSignal` is added. Hosts must document checkpoint policy, including how running source code reaches checks (engine hooks, bounded execution slices, or instrumentation are possible implementation mechanisms). Merely checking host-service calls is insufficient for arbitrary CPU-only loops. A future runtime must demonstrate those checks before claiming execution support. Finite deadline enforcement means no late result acceptance and eventual interruption under that policy; it is not a hard real-time completion-latency guarantee. Hard preemption, precise instruction interruption and cross-platform enforcement are **not claimed**.

At that boundary, all remaining work is abandoned **with its whole realm**, not by substituting callback return values or selectively cancelling Promises. Unresolved Promises, `.then` children and async-function results stay in their last pending/settled states until released; no synthetic fulfillment or rejection is performed. Queued reactions are discarded without running their handlers, child resolvers or species code. Not-yet-queued reactions and nested await continuations lose all executable paths when their upstream/resolver graph is retired. Source `finally` is not cleanup. Module/global references disappear with the realm, and JSON-only transfer prevents any later call from obtaining an old Promise or registering a new reaction on it. Ordinary Promise/await semantics apply while source execution is allowed; there is no generic Promise cancellation primitive.

Pending services are cancelled or detached; late native completions MUST NOT call old JavaScript resolvers, mutate source state or deliver into another invocation. Teardown is bounded host work under documented resource/checkpoint policy, not a wait for detached external I/O or a source-job drain. Source background tasks MUST NOT outlive an invocation. Sources must await work whose result/effect they need; completed external effects are not rolled back. An inert retained Promise does not keep a scope alive. Hosts MUST NOT retry automatically.

A completion/cancellation race chooses exactly one outcome; cancellation after publication cannot change the delivered result. Simultaneous cancellation, deadline, resource and other faults remain documented D1 host policy, not a new portable total ordering. Future fixtures isolate events or assert exactly-one/suppression invariants rather than invent a universal race winner.

### 6. Value boundary

The existing JSON-compatible Source API domain remains unchanged. Inputs are validated before source execution; outputs are validated before publication. Binding conversion MUST be a bounded, non-executing structural copy, not `JSON.stringify`/`JSON.parse` on untrusted objects. It MUST NOT invoke getters, `toJSON`, `valueOf`, iterators or arbitrary coercions. Runtime checks use trusted intrinsics/internal metadata, unaffected by source mutation of globals or prototypes.

| Value | Proposed transfer rule |
| --- | --- |
| `null`, boolean | Copy exactly; operation schemas still decide where `null` is legal. Missing differs from `null`. |
| Finite number | JavaScript `Number` (binary64); no string coercion. All current valid operation input numbers are safe integer pages. Preserve that range before conversion, with no rounding an invalid page into validity. At the JSON boundary negative zero is normalized to zero. |
| String | Preserve Unicode text and keys without trimming/normalizing/case folding; reject unpaired UTF-16 surrogates, including keys. |
| Array | Dense ordinary arrays with the invocation realm's original `Array.prototype`; copy indices in order. Holes, extra own properties, symbols or accessor elements are rejected. The built-in `length` property is allowed; nested values recurse. |
| Object/map | Ordinary objects with the original `Object.prototype` or null prototype; own enumerable string-keyed data properties only, recursively copied. Reject accessor/non-enumerable/symbol properties and custom prototypes. A JavaScript `Map` is not a portable object/map. |
| `undefined` | Reject anywhere, including a missing return, own object member or array element; never omit it or convert it to `null`. |
| `NaN`, infinities | Reject, never stringify to `null`. |
| `bigint`, symbol, function | Reject, never stringify or convert. They may be internal computation values. |
| Class instances, boxed primitives, dates, `Map`/`Set`, RegExp, Error, Promise in data | Reject; class instances fail the custom-prototype rule. No automatic property flattening, date formatting or exception-object transfer. |
| Typed arrays, `ArrayBuffer`, shared buffers, native handles | Reject; **no binary Source API value type**. Future service encodings cannot amend this by implication. |
| Cyclic objects | Reject. Repeated acyclic references are copied independently, with no identity preservation. Bound total expanded size as well as depth. |

Plain-record recognition is structural, not construction-history tracking: an ordinary class-created object explicitly changed to an allowed prototype and property shape is treated as a record; only its specified own data properties transfer, never private state or class behavior. Intrinsic/native objects (dates, buffers, promises, errors, service wrappers and similar branded objects) remain rejected even if their prototypes are changed.

Incoming JSON objects become ordinary objects with the invocation realm's original `Object.prototype`, defined with own data properties; keys such as `__proto__`, `constructor` and `toString` MUST be data, not setters or prototype operations. Arrays use the original array prototype. Object member insertion order on transfer into JavaScript MUST be ascending Unicode scalar-value order of keys; JavaScript's own integer-index enumeration rules still apply. Outgoing object order is not a Source API semantic; array order is. Numeric serialization must round-trip the finite binary64 value; exact byte spelling is not promised. This does not impose JavaScript numeric rounding on declarative JSON or enlarge the valid operation schemas.

Invalid caller values/shape produce `INVALID_ARGUMENT` without execution; invalid operation return values/shape produce `INVALID_RESULT`. Excessive traversal/size yields `RESOURCE_LIMIT`, not truncation. JSON duplicate member rejection still applies before values become objects. Valid JSON-compatible values can still fail their operation schema (for example a `null` detail or unknown field).

Context/service wrappers and their callable functions are a **control boundary**, not additions to transferable data. Their opaque identity cannot be returned, stored through JSON storage, or passed as ordinary service data. This RFC defines no portable binary type or arbitrary engine-native object representation; either would need a separate Source API/versioning proposal.

### 7. Error boundary

The following maps isolated failures. Error messages delivered to callers MUST be nonempty, confidential diagnostics. Runtime-produced errors use sanitized category text, never raw exceptions, stack traces, filenames, host-native objects or credentials. A source-supplied valid failure message is diagnostic only; source authors owe the confidentiality rule and hosts may redact it. Conformance compares codes and safe message structure, not wording.

| Event | Required boundary outcome |
| --- | --- |
| Invalid manifest/version/unsupported engine/unavailable required service | Distinct existing host-facing load diagnostic; no source execution, no Source API envelope. |
| Module decode/parse/link/containment/cycle failure; missing/invalid exports | Invalid-entry load diagnostic. |
| Exception, queued initialization job or export-identity failure during load-validation evaluation | Invalid-entry load diagnostic; discard validation realm, no ready instance. |
| Same initialization failures during a ready call's fresh evaluation | Sanitized `SOURCE_ERROR`; do not call operation, discard realm. Interruption/checked limits retain their rows below. |
| Loading cancellation/deadline or internal loading fault | Host-facing diagnostic distinguishing interruption/internal failure from a valid ready instance; representation remains host-defined. No operation error code is standardized for loading. |
| Loading structural/engine budget exceeded | Resource-limit load diagnostic. |
| Undeclared operation; invalid input | `UNSUPPORTED_OPERATION`; `INVALID_ARGUMENT`, in the existing declaration-before-input order. |
| Valid source failure envelope | Its existing Source API code; validate closed shape/code/message. |
| Ordinary thrown value or operation Promise rejection | `SOURCE_ERROR`, regardless of a forged `code`, `message`, Error name or stack. Throwing a failure envelope is not returning it. |
| Invalid envelope, success data, failure code or portable value | `INVALID_RESULT`; no coercion, wrapping or partial success. |
| Uncaught authentic host-service rejection in the active invocation | Preserve its classified existing code, including `INVALID_ARGUMENT`, `PERMISSION_DENIED`, `NETWORK_ERROR`, request `TIMEOUT` or `RESOURCE_LIMIT`. This is a classified host failure, not an unclassified source exception. |
| Valid returned resource/poster outside requests or current grants | `PERMISSION_DENIED`, without fetching or stripping the URL. |
| Observed invocation cancellation/deadline | `CANCELLED` / `TIMEOUT`, once only; discard candidate and dispose the logical instance if execution (including initialization) began. |
| Checked invocation/engine resource exhaustion | `RESOURCE_LIMIT`; dispose the logical instance if execution began, preserve a healthy instance for pre-execution rejection. |
| Recoverable internal execution implementation failure | Confidential `SOURCE_ERROR`, dispose unsafe logical instance, cancel waiting work. Never invent an eleventh source-call code. |

Fatal process termination, allocation abort and embedding failures that cannot run recovery code are outside a promise of delivered envelopes. An engine stack overflow is `RESOURCE_LIMIT` only when the runtime identifies a documented enforced limit; an otherwise unclassified source exception remains `SOURCE_ERROR`. Engine-specific exception text MUST NOT decide classification.

The runtime privately brands service rejection records and exposes only immutable own `code` and sanitized `message` fields, with a null prototype and no stack. The brand is instance-and-invocation scoped and cannot be forged, copied or gained by matching properties. An unmodified branded rejection propagated through `await` or rethrow in that same invocation retains classification. A copy, wrapper or ordinary thrown object is unclassified and maps to `SOURCE_ERROR`. No old rejection object can reach a later call through this binding; any test-private foreign-scope injection also cannot gain the active scope's classification. A source may inspect the two fields and return a normal explicit failure envelope; the brand never crosses the Source API boundary.

A1 remains the loading/semantic/result distinction: syntax-valid stored text or URLs do not establish permission failure at load. JavaScript results are checked when returned. U1 remains outside ready operations. U2 (malformed result plus denied origin) does not need a winner to define this binding: both must prevent success; combined-fault precedence remains deferred and must be documented locally. No fixture below prescribes that winner.

### 8. Host capability injection and ownership

The second argument is a runtime-created, frozen, null-prototype object with exactly one own enumerable data property, `services`. `context.services` is a frozen, null-prototype registry whose own enumerable properties are exactly the names in `capabilities.host`, in scalar-value key order. It has no fallback resolver or inherited services. Every declared service must be available before evaluation or loading fails. `context.services.http` is therefore either a declared wrapper or absent; an absent property evaluates to ordinary `undefined`. Attempting to call through that `undefined` throws an ordinary exception and, if uncaught, yields `SOURCE_ERROR`; no service was acquired or action attempted. Attempts through an existing wrapper to use undeclared/ungranted resources yield `PERMISSION_DENIED`.

Each registry value is a frozen, null-prototype service wrapper with only own enumerable members of a separately specified service profile; this RFC introduces **no callable members** for any named service. No provider implementation object, native pointer, mutable host map or cross-source namespace is exposed. Services are owned by the host; wrappers belong to a single invocation. Calls use authority from the owning source installation and host user context, never from a caller-supplied source ID or a cached grant.

Method wrappers capture their owning scope and ignore the JavaScript `this` receiver, so extracting a method cannot change authority. The common calling convention for future service methods is asynchronous: each returns an invocation-realm Promise, including immediately available outcomes. Boundary argument failures, denials and service failures reject with the branded record above; a valid HTTP non-2xx status is not by itself a transport rejection. Service arguments are copied and validated before action; results are copied before fulfillment. The default transfer domain is the JSON value rule above. Future profiles must explicitly specify any invocation-local opaque helper handles (for example HTML parser objects already anticipated by the Host API), their ownership/disposal and encodings; such handles remain forbidden in Source API results. This RFC does not supply those profiles or silently introduce binary service results.

Every method use MUST verify that its scope is still active, the service was declared, current permissions still allow the particular action, and applicable limits permit it. Authority is rechecked on use, including revocation during an awaited call; completion delivery cannot revive revoked authority. Returned URLs are separately checked under the existing Host API. No new origins, cookies or storage access are implied by a wrapper's presence. Adapter and indirect-service access obey the same boundary.

A source MAY retain a wrapper, method reference or context within its invocation, including in module memory; none survives into the next call's fresh realm. Closing the delivery gate under §5 revokes every wrapper before transfer/cleanup. Host-side scope checks MUST independently prevent obsolete handles and late native completions from performing new actions, invoking old JavaScript resolvers or acquiring another call's authority. They MUST NOT rebind a wrapper or inject a stale completion/rejection into the next realm. Realm retirement releases those source references; cancellation/detachment and permanent neutralization of native delivery paths are also required. No source code may run between invocation scopes. Detached host cleanup may finish without source callbacks or further authority; already performed effects are not transactional or rolled back.

This binding settles visibility, ownership, async convention, copying, rejection, gating and disposal. Method names, argument/result shapes, HTTP encodings, DNS/redirect validation, cookies, storage concurrency, HTML selectors, JSON duplicate behavior implementation, crypto algorithms, URL parsing and logging signatures remain separate work. No runtime may advertise an interoperable service solely by implementing this registry.

### 9. Globals and dynamic code

The ECMAScript 2023 baseline is a language reference, not a browser or Node environment. The initial global object MUST expose only the following standard names (with the restrictions below):

- `globalThis`, `undefined`, `NaN`, `Infinity`;
- `Object`, `Function`, `Boolean`, `Symbol`, `Number`, `BigInt`, `String`, `RegExp`, `Array`;
- `Error`, `AggregateError`, `EvalError`, `RangeError`, `ReferenceError`, `SyntaxError`, `TypeError`, `URIError`;
- `Math`, `JSON`, `Reflect`, `Map`, `Set`, `WeakMap`, `WeakSet`, `Promise`;
- `eval`, `isFinite`, `isNaN`, `parseFloat`, `parseInt`, `decodeURI`, `decodeURIComponent`, `encodeURI`, `encodeURIComponent`.

Source-added global properties are invocation-local memory, not authority. The runtime MUST expose no other host globals or extension properties on intrinsic objects. `Date`, `Intl`, `Proxy`, `WeakRef`, `FinalizationRegistry`, typed-array/buffer constructors, `SharedArrayBuffer`, `Atomics`, `WebAssembly`, timers, `queueMicrotask`, workers, `fetch`, `XMLHttpRequest`, `URL`, `console`, `window`, `document`, `navigator`, `process`, `Buffer`, `require`, filesystem/environment/native bridge APIs and global services are absent. Internal standard JSON computation does not request the `json` host service and does not claim its stricter callable profile (in particular duplicate-member rejection).

**Dynamic source-text compilation is unsupported.** Calls to direct/indirect `eval`, `Function`, and every equivalent function-family constructor, including those reachable through prototype `.constructor` paths of ordinary/async/generator functions, MUST throw `EvalError` before compiling text. `Reflect.construct` and aliases cannot bypass the restriction. The `Function` intrinsic remains for ordinary function prototype relationships, not code generation. No host service may compile source text, return executable functions as data, or provide an alternate evaluation bridge. Fetched strings remain data. RegExp pattern compilation is allowed as bounded data processing, never JavaScript evaluation; regexp resource behavior must be tested by the engine spike.

Clock/time and randomness are absent: `Math.random` MUST throw `TypeError`; no substitute seeded RNG or implicit wall clock is defined. Locale-sensitive methods `String.prototype.localeCompare`, `toLocaleLowerCase`, `toLocaleUpperCase`, and `toLocaleString` on Object, Array, Number and BigInt prototypes MUST throw `TypeError`. These restricted intrinsic properties and compilation hooks must not be made functional by modifying source-visible objects. Normal deterministic alternatives such as explicit string comparison remain available. Portable sources must not infer timezone, locale or host environment through implementation extensions.

Exceptions caught inside JavaScript may have implementation-dependent diagnostic text; nonstandard properties such as `stack` MUST NOT be exposed by the runtime/engine to sources in this profile. Portable source decisions MUST NOT depend on exception message formatting or function source formatting. These are not portable identifiers or result semantics. This profile deliberately omits observability of garbage collection/finalization and shared-memory concurrency.

### 10. Isolation, resources and portability

No engine is selected. A realm need not be an OS process, but an implementation MUST demonstrate separation of mutable module/global/intrinsic state, jobs, service handles and grants, including source attempts to mutate prototypes, both between logical instances and between consecutive calls. Retiring a realm may require retiring its entire engine runtime/queue domain; a new global object alone is insufficient if old jobs or native roots can still execute. Permission checks and converters use trusted state outside that mutable realm. Bytecode sharing is permissible only when it cannot expose shared mutable source data.

Hosts MUST publish finite bounds for consumed package/module bytes, graph size/depth, input/result bytes/depth, outstanding calls/jobs/service requests, execution deadline/checkpoint policy and engine resource policy. The Phase 2 Rust constants are not JavaScript portable constants. Limits are structural boundaries; accepted input/result size is not an exact JavaScript heap quota. A future implementation needs a credible bound for source-created allocations and CPU work in addition to boundary byte counts. Checked limits yield classified failure; process-wide containment and allocator abort recovery require embedding evidence. No precise heap/RSS accounting, hard real-time preemption or cross-platform enforcement proof is claimed here.

| Portability surface | Decision or explicit limit |
| --- | --- |
| Engine/parser | Use the language baseline and explicit restrictions, not engine extensions. Syntax/load diagnostics compare categories, not parser text. Engine conformance evidence is still needed. |
| Object/property ordering | Deterministic incoming key insertion, standard language enumeration, array order preserved; result object order ignored. No observable host hash-map iteration leaks. |
| Numbers | Binary64 inside JavaScript; valid page inputs already fit the exact safe range. No coercion or arbitrary precision portable type. Engine-approximated transcendental Math results are not exact cross-engine fixtures; source logic requiring identical results must avoid depending on their last bits. |
| Exceptions/stacks | Only existing codes and sanitized diagnostics cross the boundary; no engine-native exception transfer or stack exposure. Message/formatting differences are non-portable. |
| Locale/timezone/time/random | No ambient access; specified restricted methods throw rather than consult host configuration. |
| Environment/filesystem/network | No globals or imports grant access. Host module loading is distinct from source path access; network can enter only through later declared service profiles. |
| Promise jobs | FIFO within a scope, fixed terminal boundary, no work after it; independent external completion arrival is not a promised total order. No browser/Node queue semantics. |
| Parser algorithms | D2's complete URL/IDNA parser profile remains deferred. HTML and language/Unicode edge behavior also need their own profile/evidence; this RFC does not close them or make one library's behavior normative. |

U1 non-ready diagnostics, U2 combined-result precedence, P1 dynamic resource origins, D1 quotas/scheduling/host policies and D2 complete parser portability remain open as in the [issue register](../../docs/reviews/2026-09-28-phase-1-closure-review.md#issue-register-and-dispositions). A1's narrower fixture-validation scope and loading/result distinction remain intact. Selecting a language baseline/restricting authority does not close those broader issues.

## Compatibility and versioning

**Conclusion: adoption would complete intentionally missing JavaScript semantics within the experimental v0.1 draft; no version bump is proposed.** It does not add an engine name, manifest field, source operation, service name, error code or portable value type, and does not alter declarative execution. There is no existing normative JavaScript export/return or mutable-state persistence contract to replace. The lifecycle revision deliberately breaks the earlier unaccepted RFC's proposed cross-call persistence; experimental implementations of that draft must adopt fresh realms and identify their target revision, without a persistent-realm compatibility mode. Recognition in schemas has never guaranteed execution. Draft status still requires explicit acceptance, updated normative cross-links/text, and reports identifying the target Git revision; this RFC alone is not that acceptance.

| Pairing | Required behavior |
| --- | --- |
| Old declarative source / new runtime | Existing semantics and authored outcomes unchanged if the runtime supports declarative; this RFC adds no new requirement to static entry data. |
| Old schema-valid JavaScript source / new runtime | Schema validity alone never established executability. Load only if code satisfies the adopted binding and required service profiles; otherwise a distinct load failure, without guessing a legacy export convention. |
| New JavaScript source / old declarative runtime | Refuse with unsupported-engine load diagnostic; never execute as declarative or fallback to another version. Current Rust loader already refuses `javascript`. |
| Recognizing but nonexecuting runtime | May validate manifest structure, must state execution unsupported and refuse loading. Recognition cannot count as binding support. |
| Runtime implements binding but lacks a required service profile | Unavailable-required-service load failure, before evaluation; no no-op service. |
| Experimental pre-RFC JS runtime | Must identify its earlier revision/private scope. Adopting this proposal may break its experimental assumptions; no silent compatibility promise or second return mode. |
| Legacy source/adapter | Requires explicit translation with reported limits; no implicit CommonJS, browser, npm or privileged native compatibility. |

No schema or version is changed by this design unit. If a later accepted/finalized binding changes incompatibly, the compatibility document governs the version decision. Adding binary values, new service names or module models would require a separate analysis; implementation support alone is not grounds for a bump.

## Minimal examples

These are **informative, unexecuted examples of the proposed rules**, not added fixtures or support claims. `source.js` beside this manifest is a complete minimal package:

```json
{
  "specVersion": "0.1",
  "id": "org.example.javascript",
  "name": "Minimal JavaScript",
  "engine": "javascript",
  "entry": "source.js",
  "permissions": {"network": [], "cookies": false, "storage": false},
  "capabilities": {"operations": ["home"], "host": []}
}
```

```js
// Synchronous module initialization; no init export or host access.
const title = "Demo";

export function home(input, context) {
  return {ok: true, data: {
    categories: [], items: [{id: "demo", title}]
  }};
}
```

For a manifest declaring only `detail`, this async entry illustrates fulfillment and an intentional mapped error. It needs no earlier call:

```js
export async function detail(input, context) {
  if (input.id !== "demo") {
    return {ok: false, error: {code: "NOT_FOUND", message: "Unknown content id"}};
  }
  const title = await Promise.resolve("Demo");
  return {ok: true, data: {id: input.id, title, playables: []}};
}
```

For a manifest declaring `home` and requiring `log`, a wrapper can be inspected through the defined injection boundary. This demonstrates **capability acquisition only**, not a logging method; none is standardized here. Service availability is established before evaluating this entry:

```js
export function home(input, context) {
  const logger = context.services.log;
  // A declared wrapper is present even if the host filters all logs.
  if (logger === undefined) throw new Error("Binding invariant failed");
  return {ok: true, data: {categories: [], items: []}};
}
```

A complete callable capability example must wait for a service-signature RFC. Inventing `logger.info`, `http.fetch` or a generic `invoke` method here would defeat this unit's scope. The wrapper access above uses the only service API this RFC defines.

Each following snippet is a separate invalid `home` entry or failure illustration, not additional exports in one module:

```js
export const home = 42; // Invalid-entry loading diagnostic; never ready.
```

```js
export function home() {
  return {categories: [], items: []}; // INVALID_RESULT: Model A is not accepted.
}
```

```js
export function home() {
  return {ok: true, data: {categories: [], items: undefined}};
  // INVALID_RESULT: undefined is not transferable (nor a valid items array).
}
```

```js
export function home() {
  throw {code: "NOT_FOUND", message: "Not a returned envelope"};
  // SOURCE_ERROR: ordinary thrown objects cannot forge classified failures.
}
```

## Future conformance plan

No JavaScript conformance corpus is implemented by this RFC. The separate non-production engine experiments are feasibility evidence, including counterexamples to earlier draft rules, not production conformance. Before a runtime claims JavaScript execution support, an independently authored deterministic corpus and production-path harness MUST cover the categories below. Start with service-free execution; any declared service support additionally needs its separate profile and cases. Binding-only tests may use explicitly test-private providers to drive the already defined injection/error/lifetime rules; these are not new manifest service names or a public SDK.

Otherwise unreachable values such as dates, typed arrays or foreign-realm objects require isolated converter tests through test-private harness injection, not extra source globals. Use immutable local packages, authored full-envelope expectations, controlled host grants, fake monotonic clocks, scheduled gates and classified test-provider failures. No live network, DNS, real-time sleeps, secret data or engine-generated expected results. Distinguish load/setup diagnostics from ready operation errors. Compare JSON structurally, preserve arrays and missing/null, compare exact codes, check safe messages, and ignore exception wording/stacks. Record engine version/platform/limits and binding revision. Independent-runtime comparison is required before broader interoperability claims.

| Category | Observable assertion required |
| --- | --- |
| Entry discovery | Only manifest entry is selected; no index/package inference. Missing/wrong-case/non-file entry fails before evaluation. |
| Valid exports | Each of the five declared named sync/async forms is discovered; helpers do not add operations. Top-level replacement fails load validation (or per-call initialization under §7); functions are captured anew per realm and dispatch remains fixed after invocation-time export binding reassignment. |
| Missing export | Missing declared function prevents readiness; omitted undeclared operation call yields `UNSUPPORTED_OPERATION` before invalid input. |
| Invalid/ambiguous exports | Nonfunction/class/generator/default/object/alias/re-export/extra/duplicate entry surfaces yield invalid-entry diagnostics. |
| Synchronous success | Full envelope validated and published once with operation-specific shape, page defaults, IDs, no call-order prerequisite. |
| Promise success | Async and direct original-prototype Promise fulfillment use exactly the same envelope rules; internal standard assimilation works. |
| Throws | Error, primitive and forged code/envelope throws yield sanitized `SOURCE_ERROR`. |
| Rejections | Ordinary operation rejection yields `SOURCE_ERROR`; classified same-scope service rejection preserves its code; copies and test-private foreign-scope records do not; no source path can carry an old object to another call. |
| Invalid result | Data-only Model A, missing return, envelope conflict, bad code, unknown fields and operation identity/page defects yield `INVALID_RESULT`. |
| Invalid portable value | Every rejected value kind, holes, accessors, non-enumerable/symbol keys, custom prototype, cycles, direct thenable and nested Promise is refused without executing conversion hooks. Binary never crosses. |
| Initialization success/failure | One disposable load-validation realm, then fresh evaluation per executing call from the same snapshot; no arguments/services/init export. Initialization jobs fail without being run; inert Promises do not survive realm retirement. |
| Module evaluation failure | Throw and evaluated prohibited code generation fail loading without an envelope in the validation probe. Per-call initialization faults yield `SOURCE_ERROR` without operation execution; cancellation/checked limits retain their codes and disposal rules. |
| Local imports | Exact relative `.js` lookup, dependency order, one evaluation per resolved module identity per realm, contained symlink alias identity. |
| Disallowed imports | Bare/remote/absolute/parent/extensionless specifiers, dynamic import, import.meta and attributes fail before evaluation, even in unreachable code. |
| Package containment | Symlink escape, path tricks and alias cycles rejected; valid DAG snapshotted; file changes do not alter active instance. |
| Undeclared service | Registry contains exactly declared names and no prototype fallback; absent service read is undefined, uncaught use is `SOURCE_ERROR`, provider sees no request. |
| Permission denial | Declared wrapper exists despite denied grant; attempted denied/revoked authority use through an active-scope wrapper is `PERMISSION_DENIED`, with no action or grant expansion. Valid returned denied URL is also rejected. |
| Capability lifetime | Old wrapper/context/resolver has no source path into the next realm. Test-private late native delivery sees a revoked scope and performs no action, settlement or callback; no rebinding or new-scope rejection. |
| Host failure | Test provider classifies invalid arguments, transport failure and request timeout; async rejection carries only code/message; ordinary non-2xx is not transport failure. Concrete HTTP shapes wait for its profile. |
| Cancellation | Controlled observation yields one `CANCELLED`, no new source/provider access, executing instance disposed; queued cancellation does not cancel active work. |
| Deadline | Controlled admission/queue/initialization/execution/cleanup/publication clock gates yield `TIMEOUT`; no expired success, no hard latency assertion. |
| Never-settling Promise | Pending await remains cancellable and expires without a source job; no implicit success/null. |
| Late completion/background jobs | Delivery gates close at candidate/termination processing before another job. Queued jobs and entire pending graphs are abandoned; no handler/child/species resolver/finally executes as cleanup. No old Promise or future await reaction reaches a later call; external effects are not rolled back. |
| Operation/state isolation | Calls serialize through initialization, cleanup and publication. Every call resets module/global/intrinsic state, after sync/async success, ordinary failure, invalid result or exception alike; no cache/counter persists. Copied results outlive the retired realm without aliases; logical instance remains reusable after ordinary outcomes. |
| Source-instance isolation | Same-ID instances share no module/globals/intrinsic mutations/jobs/wrappers; disposing one does not execute or revoke another's scope. Persistent namespaces are separately host-profile tested. |
| Serialization/value transfer | Dense arrays, Unicode and reserved property keys copy correctly; key insertion order fixed, no prototype pollution, repeated references copied, negative zero normalized; input page boundaries stay exact. |
| Resource exhaustion | Controlled bytes/depth/graph/jobs/admission/allocation/CPU checkpoints yield load resource diagnostics or `RESOURCE_LIMIT` as appropriate. Test documented host limits without claiming exact heap or OS scheduling bounds. |
| Ambient authority | Probe allowlist and transitive intrinsic paths; no filesystem/environment/network/browser/Node globals or service leakage; random/locale methods throw, no clock/timezone or stack exposure. |
| Dynamic compilation | Direct/indirect eval, constructor aliases, async/generator constructor chains and Reflect paths throw before running supplied text. Runtime cannot silently enable fetched code. |
| Disposal and health | No source cleanup; revocation/quiescence/realm retirement precede publication on every outcome. Execution interruption/limit disposes the logical instance and prevents waiting calls; queued/pre-execution failure preserves it. Reload is required only after logical disposal. U1 diagnostic representation is not compared. |
| Portability and precedence | Object-order-insensitive results, FIFO controlled jobs and isolated faults agree across engines. No U2 combined-fault winner, D1 universal race order, or D2 library-specific parse expectation is invented. |

The harness must also demonstrate real checkpoint paths for CPU-only source work and engine internals such as regexps, and document remaining gaps. Passing fake-provider or source-free boundary tests alone is insufficient to claim safe arbitrary JavaScript execution. Abort recovery and platform guarantees need separate evidence.

## Alternatives and implementation considerations

A single file with no imports would be smaller, but duplicates ordinary helpers without improving the host boundary; a contained acyclic graph is the chosen minimum reusable module model. Full Node/browser/npm compatibility would import authority and resolution semantics unrelated to the project. Object/default exports and factory initialization would create additional discovery/lifecycle surfaces; named functions plus synchronous evaluation suffice for the existing five operations. Async initialization would permit effects before any owning operation and require another capability lifetime; this proposal deliberately requires independent operations instead.

The [lifecycle review](../../docs/reviews/2026-09-28-javascript-lifecycle-decision.md) selects fresh realms per call (lifecycle Model B). Persistent-state Model A needs unavailable pre-effect reaction control and explicit abandoned-child semantics. Model C needs provable future quiescence, not a queue drain; Model D cannot rely on an unenforceable source promise to leave no work; Model E makes observable state survival engine-dependent. Mandatory reset loses cross-call module caches but meets independent-operation requirements and avoids engine-private Promise graph surgery. Logical instances and immutable snapshots can still be reused. No unbounded drain or opportunistic mutable-realm reuse is permitted. Allowing arbitrary thenables or serialization hooks makes boundary validation execute source code; structural copying and genuine Promises avoid that. Enabling eval cannot add a capability the declared package needs, but defeats static graph review and adds code-size/allocation paths; it is rejected. Extending ordinary JSON to binary would alter the Source API and is excluded.

The difficult feasibility points are graph preflight, exact restricted-global setup, disabling every compilation path, exception-stack suppression, non-executing value inspection, observing Promise settlement at the prescribed job boundary, retiring entire realm graphs and suppressing late native service completions without source reentry, and bounded CPU/allocation checks. An engine that cannot implement these rules is unsuitable for this proposed profile; the reference implementation must not silently weaken the contract. Existing QuickJS-NG and Boa probes expose the earlier persistent-state rule's F1 gaps; a focused fresh-realm embedding review/proof must now validate revised lifetime and initialization requirements before any production execution, with the other feasibility gaps still open. This RFC selects neither an engine nor a Rust dependency or native ABI.

## Internal review and decision

Self-review completed on 2026-09-28 against the baseline documents below. This is an internal design review, not independent implementation evidence or normative acceptance.

| Authority/evidence | Review finding and disposition |
| --- | --- |
| Charter and design principles | Five operations, portable data, smallest bounded module model; no player, adapter, SDK, binary type or platform commitment. |
| Architecture and ADRs 0001/0002 | Rust remains reference implementation only; no engine selection ADR justified by this draft. Current implementation remains static. |
| Source API | Full envelopes chosen explicitly; copied inputs and existing shapes/errors retained. Missing `playables` would invalidate a detail example, so the illustrative detail includes it. Thrown envelopes are not a second result channel. |
| Lifecycle | A proposed `init` callback would conflict with the existing no-public-init surface. Resolved by synchronous evaluation only, with no initialization export/arguments/services. The F1 revision distinguishes reusable logical instances from fresh per-call realms, captures functions per realm, preserves synchronous load validation, defines ready-call reinitialization failures, and removes portable mutable-state persistence. |
| Host API | Capability injection is explicit and invocation-scoped; signatures deliberately absent. No callable HTTP/log example invented. Required service availability is distinct from denied permission. Branded host failures preserve existing classified errors. |
| Value/authority review | Rejected stringify/coercion, accessors, proxies and binary transfer; explicit prototype/key rules prevent engine objects and prototype setters from crossing. Constructor aliases are covered by the code-generation ban. |
| Completion/resource review | An ordinary Promise `.then` observer would leave terminal ordering engine-integration dependent; the contract now fixes the source-job boundary and requires spike evidence. The revised boundary closes delivery, copies without source execution, retires the realm, then checks interruption and selects/publishes. Deadlines include initialization/cleanup; no final drain or synthetic Promise cancellation occurs. No hard preemption or heap quota is asserted. |
| F1 evidence | The [lifecycle decision](../../docs/reviews/2026-09-28-javascript-lifecycle-decision.md) compares A–E using unchanged QuickJS-NG/Boa counterexamples. Whole-graph retirement removes selective reaction revocation from the proposal; complete embedding feasibility is still unproven. No engine selected. |
| Compatibility | Existing schema recognition has no executable JS guarantee. Draft completion needs explicit later acceptance and revision reporting, not an automatic `0.2` bump. The earlier unaccepted persistence proposal is explicitly superseded; declarative behavior remains unchanged. |
| Phase 2 closure and A1 | Maintains load diagnostics versus ready-result errors, finite structural bounds and cooperative checks. JS disposal rule is scoped to mutable executing JS contexts, not a rewrite of static reuse. |
| U1/U2/P1/D1/D2 | Reviewed directly against the issue register; all remain deferred. Invalid output plus denied origin never succeeds, without assigning U2 priority. No wildcard origin expansion, universal quotas, total fault ordering or parser-library standardization. |

Review outcome: **the initial binding and subsequent F1 lifecycle decision are internally reviewed proposals; RFC acceptance and JavaScript execution support remain pending**. No blocking contradiction in the governing specification was found. The synchronous initialization restriction is a deliberate minimal choice, not an unresolved implementation option. The operation return model, export shape, completion, transfer, error mapping, injection and ambient/code-generation boundaries are decisions of this proposal.

Acceptance still needs feasibility evidence for the restrictions above and coordinated normative edits; each future service claim needs its own signatures/profiles and conformance. The [initial completed plan](../../docs/plans/completed/2026-09-28-javascript-binding-rfc.md) records the first design unit. The [lifecycle decision plan](../../docs/plans/completed/2026-09-28-javascript-lifecycle-decision.md) records revised documentation checks and reruns of the unchanged Phase 2 baseline and isolated engine experiments; those experiments are not production JavaScript execution or conformance.

The next coherent unit is a **focused fresh-realm lifecycle embedding review/proof** through the existing QuickJS-NG/rquickjs path, as scoped in the [lifecycle decision](../../docs/reviews/2026-09-28-javascript-lifecycle-decision.md#engine-implications-and-next-unit). Establish initialization, job/root retirement and late-delivery safety before production execution. Engine selection, a justified future ADR, remaining feasibility evidence and RFC acceptance stay separate; do not begin production execution or service design in this unit.
