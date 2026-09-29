# RegExp compilation remedy review

## Outcome and scope

**Outcome A — pre-admission viable at design level.** RFC 0001 does not require interrupting the RegExp compiler mid-compile. It requires published finite bounds, a documented checkpoint policy with eventual interruption, and classified failure for checked limits. A host can meet that by refusing to hand any pattern longer than a documented UTF-16 length bound to the non-polling pinned compiler, and by checking its trusted deadline/cancellation state immediately before every admitted compile. Every compile path identified in the pinned source is at most quadratic in pattern length, so a length bound yields a finite worst-case non-polling interval. All compile entry points are finite and enumerable. Pattern text is an ordinary coerced string before the native compiler. Every route that bypasses a replaced global is a known built-in that trusted code can also wrap. Nothing here is implemented; the wrap set and classification path carry real risk and must be proven by the next spike.

The original resource **Outcome B stands unchanged**. Pinned native QuickJS RegExp compilation itself does not poll the tested interrupt handler on the reproduced path. The [resource review](2026-09-29-quickjs-resource-spike.md) is not rewritten, and this outcome does not make the pinned engine interruptible mid-compile. It concludes that a bounded-admission architecture around it can satisfy the unchanged RFC if the follow-up spike proves it.

Baseline: `8f6cafa67b7e53d95d54d60200acac852fd78ac1` (`test(engine): expose RegExp compilation resource blocker`), clean tree, no other active plan. Phase 3 remains active. [RFC 0001](../../spec/rfcs/0001-javascript-execution-binding.md) remains proposed and unedited; no engine or parser is selected; no ADR exists. Evidence comes from a pinned-source audit plus the eight-test [admission route probe](../../experiments/quickjs-regexp-admission-spike/README.md). This task accessed no external system: upstream evidence is limited to installed registry sources, and ES2023 algorithm steps are cited from the specification text as known to the reviewer, not re-fetched.

## Pins and local provenance

rquickjs/core/sys **0.14.0** (`d7ef5eeae702fea24c03643064de454f1c1dd4b0`), QuickJS-NG **0.16.2** (`0fdea21ff1090084e91dad812b343d92e79ba9d9`), Boa parser/AST/interner **0.22.0** (with `regress` 0.12.0), Oxc **0.152.0**. Unchanged; the probe's lockfile equals the static-analysis spike's lockfile after renaming only the root package.

| Registry-relative source | SHA-256 |
| --- | --- |
| `rquickjs-sys-0.14.0/quickjs/quickjs.h` | `979127138da79cad5ddc7effa1afb13e8b02f744da2aa36844d4cb9d8cb92de9` |
| `rquickjs-sys-0.14.0/quickjs/quickjs.c` | `3a6b52a225c21709ef1fd314183efd3af862e867ce4ef8caabcae67922a745b3` |
| `rquickjs-sys-0.14.0/quickjs/libregexp.c` | `af13e996abb1767fe0cdfda123c45b811d53688d403a2ac47142e97bf79f1fae` |
| `boa_parser-0.22.0/src/lexer/regex.rs` | `3c10f238b3ee6fb04325ae0e50b9bcaf6b7022ae3d01d57e1039fb511205f892` |
| `boa_ast-0.22.0/src/visitor.rs` | `deda270e4b59840708ce25c35921c8540cc17b7cb95c5331ce77ac0777f3482c` |
| `oxc_parser-0.152.0/src/lexer/regex.rs` | `84a4368587ecfdb7d471a02f8e8c96bae815c226484247f07a6635b772b662d5` |

The three QuickJS fingerprints equal the resource review's table. Line numbers below refer to these local files. Environment: macOS 27.0 (26A428), arm64, Rust/Cargo 1.96.0, debug test profile.

## The blocker still reproduces

`python3 experiments/quickjs-resource-spike/probe_regexp.py` was rerun unchanged. 1,000 / 4,000 / 16,000 forward references returned in 17 / 143 / 1,682 ms with zero callbacks. 64,000 references printed `ENTER`, then hit the external five-second kill: **exit 2**, no callback. The 11 resource tests pass unchanged. The external kill remains a test safety net, never engine evidence.

## Does the RFC require mid-compile interruptibility?

No, provided the non-polling interval is finite, documented and demonstrated.

