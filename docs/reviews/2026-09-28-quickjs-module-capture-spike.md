# QuickJS pre-evaluation module capture feasibility

**Outcome A — pre-evaluation capture viable.** The pinned public APIs provide a host callback after the package graph is linked and its function declarations initialized, but before **any package module starts evaluation**. The callback obtains the real operation function objects. Post-evaluation identity checks detect top-level replacement; later assignment does not replace the saved callable. No private engine operation, source rewriting or earlier execution of source code is required.

This conclusion concerns the requested module/capture mechanism. It does not claim a finished export validator or module loader. In particular, syntax analysis is required to enforce direct-declaration shape; namespace reflection alone cannot do so. QuickJS-NG remains **unselected**, RFC 0001 remains **proposed and unchanged**, and lifecycle Model B is unchanged. No engine ADR or production JavaScript execution exists.

Baseline: `d6c18049368826e454c91ce1df06d8cc02da3701` (`test(engine): validate QuickJS fresh-realm lifecycle`), clean starting tree. Phase 3 remains active. The [prior lifecycle review](2026-09-28-quickjs-lifecycle-spike.md) correctly limited its post-evaluation reads to lifecycle evidence. This [separate experiment](../../experiments/quickjs-module-capture-spike/README.md) supplies the missing capture evidence.

## Versions, public APIs and sources

- rquickjs/core/sys **0.14.0**, revision **`d7ef5eeae702fea24c03643064de454f1c1dd4b0`**.
- QuickJS-NG **0.16.2**, revision **`0fdea21ff1090084e91dad812b343d92e79ba9d9`**; vendored by the unchanged sys version.
- Eight pinned upstream files listed below were fetched and byte-compared to registry source: all matched. No dependency versions were upgraded. `std` plus `loader` activates existing locked relative-path **2.0.1** only in the new experiment. Its lockfile equals the lifecycle lockfile except for root package name.
- Local host: macOS 27.0 build 26A428 arm64, target `aarch64-apple-darwin`; rustc 1.96.0 (`ac68faa20`, 2026-05-25), Cargo 1.96.0 (`30a34c682`, 2026-05-25), Apple clang 21.0.0. Vendored native C compiles and links on this host only; no platform/distribution evidence is inferred.

