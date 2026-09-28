# QuickJS restricted globals and ambient authority

## Conclusion and scope

**Outcome A — restricted-global surface viable.** Pinned public rquickjs embedding APIs and standard JavaScript operations establish the tested RFC §9 surface before package execution. Nineteen tests cover the exact global allowlist, removed intrinsic extensions, clocks/randomness/locale/stacks, forbidden-object reachability, host-boundary visibility, mutations and fresh realms. All 80 previously demonstrated dynamic-compilation routes remain blocked, including during hostile initialization. No demonstrated bypass remains within this scope.

Baseline: `6476e337bdf4677324b5a2f065a9f2ddd936a737`, `test(engine): prove dynamic compilation suppression`; clean tree, no active plans. Phase 3 remains active. [RFC 0001](../../spec/rfcs/0001-javascript-execution-binding.md#9-globals-and-dynamic-code) is authoritative and unchanged, still proposed. This is an [isolated experiment](../../experiments/quickjs-global-surface-spike/README.md), not a complete sandbox, engine/parser selection, service profile, production execution or conformance implementation.

## Pins, public APIs and source evidence

| Component | Evaluated pin |
| --- | --- |
| rquickjs/core/sys | `0.14.0`, revision `d7ef5eeae702fea24c03643064de454f1c1dd4b0` |
| QuickJS-NG | `0.16.2`, revision `0fdea21ff1090084e91dad812b343d92e79ba9d9` |
| Host | macOS 27.0 / arm64, Rust/Cargo 1.96.0, `aarch64-apple-darwin`; no other-platform evidence |
| Dependencies | Separate crate, rquickjs `=0.14.0`, defaults off, `std`/`loader` on. Lockfile equals prior dynamic-code lock after root-name replacement. |

Five pinned upstream files were downloaded and byte-compared to the registry/vendor source; all matched:

| Source | Evidence used |
| --- | --- |
| [Context base](https://github.com/DelSkayn/rquickjs/blob/d7ef5eeae702fea24c03643064de454f1c1dd4b0/core/src/context/base.rs), [intrinsic builder](https://github.com/DelSkayn/rquickjs/blob/d7ef5eeae702fea24c03643064de454f1c1dd4b0/core/src/context/builder.rs) | `Context::full` and custom intrinsic installation are public. The experiment measures the actual full context, then hardens it; it assumes neither the command-line shell nor an unconfigured minimal realm. |
| [Function API](https://github.com/DelSkayn/rquickjs/blob/d7ef5eeae702fea24c03643064de454f1c1dd4b0/core/src/value/function.rs) | `Function::new` exposes a native callable without exposing the Rust closure, Ctx/Runtime or callback storage as JS properties. |
| [Public QuickJS header](https://github.com/quickjs-ng/quickjs/blob/0fdea21ff1090084e91dad812b343d92e79ba9d9/quickjs.h), [implementation](https://github.com/quickjs-ng/quickjs/blob/0fdea21ff1090084e91dad812b343d92e79ba9d9/quickjs.c) | Global/intrinsic installation, Error stack storage/accessors, function location getters, Symbol constants, iterator prototypes, Date, performance and random implementations. No private operation is called. |

The [ES2023 edition](https://262.ecma-international.org/14.0/) was retrieved directly (the browser rejected its size) to review the retained intrinsic property table. Standard Annex B legacy Object/String/RegExp properties already present in QuickJS remain; they are standard language properties, not additional global names or host authority. This is a tested property surface, not a new normative intrinsic catalog or complete language-conformance claim. Later-standard/proposal additions are removed even when deterministic. No parser syntax policy is changed.

The implementation uses safe public Rust APIs (`Ctx::eval`, Function, Module, Runtime loader/userdata operations) and standard object/descriptor/Proxy/bind operations. No unsafe Rust, FFI call, engine patch, private struct, memory surgery, bytecode rewriting or package-source rewriting was added. The existing private-native-module capture strategy is reused in a small fixture.

## Pristine inventory versus RFC and hardened surface

The committed [inventory](../../experiments/quickjs-global-surface-spike/src/pristine-inventory.json) records **70 own string names plus one symbol** on the pristine global and descriptors for 128 further objects. Broad value kinds and writable/enumerable/configurable flags are recorded for every key. Global properties are configurable, writable, non-enumerable data properties except `undefined`, `NaN`, `Infinity` (non-writable/non-configurable); `performance` is enumerable; global `Symbol.toStringTag` is non-writable but configurable. These observations are not normative API choices.

The [expected allowlist](../../experiments/quickjs-global-surface-spike/src/allowed-globals.json) contains exactly these **38 RFC names**, independently checked against the RFC text:

```text
globalThis undefined NaN Infinity
Object Function Boolean Symbol Number BigInt String RegExp Array
Error AggregateError EvalError RangeError ReferenceError SyntaxError TypeError URIError
Math JSON Reflect Map Set WeakMap WeakSet Promise
eval isFinite isNaN parseFloat parseInt decodeURI decodeURIComponent encodeURI encodeURIComponent
```

After hardening, `Object.getOwnPropertyNames`, `Reflect.ownKeys` and keys of `Object.getOwnPropertyDescriptors` agree exactly with that set. There are no global own symbols, tombstones or source-visible bookkeeping exceptions. The pristine descriptor gate runs before any hardening/package code; injected unknown global and Math properties fail it. The hardener separately checks exact retained intrinsic and final global keys. Upgrades require reviewing expectations, not silently refreshing them.

| Pristine category / observed names | Hardened result |
| --- | --- |
| Browser: window, document, navigator, location, fetch, XMLHttpRequest, WebSocket, localStorage, sessionStorage, crypto | Absent before and after, observed with membership and own-key checks. |
| Node: process, require, module, exports, Buffer, __dirname, __filename, global; also console, setImmediate | Absent before and after. |
| Other bridges: std, os, scriptArgs, print, load, gc, Deno, Bun, Worker, URL, setTimeout, setInterval | Absent before and after; no shell std/os module loader was installed. |
| Clock/queue: Date, performance, queueMicrotask | Present initially; deleted, with no reachable saved native alias. |
| GC/concurrency/binary: WeakRef, FinalizationRegistry, SharedArrayBuffer, Atomics, ArrayBuffer, DataView and all 12 typed-array constructors, including Float16Array | Present initially; deleted. These are explicitly excluded by §9, independent of result-transfer restrictions. No binary transfer research occurred. |
| Engine/nonprofile additions: InternalError, SuppressedError, Iterator, DisposableStack, AsyncDisposableStack, DOMException, btoa, atob, escape, unescape | Present initially; deleted. InternalError/SuppressedError prototype name/message/constructor properties also removed to prevent engine-created errors recovering extension constructors. |
| Proxy | Present initially; removed from source globals after hidden guards are created. Proxy has no ordinary instance prototype that would recover its constructor; no standard object reflection unwraps targets/handlers. |
| Intl, WebAssembly | Absent initially and afterward. No WASM execution path or research added. |
| Global symbol tag | Present initially; removed to match exact own-key contract. |

## Intrinsic hardening and exact order

The [intrinsic policy](../../experiments/quickjs-global-surface-spike/src/allowed-intrinsics.json) enumerates retained keys for 72 objects, including derived function-family prototypes, array unscopables and iterator prototypes. The trusted [bootstrap](../../experiments/quickjs-global-surface-spike/src/harden.js) removes:

- Function.prototype `fileName`, `lineNumber`, `columnNumber`; Error.prototype `stack`; Error `captureStackTrace`, `stackTraceLimit`, `prepareStackTrace`, `isError`.
- Object/Map `groupBy`; Array `fromAsync`; String `isWellFormed`/`toWellFormed`; Math `f16round`/`sumPrecise`; RegExp `escape`/prototype `unicodeSets`; JSON `rawJSON`/`isRawJSON`; Promise `try`/`withResolvers`; Map/WeakMap insertion helpers; Set composition helpers.
- New Iterator helpers and its constructor/disposal/tag links, retaining the ES2023 self-iterator method. AsyncIterator disposal is removed. Removing only global Iterator would leave helpers and its constructor reachable through array/string/map/set/regexp/generator iteration.

Native Symbol's `dispose` and `asyncDispose` properties are **nonconfigurable**: a negative control proves direct deletion fails. The working public-object-model solution binds native Symbol without arguments, copies only its standard static descriptors/prototype, then replaces both global Symbol and Symbol.prototype.constructor. Native Symbol stays only inside a bound function's inaccessible target slot. Tested creation, boxing, constructor identity, registry lookup and rejection of `new Symbol()` retain ordinary behavior. A Proxy directly around native Symbol could not legally hide nonconfigurable own keys; no such invalid approach is used.

Actual order:

1. Create a dedicated fresh Runtime/Context, install all intended intrinsics, verify the pristine descriptor inventory. The fixed fixture resolver/loader is host state, not a global.
2. Run the **unchanged prior dynamic-code hardener** by host Ctx::eval, locking eval/Function and constructor links and redirecting specialized constructor parents.
3. Build the Symbol facade, remove hidden error-family constructor edges, collect relevant intrinsic roots, install non-writable/nonconfigurable throwing random/locale guards, prune intrinsic extensions (including stack/location hooks), delete disallowed globals and verify exact keys.
4. Create the test-private native binding and frozen null-prototype wrapper/registry/context in host-held handles. No global service or temporary bootstrap binding is added.
5. Compile immutable entry/helper bytes. Register the export-free private capture module and compile the host root importing capture before entry. At the native hook, capture the real linked operation before any package body (the package marker is still absent).
6. Evaluate package bodies, check the saved export identity and absence of initialization jobs, then invoke with the test context and pump bounded jobs only for the allowed async operation.
7. Release scoped values/context/runtime. The next invocation repeats fresh construction and hardening.

No package executes during steps 1–5. Diagnostic modules/functions in other probes likewise compile only after hardening. Negative controls retain a native RNG and stack getter in **host-only test handles** before hardening and demonstrate that these still work afterward: deleting properties cannot revoke already leaked aliases. They are never passed to a package in the working path. No late intrinsic installation or host method may reintroduce removed originals.

## Clock, randomness and locale evidence

Pristine Date.now is checked against host timestamps. Date(), epoch string/locale formatting, timezone offset and performance numeric APIs are observed. Under child `TZ=UTC`, epoch formatting shows midnight/GMT+0000 and offset 0; under `TZ=Asia/Shanghai`, it shows 08:00/GMT+0800 and offset -480. These are labeled environment-dependent negative controls, not normative snapshots.

The RFC unambiguously requires **absent Date**, not deterministic Date or disabled no-argument construction. After hardening Date.now(), new Date(), Date(), epoch offset/string/locale calls and performance access fail at the absent binding. Date/performance/timer keys are absent under alternate enumeration. No date object is injected. Both timezone runs pass all hardened checks.

Math.random initially produces differing samples. Its replacement is a hidden Proxy over the native method with a frozen null-prototype apply handler that throws the captured native TypeError; its target is never returned. All eight random/locale properties are locked, with call/apply/bind aliases tested. Crypto/randomBytes/getRandomValues/randomUUID and std/os helpers are observed absent; full globals plus forbidden-identity traversal check other installed roots. Internal engine randomness such as hash seeds is not a callable RNG authority claim and is not investigated as side-channel research.

Intl is absent. Pristine observations include `"i".toLocaleUpperCase()` → `"I"`, `(1234.5).toLocaleString()` → `"1234.5"`, and Date locale output differing by timezone. This does not certify unrestricted locale behavior deterministic. After hardening, String localeCompare/toLocaleLowerCase/toLocaleUpperCase and Object/Array/Number/BigInt toLocaleString throw exact native TypeError without reading environment configuration. Pristine BigInt inherits Object's method; the bootstrap adds its required **own locked throwing method**, avoiding an unprotected shadowable prototype slot. Ordinary comparison/case conversion remain usable.

## Error stacks and function reflection

Pinned QuickJS stores ordinary Error stacks internally and exposes them through a configurable Error.prototype accessor. `captureStackTrace` separately can install an own stack property, and prepareStackTrace/stackTraceLimit setters control internal state. Removing **all four hooks** before source evaluation closes those paths; stripping or sanitizing a single error instance would not.

Negative controls expose the synthetic absolute module path `/fixture/private/package/source.js`, line/column information and nested function names, while bootstrap-created errors mention `eval_script`. The native-call conversion control exposes package frames, rather than an assumed Rust path. Hardened probes inspect new Error, module-initialization errors, nested calls, thrown/caught errors, eight standard Error families and subclasses, engine TypeError/ReferenceError/RangeError/SyntaxError/URIError, dynamic-guard EvalError and stack-overflow RangeError. `stack` is absent by read, `in`, own keys and prototype-chain descriptor inspection. Test-private native InternalError/SuppressedError instances also cannot recover their deleted constructors. Error class/native code is not replaced with an unrelated source error object.

Recreating Error.prepareStackTrace/stackTraceLimit creates ordinary source properties; new errors do not call the replacement callback or expose stacks. The engine's original setter/getter functions are absent from the reachable descriptor graph. Internal stack data may still be allocated; the claim is source inaccessibility, not zero allocation or eliminated resource cost. No source-visible DOMException constructor remains to obtain its separate own-stack mechanism.

Function.prototype.toString still reflects a package function's own body. Native builtins, native host callback and Proxy compiler/random guards show native-code formatting; no handler closure, package/host path or private module name is revealed through that route. Source locations from the separate nonstandard Function prototype getters are removed. The RFC explicitly treats diagnostic and function-source formatting as nonportable; it does not require hiding source-authored function text or all engine fingerprints. This is not anti-fingerprinting research.

## Reachability, host boundary and mutation evidence

The trusted [traversal witness](../../experiments/quickjs-global-surface-spike/src/traverse.js) captures forbidden object identities before hardening **only for test comparison**. It records disallowed global objects, original eval/four compiler constructors/Symbol, native RNG/locale functions and stack/location getters/setters. Starting at globals plus generated ordinary/async/generator functions, iterator instances, caught/new errors and an optional host context, it breadth-first follows prototypes and own descriptor values/getters/setters, including symbol keys, without invoking getters. Depth limit **16**, unique-object limit **4096**; exceeding either fails, never reports truncated success.

The pristine graph triggers the forbidden-identity control. The hardened graph completes at **368 nodes, 1482 edges, depth 4**; with the representative host context, **372 nodes, 1490 edges, depth 4**. Neither finds a tested forbidden identity. The independent intrinsic-key policy checks removed extension names too. This bounded graph plus pinned object-model reasoning is not a proof against engine bugs, future host injections or every possible engine allocation. Proxy/bound-function hidden slots intentionally retain targets; public JavaScript reflection cannot reveal them and their invocation paths are guarded or, for Symbol, safely forward only symbol creation.

The native integer callback closes over no Rust object, runtime, resolver or capability registry. It receives only an integer and returns an integer; its own keys are name/length, constructor reaches the guarded Function, receiver changes do not acquire authority, and bad-argument native errors have no stack. It is nested in frozen null-prototype diagnostic wrappers passed as an argument, not exposed globally. This tests authority leakage only and proposes no service name/signature. The capture closure lives in runtime userdata and is removed before callback invocation; it never enters this wrapper.

The fixture rejects package imports of `@host/capture`, `@host/root`, std, os and a filesystem path, even with the native capture record registered. Source has no resolver, loader, snapshot, context/runtime object, global services or capture global. Ordinary static helper import works. Prior unchanged parser/capture tests remain the evidence for preflight and private-name routing; complete filesystem containment and arbitrary module preflight are not implemented here.

Mutation probes fail to assign/redefine/delete protected random/locale/compiler properties or replace them with accessors. Nulling guard prototypes, replacing TypeError's global, poisoning Object.prototype trap names, and replaying saved descriptors do not restore original authority. Prior compiler probes also poison Reflect helpers and constructor/prototype links. Source-local global/prototype memory remains mutable: adding/deleting ordinary properties and defining harmless same-named Date/Intl/Proxy/WebAssembly/performance/crypto or a replacement Math object is permitted. Such source-defined code does not recover original capabilities. The entire realm is not frozen.

A hostile invocation creates local globals/prototype properties and replacements; a new dedicated Runtime/Context starts with the pristine snapshot and independently hardened allowlist/methods, without the old markers. Model B is unchanged.

## Required functionality, tests and remaining gap

The integrated module test preserves objects, arrays, Map/Set/WeakMap/WeakSet, Symbol/BigInt, JSON, strings/numbers, classes/ordinary functions, RegExp, promises, async/await and static imports. It captures an async home before initialization, passes an argument/context, and obtains 42 after bounded job pumping. The reusable dynamic corpus reports all **80** native-EvalError denials before/after hostile mutation and during module initialization; supplied text never executes. No prior test or bootstrap was weakened or duplicated into a divergent compiler guard.

All **19 tests** pass, with no ignored tests, alongside the unchanged baseline and all six prior Phase 3 experiment suites. Exact commands, results and changed files are in the [completed plan](../plans/completed/2026-09-28-quickjs-global-surface-spike.md). The two timezone variants pass. Documentation/link/schema/whitespace checks and touched-crate fmt/strict Clippy pass.

No unresolved ambiguity in §9's tested global/ambient rules was found: Date, Proxy, WeakRef/finalization, shared memory and typed-array/buffer globals are explicitly excluded; random and locale methods explicitly throw; stack is explicitly absent. This does not settle unrelated engine conformance or optional language-feature portability. Post-ES2023 syntax rejection still belongs to static preflight, not property pruning.

**Smallest next unit: complete non-executing value-conversion feasibility against RFC §6**, extending the earlier limited copier with complete string/surrogate/key rules, native/helper-brand rejection and bounded expanded-value accounting under hostile mutation, without source hooks. Resource/CPU/memory/regexp control, production module/path containment, parser strategy, service profiles, platform/build/distribution and JavaScript conformance remain separate gaps. The experiment's 32 MiB/512 KiB test limits and external timeout are safeguards, not resource-policy evidence. No engine selected, no ADR, no RFC acceptance, no production runtime/dependency change, no push/tag/release.