- §5: "Hosts must document checkpoint policy, including how running source code reaches checks (engine hooks, **bounded execution slices**, or instrumentation…)." "Finite deadline enforcement means no late result acceptance and **eventual interruption under that policy**; it is not a hard real-time completion-latency guarantee. Hard preemption, precise instruction interruption and cross-platform enforcement are **not claimed**."
- §9: "RegExp pattern compilation is allowed as **bounded** data processing."
- §10: hosts publish finite bounds including the "engine resource policy"; "a credible bound for source-created allocations and CPU work" is required; "checked limits yield classified failure."
- The Host API already requires documented quotas, and sources must tolerate `TIMEOUT` and `RESOURCE_LIMIT`.

The existing checkpoint mechanism already has non-polling intervals: 10,000 bytecode operations, one call into any native built-in, and module parsing (which never polls and is bounded only by module bytes). The blocker is that this compile interval had **no finite bound**: cost grows as O(L²) in pattern length L, and L was limited only by heap size. Rejecting patterns above a documented `L_max` before the compiler, and checking deadline/cancellation before each admitted compile, turns compilation into a bounded execution slice under a published policy. That uses existing RFC semantics and adds no portable RegExp restriction. `L_max` is host engine resource policy, like the heap limit. The failure is the existing load resource diagnostic or `RESOURCE_LIMIT`.

One interpretive risk is surfaced rather than resolved silently. The conformance plan says the harness "must also demonstrate real checkpoint paths for CPU-only source work and engine internals such as regexps." This review reads a bounded slice between a host check and the next engine poll as such a path. A reader who requires a checkpoint *inside* compilation would instead reach Outcome D for the pinned engine, or C/B. That reading would need an explicit RFC decision, not an experiment result.

The bound is only as good as its enumeration. A single unwrapped route to the compiler with an unbounded string restores the blocker. Route completeness is therefore the main risk and the next spike's central proof.

## Complete compile-route inventory (pinned source)

| Entry | Source | Reached by | Admission point |
| --- | --- | --- | --- |
| Literal at parse time | Lexer calls `ctx->compile_regexp` for every literal (`quickjs.c:27109`); `OP_regexp` later reuses precompiled bytecode (18774) | Every literal in every parsed module, reachable or not | Host preflight before `Module::declare` |
| `js_regexp_constructor` (49310) → `js_compile_regexp` (49155) → `lre_compile` (`libregexp.c:2577`) | Global binding, `prototype.constructor`, species | `RegExp()`, `new`, aliases, `call`/`apply`/`bind`, `Reflect.construct`, subclass `super`, clone with flags | Wrapped constructor |
| Same constructor via internal `ctx->regexp_ctor` | `js_string_match` (47516; construct at 47554) | `String.prototype.match`/`matchAll`/`search` with an argument lacking the symbol method | Wrapped `String.prototype` methods |
| Same constructor via species default `ctx->regexp_ctor` | `Symbol.matchAll` (50219/50241), `Symbol.split` (50615/50638) | Generic receiver: arbitrary `toString` text. Genuine receiver: recompiles its internal, already admitted source with new flags | Wrapped methods; generic receivers need the facade |
| `js_regexp_compile` (49388) | Annex B `RegExp.prototype.compile`, retained by the allowlist | Genuine `this`, string pattern | Wrapped method |
| `JS_DetectModule` (64817) | Public host helper; fresh runtime, **no limits or handler** | Host only | Hosts must not call it on package text |

Not compile routes: `exec`/`test`/`Symbol.match`/`Symbol.replace`/`Symbol.search` (execute existing bytecode); string-argument `replace`/`replaceAll`/`split` (string search); genuine clone without flags (bytecode reused); `JS_ReadRegExp` (40259, deserialization, not source-reachable). Direct/indirect `eval`, `Function`-family constructors and dynamic `import()` are already denied or rejected, so no literal can be compiled at run time.

The probe confirms every class against pristine natives. With global `RegExp` and `RegExp.prototype.constructor` replaced by a refusing function, it observes:

- **Reach the replacement:** `new RegExp("(")`, `RegExp("(")`, `Reflect.construct`, a bound constructor, `class extends RegExp`, `(/x/).constructor`, and species resolving to the replacement.
- **Native `SyntaxError` without the replacement ever running:** `"a(b".match("(")`, `matchAll`, `search`, `/x/.compile("(")`, and generic-receiver `Symbol.split`/`Symbol.matchAll`.
- **No compile of `"("`:** string `split`/`replace`/`replaceAll`, `exec`, `Symbol.replace`, and a genuine receiver whose own `source` getter is overridden. That last case proves internal-source reuse.

## Candidate A: static literal preflight

