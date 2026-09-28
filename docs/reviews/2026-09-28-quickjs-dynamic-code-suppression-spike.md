# QuickJS dynamic JavaScript compilation suppression

## Conclusion and scope

**Outcome A — dynamic compilation suppression viable.** The pinned public embedding API plus standard ECMAScript object operations can establish call/construct guards before package code runs. Fourteen tests cover 80 blocked routes, exact native EvalError prototype identity, absence of dynamic payload effects, mutation resistance, host compilation and pre-evaluation capture, and fresh-realm reset. No demonstrated dynamic-compilation bypass remains in scope.

Baseline: `de2c76972af070a1715d3db948c1cf600d050a82`, `test(parser): prove import-attribute clause detection`; clean main and no active plans at start. Phase 3 remains active. [RFC 0001](../../spec/rfcs/0001-javascript-execution-binding.md) remains proposed and unchanged. No engine/parser selection, ADR, production execution or complete sandbox claim follows. Lifecycle Model B, prior capture strategy and parser experiments are preserved.

## Pins and authoritative source evidence

| Component | Exact evaluated version / revision |
| --- | --- |
| rquickjs/core/sys | `0.14.0`; `d7ef5eeae702fea24c03643064de454f1c1dd4b0` |
| QuickJS-NG | `0.16.2`; `0fdea21ff1090084e91dad812b343d92e79ba9d9` |
| Local environment | Rust/Cargo `1.96.0`; rustc `ac68faa20c58cbccd01ee7208bf3b6e93a7d7f96`; macOS `27.0` build `26A428`, `aarch64-apple-darwin` / arm64 |
| Dependency change | New isolated crate pins rquickjs `=0.14.0`, `std`/`loader` only. Its lock equals the module-capture experiment's after replacing only the root package name. No upgrade or production dependency change. |

Six pinned upstream files below were fetched and byte-compared against registry/vendor source: all matched. Source reading explains the exercised public interfaces; no private function, engine struct, pointer offset or patched code is used.

