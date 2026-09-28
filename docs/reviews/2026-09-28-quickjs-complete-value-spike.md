# QuickJS complete RFC §6 value-boundary feasibility

**Outcome A — complete RFC §6 value boundary feasible.** The demonstrated public-API path copies the tested complete portable domain in both directions, with class-gated descriptors, host-held identity exclusions, strict Unicode, dense arrays, cycles/repeated references, expanded budgets and incoming own-property construction. No source getter/hook/trap executes during working conversion. No remaining blocker was demonstrated within this unit.

Baseline: `fb7ce8e336d9910a327568ea9921876c595d4948` (`test(engine): prove audited public API class bridge`), initially clean with no active plan. Phase 3 stays active. RFC 0001 remains proposed; no engine/parser selection, ADR, production JavaScript or service support follows. The [experiment](../../experiments/quickjs-complete-value-spike/README.md) and [completed plan](../plans/completed/2026-09-28-quickjs-complete-value-spike.md) contain reproduction and full validation.

## Pins and prior evidence

- rquickjs/core/sys `0.14.0`, revision `d7ef5eeae702fea24c03643064de454f1c1dd4b0`.
- QuickJS-NG `0.16.2`, revision `0fdea21ff1090084e91dad812b343d92e79ba9d9`.
- Default features disabled; `std`/`loader` enabled. The only new local dependency is the unchanged class-bridge experiment. External locked versions/checksums are unchanged.

The [original no-unsafe blocker](2026-09-28-quickjs-value-boundary-spike.md) remains valid under its constraint and its tests remain untouched. The [audited class bridge](2026-09-28-quickjs-class-bridge-spike.md) closes the missing positive ordinary-class query. This experiment consumes that crate rather than duplicating its FFI. Native Map/Set prototype mutations continue to fail; descriptor safety and host identity policy are now demonstrated after the gate.

## Trust setup and actual execution order

1. Create a fresh runtime/context and pristine trusted intrinsics.
2. Capture the existing class gate, original Object/Array prototypes, and intrinsic identity roots using fixed host-only setup code. No package runs. The root traversal follows descriptors/prototypes without calling getters, with iterator/specialized-function witnesses for otherwise hidden shared prototypes. It is bounded to depth 16 and 4,096 identities.
3. Prepare test-private removed-native witnesses where needed; retain them host-side.
4. Verify the unchanged pristine inventory and execute the existing dynamic-code suppression and restricted-global hardening scripts, directly included from their original files.
5. Install/register test-private helpers before exposing them. Every ordinary helper, including nested helpers, is registered by identity; native functions already fail the class gate. Removed Date/Proxy/buffer constructors are never restored to source globals.
6. Construct controlled candidate values with diagnostic source, then execute outgoing conversion; for incoming tests prevalidate the host tree, construct values, and optionally inspect/round-trip the result. No source calls occur inside either converter.
7. Drop held values and retire the fresh realm without pumping jobs. No source or converter state passes to another invocation.

This unit does not rebuild the package loader/operation-capture harness. Its established private capture ordering is preserved and its unchanged suite is rerun. Pristine host setup uses JavaScript, but is not source-visible conversion code and is never run on a source result. All candidate conversion uses native metadata and Rust operations.

## Public APIs and provenance