**Finding: viable.** Pinned QuickJS compiles every RegExp literal while parsing, before any bytecode runs. A 20,007-unit literal compiles in ~0.1 s during compile-only `Module::declare` with zero callbacks. An invalid final character pays the same cost before `SyntaxError`, so syntax validity does not bound cost. Literals must therefore be admitted from exact bytes before QuickJS sees the module.

- **Positions.** `/(/` fails declaration everywhere: entry module, imported helper, unreachable `if (false)` branch, nested function, arrow, class method, static field, template substitution, statement after `if`, default parameter, generator, async function and object/array value. Boa 0.22's public visitor (`visit_reg_exp_literal`) finds every literal in all eleven positions.
- **Lexical ambiguity.** Fourteen fixtures give the same literal lists in QuickJS, Boa and Oxc:
  - division forms that are not literals: `4 /2/ 1`, `a /(b)/ g`, `({}) / 2`, `x /= 2`, `x /2/ 1`;
  - literal forms: block-then-literal, `if (x) /y+/g`, `/=a/`;
  - escapes and flags: `/\//`, `/[/]/`, `/\u{61}\u0062/u`, all ES2023 flags (`dgimsuy`), and a non-ASCII body;
  - slashes inside comments, strings and templates.

  Bodies are raw, verbatim text; the exact bytes are never rewritten. Length is the UTF-16 code-unit count of the raw body, which QuickJS compiles.
- **Syntax-invalid literals.** Boa and Oxc (validated mode) reject `/(/`, and QuickJS rejects it at declaration. Oxc raw mode (`parse_regular_expression: false`) does not validate patterns. Validity is a separate acceptance question; the resource bound must precede any expensive validation.
- **Preflight cost (new risk).** Boa always validates literal bodies with `regress` (`boa_parser` `regex.rs:150`). On this family it took 44 ms at 80,007 units and ~550 ms at 320,007 units: about 12× for 4× input, a two-point debug-build observation with the cause not diagnosed. Oxc raw mode took 0–2 ms and validated mode 12 / ~50 ms, both linear. The bounded Rust preflight is not the unbounded engine, but a parser that validates before measuring moves superlinear work into loading. The implementation must measure literal length before, or without, pattern validation, or rely on a module byte bound small enough to cap validator cost.
- **Classification.** A literal over `L_max` is a loading engine budget failure (§7 "Loading structural/engine budget exceeded → Resource-limit load diagnostic") with no QuickJS parse. Instance health is unaffected.
- **Parser disagreement.** If the preflight parser treats a slash as division where QuickJS sees a literal, a literal escapes measurement. The existing static-analysis design already needs exact-byte parser agreement. The spike must fail closed when literal lists disagree, for example Boa's visitor list against a second parser's literal list, and keep the module byte bound as defense in depth.

**Relationship to byte limits.** A module byte bound B alone is finite: no literal exceeds B, so parse-time compile per module is at most c·B² for the known quadratic families. That is too loose at realistic B: extrapolating this family, 1 MiB is about 280 s. With preflight, Σ L_i² ≤ L_max · Σ L_i ≤ L_max · B, so total literal compile cost per module parse is at most c·B·`L_max`. Under lifecycle Model B that cost recurs for each fresh-realm parse unless isolated bytecode sharing is used, so it counts inside each invocation's deadline.

## Candidate B: dynamic construction facade

**Finding: viable, with a finite but non-trivial wrap set.** Dynamic patterns are bounded only by heap size (8 MiB gives about 4M code units), so literal preflight cannot cover them.

### Construction forms

| Form | Route observed | Facade obligation |
| --- | --- | --- |
| `RegExp(p)`, `RegExp(p, f)` | Replaced global | ES2023 call form: if `p` is RegExp-like and `f` is undefined, return `p` when `p.constructor` is **the source-visible RegExp**. The facade must perform this identity step itself; delegating the call form would compare against the native identity. |
| `new RegExp(p[, f])` | Replaced global | Construct with `newTarget`. |
| Aliases (`const R = RegExp`), `call`/`apply`/`bind` | Same function object | No extra route. |
| `Reflect.construct(RegExp, args, nt)` | Replaced global | Forward `nt`. |
| `class X extends RegExp` | `super()` reaches the facade with `newTarget = X` | Forward `newTarget`; `X.prototype` inherits the retained prototype. |
| `(/x/).constructor`, `RegExp.prototype.constructor` | Replaced link | Point the link at the facade. |
| Species (`Symbol.species` getter returns `this`) | Facade when resolvable | Retain the native getter. |
| Clone `new RegExp(r)` / `new RegExp(r, f)` | Genuine: bytecode reused, or internal source recompiled with new flags | Already admitted source. The cost bound must hold for the **worst flags**, since flags can change (`i`, `u`/`v`). |

