# Audited QuickJS public-API class bridge

Date: 2026-09-28. Baseline `d1d94be87a4c43b003a4568d8f4176034b9b3382` (`test(engine): demonstrate safe value inspection blocker`). **Outcome A — audited public-API class bridge viable.** Phase 3 remains active. This closes the positive engine-class admission prerequisite only, not RFC §6 conversion.

## Previous blocker and narrow resolution

The [previous review](2026-09-28-quickjs-value-boundary-spike.md) correctly found Outcome B under a no-unsafe-Rust constraint. Safe coarse type/prototype checks could not distinguish an ordinary record from a prototype-mutated Map; property-value iteration and native tag stringification could execute getters. Its six behavior and two compile-fail tests remain unchanged and pass. This unit explicitly permits a tiny audited public-FFI bridge; it does not invalidate that prior constrained result.

The [new experiment](../../experiments/quickjs-class-bridge-spike/README.md) captures opaque class identities from trusted native-created object/array values with the public `JS_GetClassID`. Exact identity comparison then admits only those two engine classes to later inspection. Every other object class, including Proxy and registered Rust callable objects, returns `OtherObject`. No private class constants, object layout access, engine patches or manual ABI declarations are used. The previous mutated Map now returns `OtherObject` while `{}` returns `OrdinaryObject`.

Pins are unchanged: rquickjs/core/sys **0.14.0**, revision `d7ef5eeae702fea24c03643064de454f1c1dd4b0`; QuickJS-NG **0.16.2**, revision `0fdea21ff1090084e91dad812b343d92e79ba9d9`. Defaults off, `std`/`loader` only. Separate lockfile equals the prior value-boundary lockfile after root-name replacement. No production dependency or other engine is added.

## Public API provenance and evidence level

Six retrieved pinned upstream files byte-match the local registry: QuickJS header/implementation, rquickjs Value/Object/Array implementations, and the target sys binding. The exact public declarations below are available in the supplied `aarch64-apple-darwin` binding; no custom `extern` declaration is necessary.