| Public/source evidence | Finding |
| --- | --- |
| [QuickJS header](https://github.com/quickjs-ng/quickjs/blob/0fdea21ff1090084e91dad812b343d92e79ba9d9/quickjs.h) | Public intrinsic installation and JS_Eval/JS_EvalFunction/module APIs, but no source-only eval-disable toggle or supported per-context compiler-hook replacement. Eval flags describe host evaluation modes, not an untrusted-source compilation policy. |
| [QuickJS implementation](https://github.com/quickjs-ng/quickjs/blob/0fdea21ff1090084e91dad812b343d92e79ba9d9/quickjs.c) | `JS_AddIntrinsicEval` installs compiler support. `JS_EvalInternal` throws TypeError when absent. `js_global_eval` routes to `JS_EvalObject`. All four `js_function_constructor` registrations share dynamic source construction/compilation, with normal/async/generator/async-generator kinds. |
| [rquickjs context builder](https://github.com/DelSkayn/rquickjs/blob/d7ef5eeae702fea24c03643064de454f1c1dd4b0/core/src/context/builder.rs) | `Context::full` installs standard intrinsics; `Context::custom` can omit `intrinsic::Eval`. No separate safe-Rust source-eval policy switch is exposed. |
| [Ctx API](https://github.com/DelSkayn/rquickjs/blob/d7ef5eeae702fea24c03643064de454f1c1dd4b0/core/src/context/ctx.rs) | Host `Ctx::eval` executes a fixed trusted bootstrap independently of the JavaScript global eval property. Scoped runtime userdata holds the test-native capture callback. |
| [Module API](https://github.com/DelSkayn/rquickjs/blob/d7ef5eeae702fea24c03643064de454f1c1dd4b0/core/src/value/module.rs) | `Module::declare`, `declare_def`, `eval`, `ModuleDef::evaluate` preserve the prior host-controlled compile/link/capture mechanism after hardening. |
| [Function API](https://github.com/DelSkayn/rquickjs/blob/d7ef5eeae702fea24c03643064de454f1c1dd4b0/core/src/value/function.rs) | `Function::new` creates the diagnostic native callback after hardening; its constructor path also reaches guarded Function. No callback exposes a compiler. |

The engine source also establishes the relevant intrinsic graph. Function owns the ordinary function prototype; AsyncFunction, GeneratorFunction and AsyncGeneratorFunction constructors inherit from **Function itself**, while their function prototypes inherit from Function.prototype. Generator/async-generator object constructor paths pass through those prototypes. Merely replacing global Function and `.constructor` properties would leave native Function reachable as the specialized constructors' parent. A passing negative control reproduces that leak.

QuickJS `js_proxy_call` / `js_proxy_call_constructor` dispatch to apply/construct traps before forwarding to the target. The guards always throw; forwarding is never reached. Bound-function invocation delegates to its saved target, preserving the guard rather than unwrapping it. Internal function-realm lookup can traverse proxies for engine bookkeeping but does not return a compiler handle to JavaScript. Source cannot obtain proxy targets/handlers through these standard APIs.

## Host hardening sequence

The fixed [harden.js](../../experiments/quickjs-dynamic-code-spike/src/harden.js) bootstrap is trusted host code, not rewritten package code:

1. Create a fresh Runtime/Context with the intended intrinsics installed. No package function, job or initialization has run, and no source reference exists.
2. Through host `Ctx::eval`, capture native EvalError and the four native constructor families. Create a hidden null-prototype handler with apply and construct traps that immediately throw `new NativeEvalError(...)`. Freeze that handler only; source cannot reach it. A null prototype prevents later Object.prototype poisoning from supplying missing traps.
3. Wrap each constructor and eval with a standard Proxy using that handler. A constructor proxy preserves its original `.prototype`, name, length, constructibility and function relationships while blocking both invocation modes. No original callable is returned to source.
4. Change each specialized native constructor's parent to guarded Function, closing the parent-link leak. Replace the four function-prototype `.constructor` links with their corresponding guard. Make those properties and the global eval/Function bindings non-writable, non-configurable and non-enumerable.
5. Install only non-compiling host bindings, compile immutable package bytes, link/capture the operation functions using the existing host-native hook, then evaluate the package. All source aliases obtained during initialization are already guarded.

The guard closure retains its native EvalError reference; replacing the source-visible EvalError binding cannot change what it throws. The hidden handler ignores target/arguments/newTarget supplied by Proxy dispatch and never forwards them. No host service or callback may expose original constructors, handler objects, compiler entry points or later compiler-bearing intrinsic installation. Installing new intrinsics that reintroduce native constructor links would violate this setup discipline; the proof does not harden arbitrary future host plugins.

Proxy is an implementation mechanism here. RFC §9 excludes the source-visible Proxy global; deleting that global after bootstrap leaves these internal guards working. A stronger-than-profile diagnostic additionally lets source wrap guards in new proxies and verifies forwarding still cannot reach the native compiler. The experiment otherwise uses `Context::full` and intentionally does **not** remove every forbidden global/extension or suppress stacks. That larger profile remains a separate feasibility unit.

Omitting Eval instead is not the chosen strategy: both host script and module compilation fail with TypeError. Compile-time parser exclusion likewise is not a source/host separation and would change the pinned build. No disable/reenable window, bytecode relocation or raw engine hook is needed.

## Tested routes and static/runtime boundary

[probes.js](../../experiments/quickjs-dynamic-code-spike/src/probes.js) contains the complete named 80-route matrix. Every guarded attempt throws an object with the saved native EvalError prototype; no message-string comparison substitutes for class identity. Every attempt checks that both marker globals remain absent.

| Route family | Static detectability | Runtime result with valid call/construct inputs |
| --- | --- | --- |
| Direct eval, comma eval, global eval, saved alias, descriptor lookup | Direct spelling observable; alias flow not established by name matching | EvalError; arithmetic and marker text never evaluated. |
| Function/new Function, saved Function | Direct spelling observable, subject to shadowing | EvalError before compilation. |
| Ordinary/arrow function constructor, prototype lookup, descriptor lookup | Syntactic access observable; target identity requires runtime | EvalError. |
| AsyncFunction, GeneratorFunction, AsyncGeneratorFunction, async arrow | Constructors derived through intrinsic prototypes, no assumed globals | EvalError for calls, new and descriptor/prototype paths. |
| Function.constructor, constructor.constructor, object/Error/builtin method/getter/bound-function chains | Not reliably forbidden by simple static names | EvalError; no native constructor recovered. |
| Generator and async-generator object prototype chains; specialized constructor parent | Runtime object graph | EvalError; explicit parent redirection is necessary. |
| Reflect.construct / Reflect.apply for every constructor family; Reflect.apply(eval) | Reflect target can be an arbitrary alias | EvalError. |
| `.call`, `.apply`, `.bind`, bound construction for all four families; eval call/apply/bind | Indirection requires runtime guard | EvalError. |
| Class extending guarded Function | Parent can be aliased | Construction throws EvalError. |
| Invalid source strings, no Function arguments, nonstring eval argument | Not a basis for compilation authority | EvalError under the RFC's call prohibition, not SyntaxError or ordinary eval pass-through. |

RFC §9 requires attempted dynamic compilation to throw; it does not require banning `eval`/`Function` identifier syntax. No parser dependency or static scan is used in this spike. Static dynamic-import/import.meta restrictions from prior work remain independent; this task neither permits package-driven dynamic module loading nor implements its loader policy.

## Error timing, ordinary behavior and mutation

The guard runs before native constructor string coercion/compilation. Malformed strings produce EvalError rather than a parser SyntaxError. Object arguments with observable `toString` and a proxy newTarget with an observable prototype getter show no such effects on reaching the guard. Supplied dynamic strings never create `__dynamic_code_executed` or `__pwned`. Proxy machinery may allocate an argument array/error, so this is not a zero-allocation or exact resource claim. Ordinary argument evaluation, Reflect argument-list getters or invalid call arguments can execute/throw before the target is called; the RFC does not turn all expression evaluation into EvalError. Resource exhaustion remains separately classified and unproven here.

The source-visible guarded Function still has Function.prototype, name Function and length 1; eval retains its name/length and nonconstructible nature. Family-specific prototype/constructor identities and `instanceof` relationships hold in the tested graph, including specialized constructor parents. Ordinary functions, bind/call, classes, Reflect construction of a normal constructor, generators and RegExp data compilation still work. Only dynamic callable behavior, the required protected links and specialized parent references change. Prototype objects remain mutable for ordinary invocation-local state; no unrelated globals/prototypes are frozen. Function source formatting is not a portable identifier under the RFC. The proof uses guarded replacements rather than deleting Function or collapsing all four families into one unrelated stub.

Mutation probes attempt strict assignments, Reflect.set, defineProperty replacements/getters and deletion of the globals and constructor links; these fail as expected. Reapplying descriptors with the same recovered references only restores the guards. They also mutate prototype chains, install generic Object.prototype traps, change unrelated Function.prototype state, replace public Reflect/descriptor helpers and replace global EvalError. The matrix is rerun using saved trusted observation helpers. Replacing public Reflect with a harmless source function can of course return its own value; it does not create compiler authority, and captured native Reflect still reaches the guard. Source-defined lexical bindings or methods called eval/Function are likewise ordinary code, not original compiler handles.

A separate package's top-level body performs hostile mutation before calling the saved probes, with all 80 routes still denied. Across two fresh Runtime/Context pairs, the hostile first invocation leaves no global/prototype marker in the second; each is hardened independently and passes the matrix. No realm reuse or selective cleanup is introduced.

## Executable evidence and negative controls

All fourteen [tests](../../experiments/quickjs-dynamic-code-spike/src/tests.rs) pass; no ignored tests or doc tests.

| Test | Evidence |
| --- | --- |
| `all_dynamic_routes_throw_evalerror_without_payload_execution` | 80 required/direct/derived/descriptor/indirect routes, exact class and absent effects. |
| `hostile_mutation_cannot_restore_compilation_and_saved_aliases_stay_denied` | Full matrix after descriptor/global/prototype/helper mutation. |
| `denial_precedes_argument_coercion_and_newtarget_prototype_access` | No coercion/newTarget getter effects; malformed text denied before parsing. |
| `unchanged_intrinsic_relationships_and_ordinary_function_behavior` | Required prototype/family relationships and ordinary code remain usable. |
| `fresh_invocation_is_hardened_independently_after_hostile_realm_retirement` | Model B reset and independent hardening, no cross-invocation state. |
| `guards_do_not_require_source_visible_proxy_or_mutable_reflect_helpers` | Proxy global deleted and Reflect helpers overwritten; saved operations still deny. |
| `public_proxy_cannot_unwrap_a_guard_and_alternative_prototype_mutation_does_not_compile` | Additional proxy wrapping/forwarding and null prototype changes do not recover native targets. |
| `hardened_host_compile_link_capture_and_package_initialization_order` | Observed `harden → bindings → compile → capture → package`; native hook captures actual home before body, checks its guarded constructor, then home returns 42. Source initialization's eval/Function/native-callback-constructor attempts are blocked. |
| `hostile_package_top_level_mutation_keeps_initialization_aliases_guarded` | Unchanged fixed entry imports diagnostic probes, mutates during evaluation and checks all 80 denials synchronously. |
| `unhardened_negative_controls_execute_payloads_for_all_four_families` | Direct/indirect eval and Reflect construction of all families execute harmless payloads; six markers, fixed result 123. Bounded job pumping is used only for this negative async control. |
| `deleting_only_globals_leaves_constructor_compilation_authority` | Removing eval/Function alone still allows arrow.constructor payload execution. |
| `hardening_too_late_cannot_revoke_an_already_leaked_native_eval` | Deliberately saved pre-bootstrap eval still executes afterward. Required pristine-realm ordering is essential. |
| `failing_to_redirect_specialized_constructor_parents_leaks_native_function` | Intentionally incomplete guard leaves native Function reachable from an async constructor's parent; payload executes. The actual bootstrap redirects all three parents. |
| `missing_eval_intrinsic_disables_host_compilation_with_wrong_error_class` | Custom context without Eval rejects host script/module compilation with TypeError; not an RFC solution. |

The capture fixture uses fixed local module records and private Rust userdata; diagnostic `mark` has no compile capability. Its native Function constructor route is tested. The package byte vector is compared exactly to the immutable fixture before `Module::declare`; no eval/constructor references are rewritten. Extra diagnostic exports/markers are test instrumentation, not Source API integration. The earlier production preflight restrictions and private-module containment requirements still apply to any future integration.

## Limits and next unit

No bypass remains in the demonstrated surface, but the result is a public-API feasibility argument with focused tests, not a proof against every engine defect. It depends on pristine setup, no leaked native targets/handlers, no source-visible compiler bridge, and no late intrinsic installation restoring links. These are host embedding obligations, not new restrictions on valid package source. Engine upgrades would require rechecking intrinsic reachability and the route matrix.

This unit does not validate the complete global allowlist, extension/stack removal, clocks/randomness/locale suppression, complete value conversion, memory/CPU/regexp limits, parser strategy, production module/path containment, service profiles, platform/distribution behavior or conformance. Guard implementation via Proxy does not require exposing the Proxy constructor to source and does not loosen result-copy restrictions on proxies. Error text remains implementation-specific; the tested class/compilation denial follows the unchanged RFC.

Next coherent task: **broader restricted-global and ambient-authority surface feasibility** against RFC §9 on these pinned APIs, including transitive intrinsic extensions, stack visibility, clock/randomness/locale exclusions and preservation of this suppression mechanism. Keep it a scoped experiment, not a production sandbox or engine-selection decision. Other resource/value/platform/service gaps remain separate.

Exact validation commands/results and changed files are in the [completed plan](../plans/completed/2026-09-28-quickjs-dynamic-code-suppression-spike.md). Production runtime, dependencies, RFC text/status and prior experiments remain unchanged. No engine selected or ADR created; no push, tag or release.