### Coercion and the admission boundary

The ES2023 constructor performs every source-observable step before `ParsePattern`: IsRegExp (Get `@@match`); the call-form `constructor` read; for RegExp-like objects, Get `source` and `flags`; `RegExpAlloc` (Get `newTarget.prototype`); then ToString(pattern) and ToString(flags). The probe observes `isRegExp, pattern:string, flags, SyntaxError`, so the final string exists before the compiler. A facade can do these steps **once**, in order, measure the resulting primitive pattern, and pass primitive strings to the native constructor. The native IsRegExp/ToString on primitives observes nothing, so object patterns and `toString` coercion stay supported. This is not a primitive-only restriction; RFC 0001 does not permit one and none is proposed.

Two pinned deviations must be handled deliberately:

- QuickJS reads `newTarget.prototype` only after a successful compile. The probe logs `pattern, SyntaxError` with no `prototype` read. A facade cannot reproduce either order without a second observable read unless it allocates through the native constructor and assigns the prototype itself.
- QuickJS implements String `RegExpCreate` through its full constructor. On `r[Symbol.match] = undefined`, `"/a/".match(r)` returns `a` (internal source); the ES2023 `RegExpCreate(r)` stringifies `r` and returns `/a/`.

For genuine RegExp arguments the facade must avoid repeating the IsRegExp read. The native `source` getter returns escaped text, whose length can exceed the originally admitted length. The spike must choose and test one representation.

### Access to the native constructor

After all intrinsics and hardening, and before any package code runs, trusted setup must:

- hold the native constructor only in a host-owned persistent reference;
- replace the global `RegExp` and `RegExp.prototype.constructor` with a host-native constructor function whose `prototype` is the original prototype object, with matching `name`, `length`, property attributes and species accessor, so `instanceof`, `Object.getPrototypeOf(/x/)` and subclassing are preserved;
- prove with the existing bounded reachability walk that the native constructor is unreachable and cannot be restored.

The engine's internal `ctx->regexp_ctor` cannot be redirected through the public API. `JS_AddIntrinsicRegExpCompiler` only installs the built-in compiler; there is no public compile hook. That is why the secondary routes must be wrapped. A side effect: `js_is_standard_regexp` (50343) compares `constructor` with the internal identity, so the facade disables that performance fast path, not correctness.

### Secondary-route table

| Route | Compiles source text? | Reaches the source-visible RegExp? | Required handling |
| --- | --- | --- | --- |
| `String.prototype.match(x)` | Yes, when `x` lacks `@@match` | No: internal constructor | Wrap: ES `RegExpCreate(x)` through the facade, then invoke `@@match`. |
| `String.prototype.matchAll(x)` | Yes (flags `"g"`) | No | Wrap likewise. |
| `String.prototype.search(x)` | Yes | No | Wrap likewise. |
| `String.prototype.replace`/`replaceAll` | No (string search or delegation to `@@replace`) | Not applicable | None. |
| `String.prototype.split(x)` | Only via `RegExp.prototype[@@split]` | See species | None directly. |
| `RegExp.prototype.compile(p, f)` | Yes | No | Wrap: same single coercion plus measurement; genuine `this` required. |
| `RegExp.prototype[@@split]` / `[@@matchAll]` | Yes, via SpeciesConstructor | Only if species resolves to a source-visible constructor; the internal default otherwise | Wrap: a genuine receiver may delegate (internal source already admitted, worst-flag bound); a generic receiver needs a spec-faithful reimplementation that constructs through the facade. |
| Species / subclass | Through the resolved constructor | Yes | Covered by the facade. |
| Cloning a genuine RegExp | Only with new flags, of admitted source | Yes | Worst-flag calibration. |

A brand check that is invisible to source (native getter or public `JS_IsRegExp`) separates genuine from generic receivers without extra observable reads. Faithfully reimplementing generic-receiver `@@split` and `@@matchAll` is the largest implementation risk in this design.

### Checkpoint density

A bounded compile per call is not enough on its own. A loop of admitted `L_max` compiles executes only a few bytecode operations per iteration, so thousands of compiles could run between engine polls. Every host admission gate must therefore also read the trusted deadline/cancellation state on entry, before calling the native compiler. The worst non-polling interval then becomes one admitted compile. This is the §5 "host checks … under its checkpoint policy" mechanism and must be tested.