| Pinned authoritative source | Reviewed public surface and significance |
| --- | --- |
| [rquickjs module API](https://github.com/DelSkayn/rquickjs/blob/d7ef5eeae702fea24c03643064de454f1c1dd4b0/core/src/value/module.rs) | `Module::declare`, `eval`, `namespace`, `get`, `name`, `declare_def`, `ModuleDef::declare/evaluate`. Native modules supply the host intervention point. `write/load/import` do not expose declaration syntax or a link-only phase. |
| [Ctx API](https://github.com/DelSkayn/rquickjs/blob/d7ef5eeae702fea24c03643064de454f1c1dd4b0/core/src/context/ctx.rs) | `store_userdata/remove_userdata` retains then removes the private capture callback. No source global exposes it. |
| [Loader API](https://github.com/DelSkayn/rquickjs/blob/d7ef5eeae702fea24c03643064de454f1c1dd4b0/core/src/loader.rs), [BuiltinLoader](https://github.com/DelSkayn/rquickjs/blob/d7ef5eeae702fea24c03643064de454f1c1dd4b0/core/src/loader/builtin_loader.rs), [BuiltinResolver](https://github.com/DelSkayn/rquickjs/blob/d7ef5eeae702fea24c03643064de454f1c1dd4b0/core/src/loader/builtin_resolver.rs) | `Runtime::set_loader`, Resolver and Loader callbacks route fixed in-memory text; no filesystem loader or dynamic-library feature is used. |
| [Compile utility](https://github.com/DelSkayn/rquickjs/blob/d7ef5eeae702fea24c03643064de454f1c1dd4b0/core/src/loader/compile.rs) | Reports resolved module paths and serialized bytecode, not a public AST or direct-export declaration metadata. Bytecode-layout inspection is not an alternative public API. |
| [QuickJS-NG header](https://github.com/quickjs-ng/quickjs/blob/0fdea21ff1090084e91dad812b343d92e79ba9d9/quickjs.h) | `JS_Eval` with COMPILE_ONLY, `JS_ResolveModule`, `JS_EvalFunction`, module loader callbacks, `JS_NewCModule`, namespace/name/private-value and native-export functions. No public link-only or declaration-AST API. |
| [QuickJS-NG implementation](https://github.com/quickjs-ng/quickjs/blob/0fdea21ff1090084e91dad812b343d92e79ba9d9/quickjs.c) | Read-only evidence: `__JS_EvalInternal`, `js_resolve_module`, `JS_EvalFunctionInternal`, `js_create_module_function`, `js_inner_module_linking`, `js_inner_module_evaluation`, `js_execute_sync_module`, `JS_GetModuleNamespace`. None of the private functions is called by experiment code. |

The [ECMAScript 2023 InitializeEnvironment algorithm](https://262.ecma-international.org/14.0/#sec-source-text-module-record-initialize-environment) initializes function/generator/async declaration bindings during environment initialization; lexical variable/class initialization is different. The official edition was fetched directly after the browser tool rejected its size. This language rule alone does not supply host access: the native-hook tests below establish that separate fact for the pinned embedding.

## Phase model and actual intervention point

| Phase / API | Source execution and visibility observed |
| --- | --- |
| `Module::declare` → `JS_Eval`, MODULE + STRICT + COMPILE_ONLY | Parses/compiles and recursively resolves/loads dependencies already. Returns a module record; records its name. Does not run bodies or capture callbacks, initialize lexical values or enqueue source jobs in the probes. Missing dependency loading can fail here. |
| `JS_ResolveModule` | Source inspection shows graph resolution only, not function creation/linking. Compile-only already performs this resolution for source modules; a second call is not an instantiation API. Not separately called by this safe-Rust experiment. |
| `Module::eval` → `JS_EvalFunction` | Creates module functions, links the complete graph, then enters evaluation in the same call. A direct entry `eval` returns too late to recover an overwritten original export. |
| Internal linking | Establishes export binding slots and initializes hoisted function declarations. Source evidence: `js_inner_module_linking` invokes the module initialization portion, then marks modules linked. This is distinct from executing their top-level bodies. |
| First private native dependency's `ModuleDef::evaluate` | All package records are linked, but package evaluation has not been entered. The host can obtain the namespace, enumerate names and capture actual function values. Lexical const/let/class reads throw ReferenceError here. Returning a native error prevents package evaluation. |
| Package evaluation after callback returns | Dependencies run before their importers; shared dependencies run once. Top-level assignments change namespace live bindings. Graph evaluation completes synchronously for the tested no-TLA sources. Direct Promise-state inspection needs no job drain. |
| Post-evaluation check / operation invocation | Compare each namespace value with its saved Function value without invoking it. Reject changed identities. A later operation's assignment changes the namespace but not saved dispatch. |

The public Rust `namespace/get` methods are available even on `Module<Declared>` (the implementation's `Evaluated` parameter is generic). **Their availability is not evidence that an unlinked module is safe to inspect.** The pinned native namespace path expects initialized module variable references; `js_get_local_export_var_ref1` accesses function-object storage if an export slot is not yet filled. The experiment does not call this path on merely compiled records or attempt a crash/undefined-behavior proof. It calls it only from the post-link hook or after evaluation. No Rust layout access, FFI, unsafe block or bytecode parsing is added.

## Public-API strategy and RFC interpretation

The host creates an internal root with this fixed text:

```js
import '@host/capture';
import 'entry.js';
```

`@host/capture` is an export-free native module registered with `Module::declare_def`. Its synchronous evaluation callback retrieves a Rust closure from private runtime userdata. That closure captures the entry Module record, enumerates its linked namespace and stores its actual Function values in scoped Rust handles. It does not evaluate snippets, invoke exported functions, inspect source text heuristically or synthesize replacement functions. Expected names represent the manifest's operation list, not hard-coded knowledge of the source's implementation.

QuickJS links the entire root graph before evaluation. Its evaluation walk visits dependencies in import order; therefore the native module runs before entering the entry's dependency subtree. The captured entry and helper functions exist, but neither entry nor helper body has run. The host root/native module has begun engine evaluation: there is **no claim that the public API returns from a standalone instantiate call**. This is pre-evaluation capture of the **package graph**, at the instantiated state required by RFC §2/§4. Capturing after a package body had run would not satisfy that rule and is explicitly the direct-eval negative control.

The two host records are embedding machinery, not modules in the source package. The entry and all helper text remain byte-for-byte unchanged. The source's graph, exports and evaluation order are unchanged. No source-visible `init`, `initialize`, callback, service, or native-module import is allowed. The fixture resolver rejects package imports of the private name even though its native module is registered; full production preflight must also preserve the existing RFC prohibition. This is an implementation constraint, not an added author restriction. Runtime userdata is not visible to source JavaScript.

After root evaluation, the host checks Promise state directly, checks the runtime job queue without pumping, and compares saved identities. An initialization-created reaction makes initialization fail; the whole Runtime/Context is retired without running it. The native callback adds no reaction. No jobs were pending in the valid no-TLA controls, including inert Promises. A callback/shape failure or source exception rejects evaluation synchronously; a link failure returns an immediate error before capture. Existing load-vs-ready error classification remains applicable: invalid-entry diagnostic during load validation, ordinary per-call initialization `SOURCE_ERROR`, with interruption/resource rules unchanged.

The callback is removed from userdata before it runs, and any leftover callback after early failure is explicitly removed. All captured values and modules remain within `Context::with` and are dropped before Runtime teardown. This keeps the previous Model B discipline; it introduces no cross-realm cache. Tests' `mark` global and diagnostic calls of rejected originals are observation tools only, not production initialization behavior.

## Executable evidence

All tests are in [tests.rs](../../experiments/quickjs-module-capture-spike/src/tests.rs). **14 passed, 0 failed/ignored, 0 doc tests.** Names below are exact; a passing negative control means the limitation was reproduced.

| Test | Requirement and result |
| --- | --- |
| `compile_resolves_graph_without_body_or_job_execution` | Entry compilation loads the dependency and returns a record/name; compiling the private root still causes no capture/body execution. No job is queued. |
| `native_hook_captures_all_direct_sync_and_async_declarations_before_bodies` | All five operation names, mixed normal/async direct declarations, are captured before the entry's first marker. Identity survives evaluation; saved home and async category are callable afterward. |
| `top_level_replacement_is_detected_against_real_precaptured_identity` | Function and nonfunction replacements both fail identity checks. Diagnostic invocation of the saved value still yields `original`, proving it is the pre-replacement object. |
| `later_operation_assignment_changes_namespace_but_not_captured_dispatch` | Initial identity matches; invoking saved home changes its live export to `replacement`; repeated saved calls still execute `original`. |
| `direct_eval_negative_control_loses_original_before_host_can_read_exports` | Without the native hook, first host access after `eval` observes only the replacement and the body has already run. This is not an acceptable fallback. |
| `lexical_exports_are_uninitialized_at_capture_and_fail_before_bodies` | `export const home = 42`, exported class, and exported let/function assignment compile but raise ReferenceError during capture; no body executes. These are invalid RFC entries, not alternate operation forms. |
| `namespace_callable_checks_cannot_validate_generator_alias_or_reexport_syntax` | Generator, async generator, alias, same-name export-list and helper re-export all yield callable named home at the hook and pass identity after evaluation. Reflection is insufficient to reject these RFC-invalid forms. |
| `default_extra_and_missing_export_names_fail_before_source_bodies` | Default, additional and missing names fail the name-set check at the native hook; source bodies never run. |
| `duplicate_exports_fail_compile_and_missing_import_fails_link_before_capture` | Duplicate export fails declaration/compilation; a missing named dependency export fails linking in `eval`. Neither source bodies nor the capture callback run on link failure. |
| `shared_dependency_loads_and_evaluates_once_after_capture` | Fixed entry→dep→leaf plus entry→leaf graph loads each helper once. Trace: load dep, load leaf, capture start/done, leaf body, dep body, entry body. Saved function reads both initialized imports. Re-evaluation is cached within this realm only. |
| `initialization_jobs_are_detected_and_abandoned_without_pumping` | Top-level reaction appears only during evaluation, remains pending after synchronous graph completion and never fires through context/runtime destruction. Initialization must be rejected. |
| `inert_promise_and_source_throw_keep_initialization_synchronous` | An inert Promise creates no job; ordinary initialization throw produces an immediately rejected evaluation Promise after successful pre-body capture. |
| `source_cannot_import_private_capture_module` | A source import of the registered private name fails compile-time resolution, before capture or body execution. |
| `lexical_values_appear_only_after_evaluation_negative_control` | Direct eval exposes the replacement let function, const 42 and class constructor only afterward. A class is function-shaped to the value predicate, so “is function” is not declaration validation. |

## Syntax metadata boundary and smallest alternative

The public module APIs provide names/values, not whether an export came from a direct declaration, export list or re-export. Function names, types, constructors and source-string reflection cannot establish that surrounding module syntax. The passing same-name export-list/re-export controls make that distinction concrete. Default/extra/missing names can be rejected by names alone; lexical forms fail early in this probe, but those failures are not a substitute for preflight syntax rejection.

For this public-API strategy, **pre-execution source-static analysis is required**, in addition to QuickJS compilation. This is compatible with RFC §1's existing complete-graph preflight over immutable text (which must already detect forbidden syntax and cycles). An AST can distinguish direct FunctionDeclaration/AsyncFunctionDeclaration exports and the forbidden forms, validate names against the manifest, and feed the unchanged bytes to QuickJS. It must run before root evaluation; it cannot recover the actual function objects or replace the native hook.

As source-level feasibility evidence only, the already present Boa 0.22.0 AST exposes separate [ExportDeclaration variants](https://github.com/boa-dev/boa/blob/337a3668a0dc86dd401ea20906e782249a64a228/core/ast/src/declaration/export.rs) for Declaration, List, ReExport, variables and default forms; declaration nodes distinguish normal/async/generator/class forms. This is an example of a realistic AST representation, **not parser selection, an executed parser experiment or proof of complete ES2023/QuickJS agreement**. No parser dependency was added. Regexes, engine-private bytecode formats, reading private export tables and modifying QuickJS are not proposed substitutes.

The need for syntax analysis is established; a particular parser and its compatibility are not. That dependency/complexity is an engine-selection evidence gap rather than a capture-API blocker. The demonstrated capture strategy works on arbitrary preflight-valid DAG packages using their manifest names and module records; it does not require author annotations or forbid otherwise valid top-level assignments. A full parser/validator is deliberately outside this experiment.

## Limits, conclusion and next unit

No pre-evaluation function-capture blocker remains for the demonstrated pinned public-API strategy. It satisfies the required observable ordering without weakening RFC semantics. The direct API does not expose link-only execution, but the supported native-module hook supplies host access before package evaluation; Outcome A does not depend on a hypothetical future engine API.

Remaining work includes preflight parser compatibility, full snapshot/path policy, restricted globals/dynamic compilation, complete non-executing conversion and resource policy, platform/build/distribution proof, conformance and service profiles. None was implemented or re-evaluated here. No portable heap, hard-real-time, sandbox, performance or cross-platform claim is made. The source-marker tests and source review demonstrate phase order, not comprehensive malicious-module conformance.

**Next task: one narrow static-analysis feasibility experiment** that tests a candidate parser's ES2023 module/export AST against the pinned QuickJS compiler and RFC forbidden syntax, including escaped identifiers and nested syntax. Establish exact-text/no-rewrite agreement and bounded preflight feasibility before choosing that dependency. Do not begin production execution, a production module loader, engine selection or RFC acceptance.

Exact validation commands/results and the complete file inventory are in the [completed plan](../plans/completed/2026-09-28-quickjs-module-capture-spike.md). Existing runtime behavior/dependencies, RFC/specification, ADRs and conformance expectations remain untouched.