| API or interface | Provenance and use |
| --- | --- |
| `JSClassID`, `JS_INVALID_CLASS_ID`, `JS_GetClassID(JSValueConst)` | [Public quickjs.h](https://github.com/quickjs-ng/quickjs/blob/0fdea21ff1090084e91dad812b343d92e79ba9d9/quickjs.h), lines 153 and 706–709. Header documents object-class ID or invalid sentinel for a non-object. Sys exposes the sentinel at line 60 and query at line 764 in [target binding](https://github.com/DelSkayn/rquickjs/blob/d7ef5eeae702fea24c03643064de454f1c1dd4b0/sys/src/bindings/aarch64-apple-darwin.rs). This is the bridge's only direct C call. |
| `Value::as_raw`, `Value::ctx`, `Ctx::as_raw` | Supported rquickjs accessors: [value.rs](https://github.com/DelSkayn/rquickjs/blob/d7ef5eeae702fea24c03643064de454f1c1dd4b0/core/src/value.rs), [ctx.rs](https://github.com/DelSkayn/rquickjs/blob/d7ef5eeae702fea24c03643064de454f1c1dd4b0/core/src/context/ctx.rs). Raw value exists only inside the private query; context pointers are compared for identity, never dereferenced or exported. |
| `Object::new` → `JS_NewObject` | [object.rs](https://github.com/DelSkayn/rquickjs/blob/d7ef5eeae702fea24c03643064de454f1c1dd4b0/core/src/value/object.rs), public C declaration quickjs.h:929. Safe native allocation supplies the ordinary-class witness independently of mutable global Object. Ownership/exception handling stays inside rquickjs. |
| `Array::new` → `JS_NewArray` | [array.rs](https://github.com/DelSkayn/rquickjs/blob/d7ef5eeae702fea24c03643064de454f1c1dd4b0/core/src/value/array.rs), public C declaration quickjs.h:954. Supplies the array-class witness without invoking global Array. |
| `JS_IsProxy`, `JS_IsArray` | Public declarations quickjs.h:964 and 962; supplied sys bindings. Examined alternatives. Only safe `Value::is_proxy` is used as an independent test assertion on active/revoked proxies; neither predicate is necessary in the bridge. Header explicitly distinguishes native `JS_IsArray` from proxy-unwrapping semantics. |
| `JS_GetOwnProperty` | Public declaration quickjs.h:1011, sys-exposed; examined in the previous review. Not called or wrapped here. Descriptor inspection remains a subsequent converter proof. |

The header defines class-ID query semantics and `JSValueConst` borrowing (lines 194–206). It does **not** promise a portable numerical ID for an ordinary object, an immutable cross-version class taxonomy, or a comprehensive non-execution theorem. We do not infer those promises. The [pinned implementation](https://github.com/quickjs-ng/quickjs/blob/0fdea21ff1090084e91dad812b343d92e79ba9d9/quickjs.c) at `JS_GetClassID` (4163) shows a tag check and class read, with no allocation, callback, prototype walk or proxy dispatch. Native creation (6322, 6377, 6410) assigns the ordinary/array classes; source prototype mutation (8671 onward) changes prototype state without changing the object's class. Reading those internals supports the pinned behavioral audit; Rust does not access them. Executable matrix tests establish the needed ordinary/native partition on this build. No numerical ID is printed, embedded, assumed stable across versions, or persisted between realms.

## Safe API, algorithm and unsafe audit

[bridge.rs](../../experiments/quickjs-class-bridge-spike/src/bridge.rs) exposes only `ClassBridge::new(&Ctx)` and safe `classify(&Value) -> Result<ObjectClass, ForeignContext>`, with four results: `NonObject`, `OrdinaryObject`, `Array`, `OtherObject`. Fields and class IDs are private.

Initialization creates one fresh native object and array, obtains their IDs, and checks that both are valid and different. Setup allocation failures return the binding error; broken witness assumptions panic before admitting any candidate (experiment setup failure, not a portable source error). Witness handles are dropped; metadata remains in the bridge with a scoped Ctx handle. Classification first rejects a different associated context, then reads the borrowed candidate's class once. Sentinel means non-object; equality to either witness means that engine class; everything else is other-object. It does not retain or clone the candidate and returns only Rust scalars.

**Exactly one unsafe block, zero unsafe functions, zero unsafe impls**, all in the private `class_id` function at `src/bridge.rs:37`. The crate has `deny(unsafe_code)` with a function-local exception only there; `tests.rs` forbids unsafe code. Every unsafe operation is that single expression `qjs::JS_GetClassID(value.as_raw())`.

Audited invariants:

- The borrowed `Value` is a live, rooted supported rquickjs value. Its lifetime and Ctx protect the underlying allocation during the synchronous query. No raw value is fabricated or ownership converted.
- rquickjs's context access discipline is retained; the `parallel` feature is disabled. The bridge retains Ctx, cannot escape its scope, and exposes neither raw value nor context/runtime pointer.
- The query takes no context argument, so no C context/value pair can be mismatched. The safe wrapper additionally compares supported Ctx pointer identities before querying candidates. Same-runtime foreign contexts via `Persistent::restore`, and values from a distinct runtime, return `ForeignContext` without reflection.
- Candidate and bridge lifetimes need not be identical: both remain borrowed/live for the call, and the context identity check occurs first. This avoids unsafe lifetime coercions.
- Public `JSValueConst` is borrowed. The call neither consumes nor duplicates it, retains references, allocates, changes engine state, nor invokes JS in the audited pinned implementation. No manual free or reference-count adjustment is required.
- The only class constant used is public `JS_INVALID_CLASS_ID`, whose meaning is explicitly in the header. Ordinary/array classes come from public creation plus query, not private IDs.

The associated Ctx check is not a universal origin/provenance proof if trusted host code deliberately imports an object through another realm and obtains a new wrapper for it. Original-prototype checks, host-handle policies and Model B isolation still belong to the full converter/embedding. Classification also does not prove a record's properties or shape valid.

Miri was **not run**: ordinary Miri execution cannot validate the central native QuickJS C call. This is source/API/lifetime audit plus native executable evidence, not a Miri memory-safety proof or platform ABI certification.

## Executable classification matrix

All ten tests pass. `OrdinaryObject` and `Array` mean eligible engine class for later checking, not transfer acceptance. All native rejection below uses the same captured-ID positive admission, conservatively returning `OtherObject`, without a list of native class IDs.

| Candidate | Result and qualification |
| --- | --- |
| `{}`, null-prototype record | `OrdinaryObject`. |
| Ordinary class instance, including private field, before and after prototype reset | `OrdinaryObject` both times; before reset its custom prototype still disqualifies it under §6. Private fields are not inspected/transferred. |
| Accessor-bearing record, custom-prototype record | `OrdinaryObject`; future descriptor/prototype checks must reject. Classification executes no getter. |
| Dense array, sparse array, Array subclass, array with null prototype | `Array`; density/original prototype/extra properties remain separate validation. |
| Object with Array.prototype; numeric-key/length object | `OrdinaryObject`, never `Array`. Map changed to Array.prototype remains `OtherObject`. |
| Map, Set, RegExp, intrinsic Promise | `OtherObject`, including replacement with Object.prototype, null or Array.prototype and forged constructor/tag/data fields. |
| Error, TypeError, EvalError, RangeError, ReferenceError, SyntaxError, URIError, AggregateError, Error subclass | `OtherObject` through the same three prototype mutations. |
| Boxed String, Number, Boolean, BigInt, Symbol | `OtherObject` through the same mutations. |
| WeakMap/WeakSet, ordinary/arrow functions, generator object, array/Map/string iterator | `OtherObject` through the same mutations. These fixtures exercise additional native classes not enumerated in the bridge. |
| Test-private Date, ArrayBuffer, SharedArrayBuffer, DataView, all 12 typed-array constructors present in the pin | `OtherObject` originally, with Object.prototype and with null prototype, after forged constructor/data fields. Date uses explicit epoch 0. |
| Test-private WeakRef, FinalizationRegistry, DisposableStack, AsyncDisposableStack | `OtherObject` before/after the same private-fixture mutations; additional unsupported classes fail closed without code changes. No finalization or disposal is invoked. |
| Proxy around object, array, Map, callable/constructible function; revoked versions of all four | `OtherObject`; no trap fires. Independent `is_proxy` remains true after revocation. |
| `Function::new` host callback holding opaque `Rc<Cell<i32>>` payload | `OtherObject` before/after null prototype and record property; payload unchanged, callback never called. This safely exercises rquickjs's registered callable Rust class without an unsafe custom-class fixture. |
| Plain host-created null-prototype wrapper | `OrdinaryObject`: it has no engine-native brand. A future converter must track protected helper identities or use a branded representation. This control prevents conflating host provenance with engine class. |
| Null, undefined, boolean, number, string, bigint, symbol | `NonObject`; no primitive conversion/domain acceptance is implied. |

The standard native matrix has 26 fixtures × three mutated prototypes. Twenty excluded/native fixtures are created privately in the pristine realm, retained only for tests, then checked before and after two prototype mutations. No removed constructor is put back on the source global. Date, ArrayBuffer and Proxy globals remain absent after hardening. The host-native callback uses the existing safe `Function::new` implementation; its [registered RustFunction class](https://github.com/DelSkayn/rquickjs/blob/d7ef5eeae702fea24c03643064de454f1c1dd4b0/core/src/value/function/ffi.rs) has a non-ordinary callable class identity. We do not claim every native application helper is automatically branded.

## Non-execution, negative controls and hardening preservation

Hostile record properties include getters/setters for constructor, __proto__, toString, valueOf, toJSON, then, Symbol.toPrimitive, Symbol.toStringTag, Symbol.iterator and Symbol.species; the inherited prototype also has a throwing getter. Array elements and Promise then/constructor properties have throwing getters. Classification touches none; counters remain zero. Proxy handlers install loudly throwing get/getOwnPropertyDescriptor/ownKeys/getPrototypeOf/has/apply/construct traps; all active/revoked classifications leave the shared counter zero. The classifier never asks for Proxy targets or handlers.

Separate negative controls establish observability: constructor-name access, native Object.prototype.toString, Reflect.get, String coercion and spread each trigger one source hook (five total). Proxy prototype traversal triggers one trap. A source Symbol.hasInstance implementation runs during instanceof; a prototype-mutated Map passes instanceof Object while class inspection still returns OtherObject. These controls are separate from classification and are not part of its implementation. The prior getter/property-iterator counterexamples remain intact.

Object/Array constructor links, Object.prototype tag, reflection helpers and global Object/Array/Map/Set/Reflect are poisoned after fixture creation. Captured class metadata continues to classify correctly. Even a second native witness capture avoids mutated global constructors. The dynamic eval guard still throws EvalError. The prior complete dynamic/global matrices, module capture, static analysis/import attributes and lifecycle suites all pass unchanged.

Actual order: fresh Runtime/Context with native intrinsics → capture class witnesses and optional private diagnostic handles → pristine inventory check → unchanged dynamic hardening → unchanged restricted-global hardening → install integer-only diagnostic callbacks → evaluate fixed source fixtures → classify → run labeled negative controls → release all handles/context/runtime. No package fixture runs before hardening, no source text is rewritten, no jobs are pumped, and no new loader/capture strategy is introduced. Fresh independent runtime/context pairs recapture their own witnesses. Test settings are 16 MiB engine memory, 256 KiB stack and external 60-second timeouts, not resource-policy proof.

## Upgrade behavior, limits and next unit

The classifier has two positive identities, not a closed native blacklist. Any new distinct native class automatically maps to OtherObject; the tests demonstrate this using additional iterable/disposable/GC and registered Rust callable classes absent from the algorithm. Unknown does not default to ordinary. Setup re-captures witnesses rather than relying on IDs persisting across runtimes or releases.

This is fail-closed for **new distinct classes**, not a guarantee against arbitrary future engine semantic changes. If an engine upgrade starts representing an exotic/native payload using the ordinary object class, a pinned source/matrix re-audit must catch and resolve that change. The header does not promise such future designs impossible. Version pins, prior pristine-surface drift checks, native mutation matrix and source audit must be retained on upgrade; do not regenerate expectations silently. No private constants are required on the tested pin.

**Outcome A — audited public-API class bridge viable.** Positive ordinary-object/array recognition is demonstrated, including source construction-history preservation and rejection of all tested branded/proxy mutations without JS execution. The allowed unsafe scope fits one public-query block; no private ABI/layout/API is needed.

Next: resume the complete RFC §6 value-boundary proof using this safe gate. It must separately establish non-executing descriptor inspection after admission, original prototypes, host helper identity exclusion, complete Unicode/key rules, dense shape validation, cycles/repeated references, expanded budgets and incoming construction. None is implemented here. No engine/parser selected, no ADR, no RFC modification/acceptance, no production runtime/dependency change, no push/tag/release. Full validation and delivery are recorded in the [completed plan](../plans/completed/2026-09-28-quickjs-class-bridge-spike.md).