## Length bound versus complexity bound

A simple UTF-16 length bound is defensible for this pin. The audited compile paths are all at most quadratic, so `L_max` gives a finite worst case without a regex complexity analyzer:

- forward named-reference `re_parse_captures` full scans (1731, 2114–2136);
- per-quantifier `dbuf_insert` memmoves and `re_need_check_adv_and_capture_init` scans (2296–2410; loop opcodes, no atom duplication);
- lookbehind term reversal in `re_parse_alternative` (2421, reversal near 2438);
- per-alternative inserts in `re_parse_disjunction` (2455).

The `v`/unicodeSets and `\p{…}` classes add large linear constants. Calibration must therefore be worst-case over families **and flags**, not only over the reproducer. Extrapolating the reproducer's quadratic scaling (about 115–143 ms at 20,007 units), `L_max` = 4096 is on the order of 5 ms per compile in this debug build. That is an estimate for one family, not a proposed value.

A complexity bound is not needed as long as the audit finds no worse-than-quadratic path. Its limitations: it rests on reading one engine revision; any engine upgrade invalidates it; and it does not prove no other superlinear path exists. The follow-up spike must measure a small fixed set of families at `L_max` and publish the policy with its pin.

## Failure classification

| Situation | Classification |
| --- | --- |
| Static literal over bound | Load resource diagnostic from host preflight; QuickJS never parses the module; instance stays healthy. |
| Dynamic construction over bound during load-validation initialization | Loading engine budget failure (load resource diagnostic). |
| Dynamic construction over bound in a ready invocation or its initialization | `RESOURCE_LIMIT`; the logical instance is disposed (§7). |

The gate first sets the trusted host latch. The latch decides classification, never exception text. The resource spike already demonstrates candidate discarding after host observation.

The rejection must also be **uncatchable**, otherwise `catch`/`finally` source would run after a resource stop. Two options for the spike:

1. **Preferred:** mark a fresh Error with public `JS_SetUncatchableError` (`quickjs.h:841`). rquickjs exposes only the query (`is_uncatchable_error`), so this needs one narrow, audited public-API unsafe call, as in the class-bridge precedent.
2. **Fallback:** a safe latch plus the handler interrupting at the next poll. This runs catch code for up to one poll interval (with service and publication checks suppressing effects) and would need explicit review against §4/§5 before acceptance.

Plain thrown errors are catchable, and a caught default OOM already showed source can mask failures.

## Match execution

Matching remains a separate requirement, but it is not this blocker. `lre_exec` polls the handler every 10,000 matcher steps (`lre_poll_timeout`, 2798), and the counter resets per call (3402). The probe adds evidence: global `replace`/`split`/`match`/`matchAll` over 200,000 short matches are all interrupted at the first armed callback (5–16 ms). This is consistent with each per-match exec entering `JS_CallInternal`'s shared poll counter (18074).

Still unproven:

- a finite worst-case interval between polls: up to ~10,000 short exec calls, each up to 10,000 steps;
- the cost of a single matcher step with long back-reference comparisons;
- native built-ins invoked with admitted patterns inside such loops.

Smallest follow-up: a focused matching checkpoint-interval audit and probe with those three cases, including host facade entry checks when present. It can run in the same spike or immediately after it.

## Candidate C: process containment

Running each engine domain in a child process, with the host killing it on its deadline, covers literals, dynamic construction, matching and unknown native paths alike. It needs no engine or RFC change, and the host knows why it killed the child, so classification is trustworthy. It also addresses allocator abort, which §10 already notes needs separate evidence.

It is not required here and costs the most:

- it replaces the in-process lifecycle evidence (Model B retirement, late delivery, scoped resolvers) with an IPC protocol for every §6 value transfer and every service call;
- it adds per-invocation process or pool lifecycle and latency;
- the charter targets iOS, iPadOS and tvOS, where hosts generally cannot spawn helper processes. It is a desktop or server option, not a portable remedy.

It remains the fallback if the admission spike fails, and a possible defense-in-depth layer. §10 "A realm need not be an OS process" confirms it is permitted, not mandated.

## Candidate D: upstream or version change

Local evidence contains only rquickjs 0.14.0 / QuickJS-NG 0.16.2: no newer registry copy, VCS checkout or changelog. No upstream fix is evidenced, and none was researched remotely, which this task forbade. Outcome C needs a *specific* justified remedy and is **not reached**.