All C functions below are declared in the pinned [QuickJS-NG public header](https://github.com/quickjs-ng/quickjs/blob/0fdea21ff1090084e91dad812b343d92e79ba9d9/quickjs.h) and already exposed by `rquickjs-sys`. No manual extern declaration, private struct, private class constant, patch or memory-layout inference is used. Nine upstream files byte-match local registry files: QuickJS header/implementation, rquickjs Value/Object/Property/Atom/String/Array and the aarch64 Apple sys bindings.

| Public API / header line | Use and non-execution boundary |
| --- | --- |
| `JS_GetClassID`, 709 | Unchanged bridge captures classes from native ordinary witnesses and compares candidates. Unknown distinct classes reject before reflection. No prototype or source tag is read. |
| `JS_GetPrototype`, 994 | Safe `Object::get_prototype` only after ordinary/array admission and intrinsic/helper rejection. It does not inspect the prototype object's properties. |
| `JS_GetOwnPropertyNames`, 1008 | Safe `own_keys::<Atom>` with string and symbol flags, including non-enumerables. Native ordinary/array enumeration has no source callback; Proxy enumeration would and is excluded first. Iterator storage is released by rquickjs. |
| `JS_AtomToString`, 622 | Safe `Atom::to_value`; actual result tag is checked before UTF-16 extraction. Symbols reject. No key coercion or C-string truncation. |
| `JS_NewAtomLen`, 612; `JS_FreeAtom`, 617 | Explicit-length validated UTF-8 descriptor keys, including NUL. Atom creation is native, not property access. The owned temporary atom is always freed. |
| `JS_GetOwnProperty`, 1011; `JS_FreeValue`, 863 | Descriptor extraction only from admitted objects. Accessor functions are returned as owned values and released without invocation. Data values transfer once to rooted rquickjs `Value`. Public flags identify data/accessor and enumerable/configurable state. |
| `JS_ToCStringLenUTF16`, 913; `JS_FreeCStringUTF16`, 922 | Only actual primitive strings reach this otherwise coercive API. The public contract provides native-endian UTF-16, including possible unpaired surrogates. Strict Rust decoding rejects those. RAII pairs every successful buffer acquisition with release. |
| `JS_NewObjectProto`, 928; `JS_NewArray`, 954; `JS_SetPrototype`, 993 | Safe native incoming constructors and explicit original prototypes on fresh targets; no source-global constructors or inherited setters. |
| `JS_NewStringLen`, 889; `JS_DefineProperty`, 1046 | Safe incoming string creation and own writable/enumerable/configurable data definitions. Rust strings have valid Unicode; keys are length-aware. No source assignment protocol. |

The detailed non-execution and ownership argument additionally audits the pinned [public implementation source](https://github.com/quickjs-ng/quickjs/blob/0fdea21ff1090084e91dad812b343d92e79ba9d9/quickjs.c), not just header signatures. The descriptor implementation duplicates data/getter/setter values instead of calling accessors. Its fast-array branch reads a verified present own slot; absent slots never fall through to a prototype read in this path. Exotic callbacks are precisely why the class gate must precede reflection. Native lazy intrinsic properties are captured during pristine traversal; no unregistered host-native lazy properties are installed on transferable records. Source JavaScript cannot install engine auto-init metadata. A future embedding that installs such native machinery must exclude its identities or re-audit it.

The UTF-16 function internally has a coercive fallback, so checking the real primitive tag before calling it is essential. `Atom::to_string` is deliberately avoided: the pinned binding uses a NUL-terminated C-string path and unchecked UTF-8 assumptions. This experiment uses `Atom::to_value` followed by actual tag inspection and strict UTF-16 decoding for all keys. No malformed string enters unchecked Rust UTF-8 conversion in the working path.

## Outgoing algorithm and matrix

Each visit checks context, depth, node and expanded budget. Primitives use actual tags and direct values, never coercion. Objects first pass the unchanged class gate; then host-held intrinsic/helper identity and active-path cycle checks; then trusted prototype identity. Only after those checks are own keys and descriptors inspected. Record keys/values and array values recurse through exactly the same path. A result is returned only after all visits succeed.

| Category | Executable result |
| --- | --- |
| Null, booleans, finite binary64 | Exact copy, both signs of zero normalize to positive zero. Maximum finite, minimum subnormal, safe-integer edge and fractional values tested. |
| Undefined, NaN, both infinities, bigint, symbols, ordinary/async functions | Reject without coercion, omission or null substitution. Nested invalid values also reject the whole result. |
| Unicode values and keys | Preserve ASCII, BMP, supplementary scalars, mixed text, empty text, whitespace, decomposed text and embedded NUL. Six malformed surrogate sequences reject in both values and keys; rope-string rejection also tested. No normalization/repair. |
| Reserved/numeric-looking keys | `__proto__`, `prototype`, `constructor`, `toString`, `valueOf`, `hasOwnProperty`, numeric-looking/empty/supplementary/NUL keys remain data. |
| Records | Original Object prototype or null accepted; own enumerable string-keyed data only. Frozen data records accepted. Accessor, non-enumerable, symbol or custom-prototype records reject. |
| User class-created records | Initially reject custom prototype. Explicit mutation to original Object prototype or null permits own data copy; private fields/methods/class identity do not transfer. |
| Arrays | Actual array class and original Array prototype, dense canonical indices, data elements and built-in non-enumerable/non-configurable length. Frozen arrays work. Holes/deletion/accessors/extras/symbols/custom prototypes/subclass prototypes and Array-prototype impostors reject. |
| Map, Set, WeakMap/WeakSet, RegExp, Promise, Error families, boxed primitives, iterators/generators | Reject after prototype, constructor and tag mutations. 22 native fixtures each tested with original Object prototype and null. |
| Test-private Date, buffers, DataView, all 12 typed-array constructors | 16 witnesses reject both before and after source prototype mutation. No binary domain is introduced. |
| Native Rust callable/opaque closure | Reject by engine class without calling its closure. The unchanged class-bridge suite additionally covers its prototype mutations. |
| Active/nested/revoked proxies | Four target families (object, array, Map, function), three proxy forms each: all 12 reject before reflection, zero traps. |
| Ordinary host context/registry/wrapper/nested helper | Reject host-held identity, even after all distinguishing properties are erased and prototype changed to null, directly and nested in a result. Plain host data and fresh source copies of harmless fields remain transferable. |
| Original ordinary intrinsic identities | Global object, Math/JSON/Reflect/Object prototype and shared iterator/generator prototypes reject after removable properties are stripped and prototype flattened. Fresh source records with the same remaining shape are accepted. |

RFC §6 requires enumerable **record** properties; it does not add that requirement to dense array data elements. The tests preserve non-enumerable array data elements and frozen length without weakening the required array shape. Similarly `{0: 1, length: 1}` is a valid ordinary record, not an array; an object merely given `Array.prototype` rejects. Rejecting every record with array-looking field names would narrow the RFC's record domain, so this experiment does not do that.

All errors are typed: invalid outgoing values correspond to `INVALID_RESULT`, invalid incoming values to `INVALID_ARGUMENT`, budgets to `RESOURCE_LIMIT`. `Engine` denotes an embedding failure whose external mapping remains the existing confidential host policy; raw exception text is not read or copied. Mixed invalid/budget fault precedence is not newly standardized.

## Incoming construction and identity ownership

`Portable` is a host-owned enum of null, boolean, binary64, Rust String, Vec and BTreeMap. The complete host tree is validated before any JavaScript construction. Safe Rust makes invalid UTF-8/surrogates, duplicate map keys, functions and host cycles unrepresentable here; non-finite numbers are representable and explicitly rejected. Textual JSON duplicate rejection and operation-schema validation remain prerequisites, not bypassed by this representation. No broader value domain or input-number rounding is introduced.

Native fresh object/array allocation uses captured original prototypes. Each key is defined as an own data property; inherited `__proto__`, constructor and numeric index setters are never consulted. Rust BTreeMap iteration over valid UTF-8 orders keys by Unicode scalar sequence. The tests distinguish insertion order (`10` before `2`) from JavaScript integer-index enumeration (`2` before `10`), and test a BMP private-use scalar before a supplementary scalar, where UTF-16 code-unit ordering would differ.

Nested records/arrays, reserved/NUL keys, supplementary/decomposed strings, maximum finite and minimum subnormal numbers, and equal subtrees round-trip structurally with equal accounting. Binary64 values remain numbers; serialization is never the converter. Outgoing member order is not claimed as a Source API semantic. Source poisoning Object/Array/Reflect, descriptor helpers, Object prototype accessors, Array index setters and iterator getters cannot redirect incoming or outgoing operations. Hook counts stay zero.

Ordinary helper identities cannot be inferred from class or shape alone. The safe host-only `exclude_helper` registry holds rooted values, checks same context, and retains each nested helper before exposure; source cannot erase that metadata. It intentionally does not recursively inspect untrusted helpers to discover children. Registering all future ordinary control objects remains an embedding obligation; this experiment proves the mechanism and tests it, not service profiles. A copied scalar field conveys data, not the original helper's identity or authority.

## Cycles, expansion and bounds

An active recursion path detects direct/indirect object cycles, array self-cycles and short/long mixed cycles. Identity is removed when a branch finishes; repeated acyclic identities copy independently. Mutating one copied host subtree leaves the other unchanged.

Defaults are maximum depth 16 (root 0), 1,024 expanded values, 4,096 UTF-8 bytes per string/record key, 16,384 aggregate string/key bytes and 32,768 expanded bytes. Expanded size is an explicit experiment model: 8 bytes per portable node plus UTF-8 value and record-key bytes. It is neither normative wire size nor exact Rust heap allocation. Array length/index names are validation metadata, not outgoing record keys. Counters use checked arithmetic; depth configuration above 64 fails. Own-key count is bounded against remaining node budget before descriptor collection, allowing one extra built-in array length.

Exact depth/node/text/key/piece/expanded-size limits and one-over-limit cases pass in both directions. `{a: shared, b: shared}` with `shared = {x:1}` charges five nodes, four text bytes and 44 expanded bytes. A 64-character record fits a 200-byte budget once; 100 references to that same object fail by expanded size. Counting only unique objects would incorrectly admit that graph. No truncation, skipped fields or partial publication occurs; failure and subsequent successful conversion are independent.

Atomic native own-key enumeration and UTF-16 extraction may allocate/scan engine buffers proportional to existing input before their lengths are checked. Rust collection/copying is explicitly bounded, with finite test engine heap/stack safeguards and the external timeout. This proof does **not** establish time-to-interrupt, OOM recovery, allocator accounting, or production CPU/heap safety; those remain the next unit, rather than a claim that portable-byte budgets measure all VM work.

## Non-execution controls and unsafe audit

Working fixtures have getters/setters, serializers, coercion methods, sync/async iterators, constructor/tag/species accessors, thenables/Promise overrides, prototype getters and poisoned reflection functions. None is invoked. No jobs are drained by either converter. Negative controls intentionally read a getter, stringify `toJSON`, coerce `valueOf`, iterate a source iterator, and enumerate a Proxy before class admission; counters increase only there. The unchanged earlier blocker/class suites retain the prototype-only native false-positive and JS-level classifier controls.

Three new unsafe expressions, all in `experiments/quickjs-complete-value-spike/src/inspect.rs`:

| Location | Audited safety/ownership invariant |
| --- | --- |
| Line 56, private `descriptor` | Borrow rooted admitted object and its own live Ctx. Atom uses valid explicit-length UTF-8. Free atom on every API return; initialize public descriptor only on success. Free owned getter/setter once, move owned value once to supported `Value::from_raw`. No private object dereference. |
| Line 93, private `Utf16::drop` | Private non-Clone guard owns a non-null public UTF-16 buffer and live Ctx, releasing it once through its paired public API on success/error paths. No buffer pointer escapes. |
| Line 108, `text` | Primitive-string tag checked first. Successful pointer/length checked for usize conversion, budget and addressable slice size. Slice is UTF-16 aligned by the public API and lives only inside its guard; strict scalar decoding precedes bounded Rust String allocation. |

The reused unchanged block remains `experiments/quickjs-class-bridge-spike/src/bridge.rs:37`. Total local experimental path: **four unsafe blocks, zero unsafe functions, zero unsafe impls**; three added in this unit. Crate-wide deny with function-local exceptions; tests forbid unsafe. The safe descriptor wrapper is reachable only through the admitted object type. Context/value pairs are derived together; public converter checks context identity before inspection. Held values do not outlive rquickjs context lifetimes. Miri was not run because ordinary Miri does not validate this native C FFI path. No stronger memory-safety claim is made from ordinary test passage.

## Validation, limits and next unit

Twenty new tests pass; all 131 prior Phase 3 tests pass unchanged (including five expected compile-fail checks and the expected-panic Boa control). Runtime: 53 tests; explicit declarative rerun: two harness tests/all 34 cases; Python: six tests. New-crate and runtime fmt/strict Clippy, runtime build, all engine/parser external timeouts, and documentation/link/JSON/schema/whitespace checks pass. Exact commands and final audit counts are in the completed plan.

No production runtime/dependencies, normative specification, ADR, previous experiment or pin changed. New distinct native classes fail closed through positive class comparison; engine upgrades still require re-auditing class semantics, intrinsic capture and descriptor behavior. Cross-context import, service profiles, full result schemas/publication races and platform/distribution are not newly integrated here. The tests run on macOS arm64 and are not a JavaScript conformance suite or complete sandbox proof.

The smallest justified next unit is **CPU/allocation/memory/RegExp resource enforcement feasibility** against RFC §§5/7/10: establish interruption of CPU-only and regexp work, recoverable memory limits and trustworthy limit classification/retirement under public APIs. These risks can still invalidate the engine/binding architecture even after successful structural transfer. Production module/path containment and final parser/preflight strategy remain separate subsequent gaps. Do not select QuickJS or accept RFC 0001 on this result.
