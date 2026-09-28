# QuickJS complete value-boundary feasibility review

Date: 2026-09-28. Baseline: `2f4f31924ce10868ecc2bbd12d2e68c47822b8d3` (`test(engine): prove restricted global surface`). **Outcome B — blocker demonstrated.** Phase 3 remains active; RFC 0001 remains proposed.

The pinned safe rquickjs surface has no positive ordinary-object class query for an arbitrary candidate. This prevents the required admission gate before inspecting an unknown object's prototype, descriptors or keys. The public C query exists, but calling it requires unsafe Rust, expressly excluded from this unit. The earlier lifecycle copier used that C path; its narrower success is not a safe-Rust complete-boundary proof. Stop at this binding prerequisite rather than weakening §6 or claiming partial Outcome A.

This is a blocker for the currently pinned **safe embedding path under this task's constraint**, not evidence that QuickJS-NG's public C API cannot support §6. A reviewed safe binding over its existing class query appears capable of resolving the first blocker. Full conversion would still need its own proof.

## Authority, versions and experiment

[RFC 0001 §6](../../spec/rfcs/0001-javascript-execution-binding.md#6-value-boundary) requires a bounded structural copy, original-realm prototype identity, descriptor restrictions, native/helper-brand rejection despite prototype mutation, and no getter/proxy/coercion/serialization execution. A class-created ordinary record changed to an accepted prototype may transfer; rejecting all objects is not a conforming fallback.

- rquickjs/core/sys `0.14.0`, upstream revision `d7ef5eeae702fea24c03643064de454f1c1dd4b0`.
- QuickJS-NG `0.16.2`, upstream revision `0fdea21ff1090084e91dad812b343d92e79ba9d9`.
- Separate [experiment](../../experiments/quickjs-value-boundary-spike/README.md), default features off, `std`/`loader` on; lock resolution unchanged except root package name.
- Six behavior tests and two expected compile-fail tests; the [completed plan](../plans/completed/2026-09-28-quickjs-value-boundary-spike.md) records all commands and unchanged-suite results.

## Smallest counterexample

```js
const valid = {};
const invalid = Object.setPrototypeOf(new Map(), Object.prototype);
```

Both return `Type::Object`; the tested direct predicates agree: object true, proxy/array/function/promise/error false. Both have the original `Object.prototype` and no own enumerable properties. RFC §6 permits the first and forbids the second. A prototype-only copier admits the Map as an empty record. The executable test also repeats this with null prototypes. These are known diagnostic fixtures; their prototype/key inspection is explicitly a negative control, not the unknown-value admission path.

An additional fixture creates an ordinary class instance with private `#secret` and own data `value: 42`, then changes its prototype to original `Object.prototype`. Its visible data is just `value`. RFC permits that structural record and does not transfer its private field or class identity. A rule rejecting all class-created objects would therefore be wrong. This fixture is not copied by this experiment.

The shared predicate signature is a counterexample to those predicates plus prototype/shape admission, not a claim that every possible safe operation returns identical observations for Map and Object. For example, a captured Map method can brand-check Map. A blacklist of individual native methods is not the required positive ordinary-class gate for arbitrary engine/host exotics, and this unit does not invent such a classifier.

## Public API audit

The pinned upstream files below byte-match the installed registry sources. Reading implementation details is evidence only; no private layout or numeric class constant enters the experiment.

| Surface | Relevant behavior and implication |
| --- | --- |
| [rquickjs Value](https://github.com/DelSkayn/rquickjs/blob/d7ef5eeae702fea24c03643064de454f1c1dd4b0/core/src/value.rs) | `is_object` checks the value tag; `is_proxy`, `is_array`, `is_function`, `is_promise`, `is_error` are direct engine predicates. No safe positive ordinary-object/class-ID query is exposed. `type_of` is a coarse type classification, not ordinary-object admission. |
| [rquickjs Object](https://github.com/DelSkayn/rquickjs/blob/d7ef5eeae702fea24c03643064de454f1c1dd4b0/core/src/value/object.rs) | `get` uses `JS_GetProperty`; `props`/`own_props`/`values` fetch values through `get`. Own-key enumeration uses `JS_GetOwnPropertyNames`. `get_prototype` uses `JS_GetPrototype`. These are not safe-to-execute-on-any-candidate structural conversion primitives. |
| [Property definitions](https://github.com/DelSkayn/rquickjs/blob/d7ef5eeae702fea24c03643064de454f1c1dd4b0/core/src/value/object/property.rs) | `Object::prop` defines a property; it does not read an arbitrary candidate's own descriptor. No safe direct own-descriptor reader is exposed here. |
| [Rust class support](https://github.com/DelSkayn/rquickjs/blob/d7ef5eeae702fea24c03643064de454f1c1dd4b0/core/src/class.rs) | `Class` checks a known registered Rust class, not the ordinary engine-object class. `PropertyDescriptor` is a result supplied by a host exotic callback, not a descriptor reader for arbitrary values. Host exotics are another reason a Proxy-only exclusion is insufficient. |
| [QuickJS public header](https://github.com/quickjs-ng/quickjs/blob/0fdea21ff1090084e91dad812b343d92e79ba9d9/quickjs.h) | Declares `JS_GetClassID` and `JS_GetOwnProperty`. Both are exposed as unsafe FFI functions by rquickjs-sys; two E0133 compile-fail probes demonstrate this without calling them. |
| [QuickJS implementation](https://github.com/quickjs-ng/quickjs/blob/0fdea21ff1090084e91dad812b343d92e79ba9d9/quickjs.c) | `JS_GetClassID` reads the engine class without property dispatch. `JS_IsProxy` and `JS_IsArray` directly test engine classes (the latter does not unwrap proxies). Own-property routines can dispatch exotic behavior: class admission must precede their use. Ordinary accessor descriptor inspection returns getter/setter references rather than calling them. |

The [prior lifecycle copier](../../experiments/quickjs-lifecycle-spike/src/extraction.rs) already captures ordinary object/array class IDs with `JS_GetClassID` and reads descriptors with `JS_GetOwnProperty`, using explicit unsafe blocks. Reusing it verbatim would violate this unit's no-unsafe constraint and still would not establish the full §6 matrix.

A pristine captured `Object.getOwnPropertyDescriptor` could be considered **after** a positive non-exotic classification and with a separately reviewed descriptor-result access strategy. This review does not claim that an FFI descriptor wrapper is the only possible subsequent route. Capturing reflection alone does not close the missing first gate. Calling captured `Object.prototype.toString` as a brand detector already fails: its algorithm reads `Symbol.toStringTag` and executes a source getter, even though the function itself is pristine and native. `instanceof`/prototype tests likewise are not engine-brand proof. Proxy-only exclusion, known-helper identity registries and individual typed-array/buffer/class tests do not positively identify every ordinary record.

## Actual algorithm, trusted roots and order

There is **no working structural copier** in this unit. The test-private `record_gate` is the failed prerequisite: direct tag/class predicates reject known non-record categories; otherwise it returns `BlockedOrdinaryClassProof`. That state is an embedding limitation, not a portable failure classification. It never enumerates, obtains a prototype/descriptor, reads a property, coerces, serializes, iterates or calls a candidate. Arrays are excluded only from this record gate; RFC dense-array support is not removed or implemented.

The exact test order is:

1. Create fresh `Runtime` and full `Context`; set engine test safeguards.
2. Where needed, retain original Object prototype, native brand stringifier or test-private Proxy constructor in host handles while pristine.
3. Check the previous pristine inventory and run the unchanged dynamic-code suppression bootstrap.
4. Run the unchanged restricted-global bootstrap with its existing exact allowlists.
5. Create only test-private integer counter callbacks and pass them to fixed diagnostic fixture functions where needed.
6. Evaluate those fixtures, obtain candidates and run the direct gate.
7. Run explicitly separated negative-control operations on known fixtures and assert hook counts.
8. Drop context/runtime and all handles. No queued-job drain or package loader is introduced.

Source fixtures never run before hardening. The Proxy handle remains host-private; its constructor is not restored to the global. Mutation probes replace Object/Reflect helpers and Object.prototype.toString after candidate creation: the direct gate never calls them. A direct eval regression assertion still gets the guarded EvalError without stack. Full unchanged dynamic/global/capture/lifecycle suites supply regression evidence; this experiment is not a new operation-capture or package execution implementation.

## Executable controls and execution accounting

| Probe | Direct gate | Explicit negative control / observation |
| --- | --- | --- |
| Ordinary record vs prototype-mutated Map, original/null prototype | Both blocked for missing ordinary-class proof | Prototype-only, empty-own-shape admission would accept both; RFC outcomes must differ. |
| Class-created record with private field, prototype reset | Blocked, although record is permitted | Known fixture has only own enumerable `value: 42`; blanket class-instance rejection is not a fix. |
| Getter data candidate | Zero hooks; blocked | `Object::get` calls getter once; `props` calls it again (counter 2). |
| Source `Symbol.toStringTag` getter | Zero hooks; blocked | Captured pristine native stringifier returns `[object Object]` and increments counter once. |
| Test-private Proxy with ownKeys/get/getPrototypeOf traps | Zero traps; rejected by direct `is_proxy` | Naive own-key enumeration calls ownKeys once. |
| Poisoned reflection helpers/prototypes/globals | Gate unchanged; poisoned helper not called | Dynamic eval guard remains effective. |

No source hook executes inside the working **gate**. There is no successful conversion path about which to make a broader claim. Fixture setup and labeled negative controls intentionally execute JavaScript; these must not be presented as execution by the gate. The crate forbids unsafe code. Compile-fail examples never execute C or read an uninitialized descriptor.

## Complete §6 coverage disposition

Stop expansion on the demonstrated blocker, as requested. The following unproved requirements are retained explicitly; none is silently weakened or represented as passing conversion coverage.

| Requirement | Evidence or remaining gap |
| --- | --- |
| Null, booleans, finite binary64, ±0 normalization; reject NaN/infinities/undefined/bigint/symbol/function | No primitive converter implemented. Direct type predicates are used only for the prerequisite. |
| BMP/supplementary/empty Unicode; lone or malformed surrogates | Not tested for transfer; no normalization/coercion algorithm adopted. |
| Reserved/numeric-looking keys; scalar-order incoming insertion vs integer-index enumeration | Not tested; no key decoding, ordering or host-to-JS construction implemented. |
| Original Object/null prototype; only own enumerable string data fields; reject accessors/non-enumerables/symbols/custom prototype | Ordinary/native admission blocked first. Getter execution is demonstrated only as a naive-path negative control. Descriptor validation remains unproved. |
| Dense original-prototype arrays, length exception, holes/accessors/symbols/extras/custom shape rejection | Not implemented; no array transfer claim. |
| Promise, Error families, RegExp, Map/Set, boxed primitives, Date, typed arrays/buffers, engine/native/helpers/functions/handles despite prototype changes | Mutated Map is the decisive counterexample. Full brand matrix and test-private injection of other brands are deferred, not assumed solved by available individual predicates. |
| Class-created ordinary records with private fields and accepted prototype | Permitted fixture demonstrated; copying/private-field exclusion is not implemented. |
| Getters, source tag getter, Proxy, mutated reflection | Zero hooks in gate and executing negative controls demonstrated. |
| Setters, toJSON, valueOf/toString/Symbol.toPrimitive coercion, iterators, thenables, Promise overrides, species, prototype poisoning in actual copying | No recursive/incoming copier; full hostile-hook coverage remains open. |
| Direct/indirect/mixed cycles and independently copied repeated references | Not implemented. |
| Depth, node/value count, string/key bytes, total expanded size, checked resource failure without truncation | Not implemented. No unique-identity accounting is substituted for expanded-copy accounting. |
| Incoming validated portable values, original prototypes, own definitions despite source setters, early invalid-host-value rejection | Not implemented; no bidirectional feasibility claim. |

The only runtime safeguards here are 16 MiB memory, 256 KiB stack, and external 60-second process-group timeouts. They are experiment scaffolding, not value budgets or proof of CPU/heap/resource enforcement. No arbitrary source rewrite, unsafe implementation, private ABI, engine patch, new production globals or service API was used.

## Outcome and smallest next coherent task

**Outcome B — blocker demonstrated:** the safe pinned binding lacks a positive ordinary-object class inspection primitive needed before reflection on arbitrary candidates. The prototype-mutated Map is the minimal admission counterexample. Safe value iteration additionally cannot replace descriptor inspection because it calls getters. Existing public C class/descriptor APIs appear capable of addressing these prerequisites, but their raw Rust use requires unsafe code excluded here.

The smallest next unit is an explicitly scoped, reviewed safe binding interface for positive ordinary-object classification (and a non-executing own-descriptor path after that gate) using existing public QuickJS APIs. This could require an upstream rquickjs addition or separately authorized audited unsafe wrapper over the public C ABI. No private ABI, engine patch, RFC revision or different engine is demonstrated necessary. Do not upgrade or implement that layer as an incidental part of this blocked unit. Then resume complete §6 conversion, including all deferred rows above; the wrapper alone would not establish Outcome A.

Production runtime/dependencies, every prior experiment and RFC 0001 are unchanged. No engine/parser is selected, no ADR created, no release/tag/push performed. Resource/regexp controls, module/path containment, parser strategy, service profiles, platform/distribution proof and JavaScript conformance remain separate Phase 3 evidence.