The upstream change worth proposing is compiler timeout polling through `lre_check_timeout` in `re_parse_captures`, `dbuf_insert` loops and lookbehind reversal, with propagation of the timeout as an interrupt rather than a `SyntaxError` from the lexer literal path. A future remote review must confirm availability before any version change is considered. Even then, the literal-at-parse-time path would still need host latch classification.

## Engine patch

A local patch adding polling to the scan loops would likely close the reproduced path with little code. It also creates a maintained fork of vendored C in `rquickjs-sys`. It needs a full audit of every compiler loop (not just this family), a new error path from the literal lexer, and re-verification on every engine update. It was not attempted and is not recommended while a public-API admission design remains unrefuted.

## Comparison

| Remedy | RFC semantic change? | Private API? | Engine fork? | Covers literals? | Covers dynamic construction? | Preserves in-process runtime? | Main risk |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Pre-admission (preflight + facade) | No: host-published `L_max` under §§5/10 | No; one audited public-API unsafe call for uncatchability | No | Yes (host preflight) | Yes, if the wrap set is complete | Yes | Incomplete wrap set, or unfaithful generic `@@split`/`@@matchAll`; calibration tied to one pin |
| Process containment | No | No | No | Yes | Yes | No | IPC/lifecycle rework; unavailable on iOS/tvOS-class hosts |
| Upstream/version | No | No | No, if released upstream | Yes, if polled | Yes, if polled | Yes | No local evidence it exists; dependency upgrade not authorized |
| Engine patch | No | Internal C changes | Yes | Yes, if every loop is polled | Yes, if every loop is polled | Yes | Fork maintenance; incomplete loop audit; new lexer error path |

Evidence supports pre-admission as the smallest remedy that keeps the RFC, the public-API constraint and the in-process lifecycle. Process containment is the ranked fallback.

## Remaining uncertainty

- The wrap set is derived from one engine revision. Completeness depends on the audit and a reachability proof, not a fuzzing campaign.
- Generic-receiver `@@split`/`@@matchAll` reimplementation, genuine-argument representation, and `newTarget.prototype` order are unimplemented design obligations.
- Boa's superlinear validation, the literal parser-agreement design and the parser choice remain open. No parser is selected.
- Worst-case compile families beyond the forward-reference reproducer are identified by audit and unmeasured.
- The uncatchable path (`JS_SetUncatchableError`) is untested here.
- The matching poll interval is unbounded by evidence, although reachability of the hook is now shown.
- The review reads the conformance plan's "real checkpoint paths … such as regexps" as satisfied by bounded slices; see the central question.

## Next task

**RegExp admission feasibility implementation spike** (experiment-only, pinned APIs). It should:

- build a host-native constructor facade with a host-only native reference, preserved prototype, `instanceof`, species and subclassing, plus wrappers for `String.prototype.match`/`matchAll`/`search`, `RegExp.prototype.compile`, `@@split` and `@@matchAll`;
- apply single-coercion measurement in ES2023 order and deadline checks on gate entry;
- prove, with the existing reachability walk, that the native constructor cannot be recovered;
- route every row of the inventory above, including the 64,000-reference reproducer through each dynamic route, to a trusted-latch uncatchable `RESOURCE_LIMIT` with zero native compile and a healthy fresh realm;
- add literal preflight with exact-byte, fail-closed parser agreement before `Module::declare`;
- calibrate worst-case families and flags at a candidate `L_max`;
- run the matching interval audit.

It remains unauthorized to select an engine or parser, change the RFC or production runtime, or upgrade. If any route cannot be gated, record Outcome B for that spike and escalate to process containment review.

## Validation and audit

The new probe passes **8** tests under the 60-second process-group guard (about 1 s; guard not fired); fmt and strict Clippy pass. The 11 resource tests pass, and the blocker runner exits 2 as above. All 151 prior Phase 3 tests/checks, 53 runtime tests, 2 declarative harness tests with 34/34 cases, and 6 Python tests pass unchanged; runtime build, fmt and strict Clippy pass. Exact commands are in the [completed plan](../plans/completed/2026-09-29-quickjs-regexp-admission-review.md).

**Unsafe audit:** zero unsafe blocks/functions/impls (`forbid(unsafe_code)`), no private API, patch or dependency change. Production `runtime/`, `spec/`, `conformance/`, `examples/`, `docs/decisions/` and all prior experiments are unchanged. Nothing was pushed or published.
