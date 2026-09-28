# Import-attribute clause presence feasibility

## Conclusion and scope

**Outcome A — attribute-clause preflight viable.** A supplemental Oxc 0.152.0 public syntax-tree check preserves whether an import/re-export attribute clause exists, including empty `with {}`. Combined with the unchanged Boa structural probe over identical original bytes, it closes both false-acceptance cases demonstrated by the [previous static-analysis review](2026-09-28-javascript-static-analysis-spike.md). Strings, comments, templates, regex literals, identifiers/properties and tested ASI boundaries do not produce false detections.

Baseline: `2fea0b2bf678b775964ddc0c861a8a9a5aadc824`, `test(parser): expose empty import-attribute preflight gap`; clean main, no active plan at start. Phase 3 remains active. This resolves one representation blocker, not complete preflight or parser compatibility. No parser or engine is selected, no ADR created, no RFC amendment/acceptance, and no production behavior/dependency changed. Model B and the [pre-evaluation capture strategy](2026-09-28-quickjs-module-capture-spike.md) remain unchanged.

## Public API investigation and pinned evidence

| Component | Version / revision | Role |
| --- | --- | --- |
| Boa Parser/AST/Interner | `0.22.0`; `337a3668a0dc86dd401ea20906e782249a64a228` | Unchanged parser and structural validator; public lexer examined. |
| Oxc parser/AST/allocator/span | Each pinned `=0.152.0`; `dfbc0d1ea752f021ba4a68e4703b3a9031082a46` | Sole alternate candidate, supplemental clause-preserving syntax representation. Published VCS metadata agrees for all four. MIT; requires Rust 1.96.0. |
| rquickjs/core/sys | `0.14.0`; `d7ef5eeae702fea24c03643064de454f1c1dd4b0` | Unchanged safe public compile-only module path. |
| QuickJS-NG | `0.16.2`; `0fdea21ff1090084e91dad812b343d92e79ba9d9` | Unchanged vendored compiler. |
| Local evidence | Rust/Cargo `1.96.0`, macOS `27.0` build `26A428`, arm64 | rustc `ac68faa20c58cbccd01ee7208bf3b6e93a7d7f96`, target `aarch64-apple-darwin`; no cross-platform claim. |

Boa's [public lexer source](https://github.com/boa-dev/boa/blob/337a3668a0dc86dd401ea20906e782249a64a228/core/parser/src/lexer/mod.rs) exposes `Lexer::from`, `new` and `next`. It skips comments and defaults to the RegExp lexical goal. `InputElement`, `set_goal`, slash re-lexing and template continuation are crate-private. The executable negative control parses `const ratio = 6 / 2;` successfully with Boa and QuickJS, but repeatedly calling the public lexer fails because it treats division as a regex start. A standalone scan would need parser context and unavailable controls, not merely a search for keyword tokens. This rejects that simple strategy; it does not prove every imaginable Boa-only source-analysis technique impossible.

Boa's [source-retaining parser API](https://github.com/boa-dev/boa/blob/337a3668a0dc86dd401ea20906e782249a64a228/core/parser/src/parser/mod.rs) can return source text but the same module AST. It does not expose a clause-presence callback or lossless token stream from the completed parse. The original bytes were already retained. No private parser cursor was accessed and no manual JavaScript grammar was added.

One alternate was sufficient: Oxc's public AST is **lossless for the distinction needed here**, not a claim that its entire AST is a lossless CST. It represents absent clauses as `None` and present clauses as `Some(WithClause)`, independently of entry count. Its parser handles lexical goals and owns each clause under the correct module declaration. No formatter, serializer, transpiler or Oxc engine is used.

