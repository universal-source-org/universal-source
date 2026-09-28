# JavaScript source-static-analysis compatibility review

## Scope and conclusion

**Outcome B — static preflight blocker.** The tested Boa Parser 0.22.0 AST strategy distinguishes the required operation export forms and the tested nested syntax, but cannot enforce RFC 0001's import-attribute prohibition by AST inspection alone: absent attributes and an explicitly empty `with {}` clause produce equal module-item ASTs. Pinned QuickJS accepts both. The gap needs extra syntax-aware source validation or a representation that preserves clause presence. It does not justify weakening the RFC or rejecting QuickJS itself.

Baseline: `9307dc4be4e3c9bfa27ae9127a011b6abe499279`, `test(engine): prove QuickJS pre-evaluation module capture`; clean `main` and no active plan at start. Phase 3 remains active. [RFC 0001](../../spec/rfcs/0001-javascript-execution-binding.md) remains proposed and byte-for-byte unchanged. Model B, the [lifecycle evidence](2026-09-28-quickjs-lifecycle-spike.md), and the [pre-evaluation capture strategy](2026-09-28-quickjs-module-capture-spike.md) are unchanged. This is neither parser nor engine selection, an ADR, production execution, or JavaScript conformance.

## Candidate, pins and source evidence

One candidate was examined: the standalone Boa parser. Its public AST has separate module/export/declaration variants, semantic identifier interning, source locations and a recursive visitor. It parses modules without evaluating or regenerating them and is Rust-accessible without the Boa VM. Those properties directly address the namespace-reflection gap; existing Boa engine experiments are not a parser-selection decision. The maintained [0.22 release](https://github.com/boa-dev/boa/releases/tag/v0.22) and versioned public APIs make it a reasonable feasibility candidate, not proof of full ES2023 compatibility or long-term API stability.

| Component | Exact version | Source revision / evidence |
| --- | --- | --- |
| `boa_parser`, `boa_ast`, `boa_interner` | `=0.22.0` each, default features off | `337a3668a0dc86dd401ea20906e782249a64a228`; published `.cargo_vcs_info.json` for all three. Unlicense OR MIT; upstream manifests require Rust 1.91. |
| rquickjs/core/sys | `0.14.0`; direct rquickjs `=0.14.0`, `std`/`loader` only | `d7ef5eeae702fea24c03643064de454f1c1dd4b0`; published VCS metadata, unchanged from capture spike. |
| QuickJS-NG | `0.16.2` | `0fdea21ff1090084e91dad812b343d92e79ba9d9`; same vendored engine as prior spike. |
| Rust / Cargo | `1.96.0` / `1.96.0` | rustc `ac68faa20c58cbccd01ee7208bf3b6e93a7d7f96`, Cargo `30a34c682`, LLVM 22.1.2. |
| Local machine | macOS `27.0`, build `26A428` | `aarch64-apple-darwin` / arm64 only. No other-platform claim. |

The new isolated lockfile contains 71 external packages, all exact package/version pairs already in previous experiment lockfiles; it includes no `boa_engine`. Four direct dependencies are pinned in the [manifest](../../experiments/javascript-static-analysis-spike/Cargo.toml). Production dependencies and prior lockfiles are unchanged. No engine upgrade was necessary.

Primary API/source evidence, inspected at the pinned revisions:

- [Parser module API](https://github.com/boa-dev/boa/blob/337a3668a0dc86dd401ea20906e782249a64a228/core/parser/src/parser/mod.rs): `Parser::new(Source::from_bytes(...)).parse_module(&Scope::new_global(), &mut Interner)`. Module parsing includes scope analysis. `parse_module_with_source` additionally returns source text, not a lossless module-clause AST.
- [Export AST](https://github.com/boa-dev/boa/blob/337a3668a0dc86dd401ea20906e782249a64a228/core/ast/src/declaration/export.rs) and [import AST](https://github.com/boa-dev/boa/blob/337a3668a0dc86dd401ea20906e782249a64a228/core/ast/src/declaration/import.rs): `ModuleItem`, `ExportDeclaration::{Declaration,List,ReExport,...}`, separate declaration kinds, `ReExportKind`, `specifier()` and `attributes()`. Attribute slices lack an absent-versus-present-empty discriminator.
- [WithClause parser](https://github.com/boa-dev/boa/blob/337a3668a0dc86dd401ea20906e782249a64a228/core/parser/src/parser/statement/declaration/mod.rs): missing `with` returns an empty boxed slice; parsing `with {}` also returns an empty boxed slice. This explains the executed negative control without relying on private internals at runtime.
- [Visitor](https://github.com/boa-dev/boa/blob/337a3668a0dc86dd401ea20906e782249a64a228/core/ast/src/visitor.rs), [positions](https://github.com/boa-dev/boa/blob/337a3668a0dc86dd401ea20906e782249a64a228/core/ast/src/position.rs), [lexer cursor](https://github.com/boa-dev/boa/blob/337a3668a0dc86dd401ea20906e782249a64a228/core/parser/src/lexer/cursor.rs), and [interner](https://github.com/boa-dev/boa/blob/337a3668a0dc86dd401ea20906e782249a64a228/core/interner/src/lib.rs): `Visitor`/`VisitWith`, expression-specific visits, `Spanned`, line/column positions, decoded UTF-16 symbols. These eight Boa source files were fetched from the pinned upstream revision and byte-compared with the installed registry sources; all matched.
- [rquickjs module API](https://github.com/DelSkayn/rquickjs/blob/d7ef5eeae702fea24c03643064de454f1c1dd4b0/core/src/value/module.rs): `Module::declare` compiles original bytes with module/compile-only flags. The [QuickJS header](https://github.com/quickjs-ng/quickjs/blob/0fdea21ff1090084e91dad812b343d92e79ba9d9/quickjs.h) and [implementation](https://github.com/quickjs-ng/quickjs/blob/0fdea21ff1090084e91dad812b343d92e79ba9d9/quickjs.c) are the same public compiler boundary established by the previous capture review. No engine-private data or patched parser is used.

## Experiment and source identity

The [standalone crate](../../experiments/javascript-static-analysis-spike/README.md) contains a fixture-only AST inspector, deterministic corpus and tests. Each source is held as immutable bytes. Boa reads those bytes; AST inspection decodes symbols only for comparisons. The same original content is copied into `Module::declare` without formatting, newline normalization, transpilation or regeneration. FNV-1a checksums before parsing and at compilation plus full byte equality are asserted; the checksum is an evidence aid, not a security primitive. A dedicated fixture combines comments, astral/Chinese text, CRLF, nonbreaking whitespace, escaped identifiers, U+2028/U+2029 and automatic semicolon insertion.

The compiler probe supplies a fixed in-memory `helper.js` through `BuiltinResolver`/`BuiltinLoader`; it has no filesystem or network integration. It independently compiles even statically rejected fixtures to measure syntax agreement. This is not the future execution gate: a real pipeline must stop before evaluation on any preflight failure. The matrix loader ignores attributes to observe compiler syntax acceptance, not to permit JSON/native modules or their semantics. All compile probes assert no pending jobs.

Only the escaped/Unicode name witness evaluates declaration-only modules to inspect actual namespace names. No operation is called. That witness does not replace pre-evaluation capture with post-evaluation discovery; the prior capture experiment is unchanged. No combined production snapshot/parse/capture pipeline was built.

## RFC requirement to AST evidence

All 15 [tests](../../experiments/javascript-static-analysis-spike/src/tests.rs) pass; passing negative controls demonstrate limitations, not profile compliance.

| RFC rule / question | Public representation and executable result |
| --- | --- |
| §2 direct sync/async operations, exact name set | `ExportDeclaration::Declaration` plus `FunctionDeclaration` / `AsyncFunctionDeclaration` and semantic names. Both forms pass for each of home/category/search/detail/play; missing/extra names are rejected. No arity/parameter-name restriction was invented. |
| Generator / async generator | Distinct declaration kinds, rejected as operation declarations even though namespace values could be callable. |
| Same-name export list / alias | `ExportDeclaration::List`, rejected for entry, even when local and exported names match. |
| Entry re-export / alias re-export | `ReExport`, rejected for entry; named helper re-exports remain allowed. |
| Star / namespace star, default named/anonymous/async | Distinct re-export/default variants; rejected throughout the graph. A default function's local name cannot count as home. |
| Exported const arrow, let function, class, number, var function | Lexical/class/variable variants, rejected structurally; runtime callability is irrelevant. |
| Escaped and Unicode names | Semantic symbol comparison plus identifier spans; detailed results below. |
| §1.6 dynamic import / import.meta anywhere | `ImportCall` / `ImportMeta` visitor reaches operation/helper/function/arrow/class code and unreachable branches, including computed class keys; every tested case is flagged with a span. |
| §1.6 top-level await | `Await` and `ForOfLoop::await` with function-body nesting distinguish top-level/block/computed-key await from allowed async function/arrow/for-await bodies. No source is executed to decide reachability. |
| §1.3 static graph edges | Side-effect, named, namespace imports and named re-exports expose decoded specifiers, including escaped string spelling; fixed fixtures yield exact semantic `./helper.js`. No containment/DAG resolver is implemented. |
| §1.6 import attributes/assertions | Nonempty `with {type:'json'}` exposed and rejected. Legacy `assert` syntax rejected by both parsers. **Empty `with {}` is lost: blocker.** |
| §1.2 ES2023 baseline | Tested ES2023 parameter/class/private-field/regexp/optional-chain/nullish syntax agrees. Both parsers also accept newer `using`; the explicit declaration variant is rejected in the probe. No complete ES2023 edition gate is claimed. |
| §9 dynamic code compilation | Direct eval/Function call AST sites can be observed even in unreachable code. They are not syntax errors under this RFC: attempted compilation must throw runtime EvalError. Shadowed Function identifiers and constructor aliases show why this observation cannot establish lockdown. Runtime hardening is outside this task. |
| §1.5 duplicate/invalid syntax | Duplicate export and malformed source fail parsing/compilation, before evaluation. No engine-specific text is made portable. |

Helper mode is distinct from entry mode: named lists/re-exports and generator helpers do not become invalid merely because entry operations require direct nongenerator functions. Defaults and stars remain forbidden throughout the graph. The inspector is intentionally incomplete outside these questions.

## Names, diagnostics and bounds

`h\u006fme` and `h\u{6f}me` decode to semantic `home` in Boa; QuickJS exposes the same exported name. RFC §1.2 uses ECMAScript 2023 and §2 compares export names with contract operation names. Under those existing language semantics, an escape spelling the same IdentifierName is the same operation name, not a new alias. The RFC does not specify a raw-ASCII-spelling restriction. If one were desired, it would require an explicit normative decision; this experiment adds none. The raw escape location remains available through the identifier's span and retained original text.

Greek omicron (`hοme`), Cyrillic o (`hоme`), `homé` and `homé` remain distinct and fail the home name-set check. Composed and decomposed helper identifiers can coexist. Neither stage normalizes Unicode or adds visually similar operation aliases. Static import strings similarly expose their language-decoded value; the original escaped source bytes still reach QuickJS.

The diagnostic probe supplies a host module identifier, stable internal category such as `IMPORT_META`, and the AST span. Repeated parsing gives identical diagnostic records. A CRLF/astral-character example locates `import.meta` at line 2, columns 9–20; columns count Unicode code points, not UTF-8 byte offsets. Structured parser `Expected`/`Unexpected` errors also expose spans. Not every export node or scope-analysis error supplies a precise span, so some probe diagnostics are module-level. No exact message text is standardized. The raw-identifier-slice test uses ASCII source specifically so columns equal byte offsets; a general source map must not make that assumption.

Byte admission can happen before parsing. Module item/import counts and expression-visit/depth counters are observable after parsing; the visitor can abort on an expression-count limit. These are structural feasibility observations, not parser heap/stack quotas or a complete AST-node counter. Parsing and scope analysis recurse before that visitor runs. No public hard memory/time/depth limit was established for Boa parsing, and an external test timeout is not a production sandbox. The small fixture symbol helper fails explicitly on unpaired UTF-16 surrogates rather than replacing them; complete string/specifier error handling remains outside the probe.

## Parser versus QuickJS matrix

[corpus.rs](../../experiments/javascript-static-analysis-spike/src/corpus.rs) records all 62 sources, stable IDs, syntax expectations, RFC categories and the two known gaps. The test prints every result and asserts the totals; it does not equate parser acceptance with RFC acceptance.

| Boa syntax | QuickJS compile | Count | Interpretation |
| --- | --- | --- | --- |
| Accept | Accept | 58 | Tested valid forms, many invalid RFC forms caught structurally, and the two explicitly missed empty-attribute cases. Agreement alone is insufficient. |
| Accept | Reject | 1 | `import-source-phase`: `import.source()` is parsed as an import call, rejected by the static dynamic-import rule and by QuickJS. It is outside ES2023/RFC, so this disagreement does not reject a valid RFC source. |
| Reject | Accept | 0 | No such disagreement in this bounded corpus; not a whole-language compatibility claim. |
| Reject | Reject | 3 | `duplicate`, `malformed`, and legacy `assertion`. |

All tested RFC-valid ES2023 sources are accepted by both. The blocker is not a syntax-acceptance disagreement: it is information lost between accepted syntax and static policy inspection. Nonempty attributes, newer `using`, generators, lists, aliases, re-exports, defaults and TLA illustrate the separate “JavaScript syntax accepted / RFC rejected” stage. Broader grammar/version compatibility is unproven.

## Concrete blocker and next unit

`empty_attribute_clause_is_lost_in_public_ast_negative_control` parses each pair below using the same interner and asserts equal `ModuleItemList` values:

```js
import './helper.js';
// Versus a separate source:
import './helper.js' with {};
```

```js
export {x} from './helper.js';
// Versus a separate helper source:
export {x} from './helper.js' with {};
```

QuickJS compilation accepts the attributed versions; the AST-only inspector reports no violation. These are explicit false acceptances under RFC §1.6. `WithClause` returns an empty attribute list for both cases, and the import/re-export AST stores that list without a clause-presence bit or enclosing declaration span. Checking `attributes().is_empty()` cannot distinguish them. This is a public representation limitation proven by executable evidence and pinned implementation source, not a hypothetical concern.

**Classification: extra source validation or a different/lossless parser representation is required for this candidate strategy.** Original bytes remain available, and Boa also exposes source-returning parsing and lexical APIs; the result does not prove that every possible Boa-based solution is impossible. However, merely retaining text or receiving tokens does not implement reliable clause identification across JavaScript lexical goals. No supported lossless-clause procedure is demonstrated here, so Outcome A would be unjustified. No additional author restriction, RFC grammar narrowing, private engine access, dependency upgrade or architecture rewrite is proposed.

The smallest follow-up is one bounded experiment proving attribute-clause presence detection on original text through public syntax-aware APIs (or a lossless parser candidate), including empty import and named-re-export clauses, comments, strings, templates and regex lexical contexts as negative controls. It must preserve valid static imports and exact bytes; substring/regex rejection of `with` is not sufficient. This resolves the observed static-preflight blocker before broader parser selection. It does not reopen the demonstrated lifecycle/capture mechanisms or authorize production implementation.

Other engine-selection gaps—global hardening, constructor suppression, complete non-executing conversion, resource policy, module/path containment, platform support, conformance and service profiles—remain separate. Exact validation commands, file inventory and preservation checks are in the [completed plan](../plans/completed/2026-09-28-javascript-static-analysis-spike.md). No production behavior/dependency, RFC status, specification version, ADR or existing experiment changed; nothing was pushed, tagged or released.