| Pinned primary source | Public evidence used |
| --- | --- |
| [Oxc parser API](https://github.com/oxc-project/oxc/blob/dfbc0d1ea752f021ba4a68e4703b3a9031082a46/crates/oxc_parser/src/lib.rs) | `Parser::new`, `SourceType::mjs`, `ParseOptions`, `ParserReturn::{program,diagnostics,fatal_error}`. Errors may coexist with a recovered tree; all parser diagnostics and fatal errors reject the supplemental check. Regex parsing is explicitly enabled. |
| [Oxc module AST](https://github.com/oxc-project/oxc/blob/dfbc0d1ea752f021ba4a68e4703b3a9031082a46/crates/oxc_ast/src/ast/js.rs) | `ImportDeclaration`, `ExportFromDeclaration`, `ExportAllDeclaration` each store optional `with_clause`; `WithClause` has `span`, `keyword` and `with_entries`. Empty entries do not erase the clause. |
| [Public AST helpers](https://github.com/oxc-project/oxc/blob/dfbc0d1ea752f021ba4a68e4703b3a9031082a46/crates/oxc_ast/src/ast_impl/js.rs) | `Statement::as_module_declaration`, `ModuleDeclaration::source` and `with_clause`; ordinary expressions, property names, strings and templates are not module declarations. |
| [Oxc module parser](https://github.com/oxc-project/oxc/blob/dfbc0d1ea752f021ba4a68e4703b3a9031082a46/crates/oxc_parser/src/js/module.rs) | Read-only explanation: `parse_import_attributes` returns `None` without the keyword, otherwise constructs `Some` even for zero entries. Both `with` and legacy `assert` are represented. This private implementation is read, never called directly. |
| [Span API](https://github.com/oxc-project/oxc/blob/dfbc0d1ea752f021ba4a68e4703b3a9031082a46/crates/oxc_span/src/span.rs) | `GetSpan`, byte-indexed start/end ranges into the original UTF-8 text. The executed multibyte/CRLF witness verifies the mapping. |

All five pinned Oxc files and the two Boa files above were fetched from upstream and byte-compared to installed registry source: seven matches. The existing [QuickJS module API evidence](2026-09-28-quickjs-module-capture-spike.md#versions-public-apis-and-sources) still applies. No private structs, pointer/layout inspection, unsafe blocks, patched dependency or modified engine is introduced.

## Narrow hybrid procedure

[attributes.rs](../../experiments/javascript-static-analysis-spike/src/attributes.rs) implements only this experimental procedure:

1. Parse the immutable bytes through the existing Boa module parser. Any parse/scope error stops the gate.
2. Pass the same bytes, viewed as UTF-8 without conversion/normalization, to Oxc in JavaScript module mode (not TypeScript/JSX). Reject any Oxc diagnostic or fatal parse result; do not treat a recovered tree as success.
3. Enumerate top-level module declarations in Oxc. For every source-bearing import or re-export, record its kind, decoded specifier, owning declaration span, optional clause span and entry count.
4. Compare the ordered import/re-export kind and decoded-specifier sequence against Boa's AST. Count/kind/specifier disagreement rejects the gate. This is a narrow agreement witness, not equivalence of all AST semantics.
5. Reject **any** `Some(WithClause)` regardless of whether its entries are empty. Otherwise apply the existing Boa structural inspection. That old inspector and its negative controls are unchanged.

Static import/re-export declarations belong at module top level; the parser determines where they occur. We do not search arbitrary source text for `with`, strip comments, or manually decide regex/template grammar. AST parentage supplies clause ownership, so a later string/property named `with` cannot attach itself to an import. A real clause after a line break remains part of its import when the grammar permits it.

Oxc's `WithClause.span` begins at `{` and ends after `}`; it **excludes the keyword and preceding trivia**. Its presence and keyword are typed AST data, not inferred by scanning that span. The tested source slice is exactly `{/*inner*/}` inside an enclosing named-re-export span, with Unicode comments and CRLF earlier in the file. Comments inside the braces remain in the original slice; preceding comments and intervening keyword-to-brace trivia do not obscure ownership. The declaration span covers the whole statement. No promise of exact portable diagnostic wording is made.

## Executable evidence

The extended crate has **24 passing tests**: the original 15 remain unchanged, plus nine [attribute tests](../../experiments/javascript-static-analysis-spike/src/attribute_tests.rs).

| Test | Evidence |
| --- | --- |
| `all_import_and_reexport_forms_preserve_empty_and_nonempty_clause_presence` | 24-row matrix: eight declaration forms × absent/empty/nonempty clauses. Side-effect/named/default/namespace imports, named/aliased/star/namespace-star re-exports all preserve clause presence. All 24 parse in Boa/Oxc and compile in pinned QuickJS. All 16 attributed forms are rejected. Plain star forms still fail the existing RFC star rule. |
| `prior_ast_only_false_acceptance_is_closed_without_changing_old_validator` | The original import and helper-re-export with empty attributes still evade Boa-only inspection; the hybrid rejects both. A plain import plus directly exported home passes the scoped check. |
| `lexical_negative_controls_and_asi_do_not_create_clauses` | 15 controls after an import without a semicolon: double/single strings, line/block comments, ordinary/interpolated/nested templates, regex including import-like punctuation, division, identifier/property/method/destructuring/label forms and ASI. No clause found; all compile. |
| `comments_and_line_breaks_inside_and_around_real_clauses_are_preserved` | Eight forms × five placements = 40 attributed sources: newline/block comment, inter-token comments, CRLF/line comments, Unicode whitespace and inner comments/trailing comma. All compile; all are detected/rejected. |
| `declaration_ownership_and_utf8_byte_spans_point_into_exact_source` | Three declarations with only the middle re-export attributed; exact brace slice and containment in its owner verified, including Unicode/CRLF byte mapping. |
| `syntax_errors_and_parser_disagreement_fail_closed` | Malformed/invalid UTF-8/recovered-error inputs reject. Injected missing/kind/specifier summary disagreements reject. Real legacy-assertion disagreement rejects through Boa; duplicate export is rejected by both parsers and QuickJS. |
| `naive_text_checks_are_not_a_syntax_strategy` | `contains("with")` and literal `with {` checks falsely flag strings/templates/comments/identifiers, while `with/*comment*/{}` defeats the latter pattern. Comment-looking bytes inside valid strings/regex are not removable trivia. No heuristic is used in the detector. |
| `boa_public_lexer_alone_cannot_follow_parser_lexical_goals` | Public default token loop fails on valid division; both full parsers and QuickJS accept it. |
| `previous_corpus_has_no_silent_supplemental_acceptance` | Reruns all 62 prior sources through the supplemental representation. All previously valid controls remain accepted; prior invalid forms remain rejected; both known gaps now reject. |

Default-import and star fixtures measure grammar/representation only: the fixed compiler helper exports a default for that diagnostic control, but the RFC still bans default exports throughout package graphs and bans stars. The probe does not claim those fixture graphs are admissible packages. It neither loads arbitrary files nor executes module bodies or jobs.

## Exact bytes and parser/compiler agreement

The 24-row matrix snapshots original bytes and FNV-1a checksum before parsing, checks them afterward and supplies the unchanged content to `Module::declare`. The dedicated ownership witness repeats this with multibyte Unicode, CRLF and comments. Full byte equality accompanies the checksum; FNV is not a security boundary. Neither AST is serialized back to JavaScript. There is no newline normalization or regenerated source.

QuickJS compilation is an independent diagnostic comparison even for statically rejected sources; it is never permission to evaluate them. The fixed in-memory loader ignores attribute semantics for this grammar probe. Every compile path asserts no pending jobs. The production sequencing under investigation remains preflight rejection before any package evaluation.

| Corpus / control | Boa | Oxc parser | QuickJS | Scoped gate |
| --- | --- | --- | --- | --- |
| 8 forms, plain | Accept | Accept | Accept | No attribute violation; stars rejected separately. |
| 8 forms, empty attributes | Accept | Accept, clause present | Accept | Attributes rejected. |
| 8 forms, nonempty attributes | Accept | Accept, clause present | Accept | Attributes rejected. |
| 40 comment/whitespace variations | Accept | Accept, clause present | Accept | Attributes rejected. |
| 15 lexical/ASI negative controls | Accept | Accept, no clause | Accept | Accepted by scoped check. |
| Legacy `assert {}` and prior nonempty assertion | Reject | Accept, clause present | Reject | Boa syntax failure; never accepted. |
| Duplicate exports / malformed source | Reject | Reject | Reject | Syntax failure. |
| Prior `import.source()` control | Accept | Accept | Reject | Existing dynamic-import prohibition rejects. |

There is no acceptance disagreement on tested RFC-valid sources. Oxc accepting legacy assertion syntax is an observed extra grammar surface, not a reason to widen the RFC. Oxc parser diagnostics are not comprehensive early-error checking: its public docs require semantic analysis for standalone full validation. Here Boa parsing/scope checks remain in the gate, and Oxc only supplies clause data. No claim that this combination proves every early error or whole-language agreement is made. Parser errors and observed summary disagreements fail closed; resolving a future disagreement on valid profile source would still require evidence, not silent grammar narrowing.

## Limits, dependencies and next unit

Four exact Oxc direct dependencies are added only to this unpublished experiment. The lockfile adds 32 packages (103 external total), with no old package/version removed or upgraded. Some shared dependency feature edges expand; this is contained in the experiment. No `boa_engine`, production dependency or additional parser candidate is added. The cost and long-term suitability of two parsers are not decided here.

This is a finite compatibility proof for clause presence, not a full lossless syntax tree, complete ES2023 gate, production validator, security sandbox, resource accounting or platform proof. Parsing/AST traversal memory and recursion bounds remain unproven. The earlier fixture-string limitations remain. No new restrictions on source authors, normalization, patch, private API or unsupported lexical assumption is needed for this result.

The **previous empty-attribute false-acceptance path is closed in the demonstrated hybrid strategy**. Boa AST alone remains insufficient; its preserved negative control remains useful evidence. No clause-presence blocker remains within the tested scope.

Next coherent task: a focused pinned QuickJS-NG/rquickjs **dynamic JavaScript compilation suppression proof** for RFC §9, covering direct/indirect eval, function-family constructor aliases/prototype paths, Reflect construction and mutation resistance with required EvalError behavior. Do not turn that into full sandbox implementation, engine selection or production execution. Complete grammar/resource/global-surface/value-conversion/platform/service evidence remains separate.

The [completed plan](../plans/completed/2026-09-28-javascript-import-attributes-spike.md) records exact validation and changed files. RFC 0001 remains proposed and untouched. No parser/engine selection, ADR, push, tag or release follows.
