use super::analysis::*;
use boa_interner::Interner;
use rquickjs::{
    Context, Module, Runtime,
    loader::{BuiltinLoader, BuiltinResolver},
};

fn quickjs_accepts(bytes: &[u8]) -> bool {
    let rt = Runtime::new().unwrap();
    rt.set_memory_limit(16 * 1024 * 1024);
    rt.set_max_stack_size(256 * 1024);
    rt.set_loader(
        BuiltinResolver::default().with_module("helper.js"),
        BuiltinLoader::default().with_module(
            "helper.js",
            "export const x = 1; export function home() {} export function impl() {}",
        ),
    );
    let context = Context::full(&rt).unwrap();
    let accepted = context.with(|ctx| {
        let compiler_input = bytes.to_vec();
        assert_eq!(compiler_input, bytes);
        Module::declare(ctx, "entry.js", compiler_input).is_ok()
    });
    assert!(!rt.is_job_pending());
    accepted
}

#[test]
fn parser_quickjs_acceptance_matrix() {
    let mut totals = [0; 4];
    for case in super::corpus::corpus() {
        let bytes: Box<[u8]> = case.text.as_bytes().into();
        let before = bytes.to_vec();
        let hash = fingerprint(&bytes);
        let mut interner = Interner::default();
        let parsed = parse(&bytes, &mut interner);
        let report = parsed
            .as_ref()
            .ok()
            .map(|ast| inspect(ast, &interner, case.id, case.operations, 1000));
        assert_eq!(&*bytes, &before);
        assert_eq!(hash, fingerprint(&bytes));
        let quickjs = quickjs_accepts(&bytes);
        totals[usize::from(parsed.is_ok()) * 2 + usize::from(quickjs)] += 1;
        println!(
            "{}: P={} Q={} gap={} violations={:?} error={:?}",
            case.id,
            parsed.is_ok(),
            quickjs,
            case.known_gap,
            report.as_ref().map(|r| &r.violations),
            parsed.as_ref().err()
        );
        assert_eq!(parsed.is_ok(), case.parser, "{} parser", case.id);
        assert_eq!(quickjs, case.quickjs, "{} QuickJS", case.id);
        if let Some(report) = report {
            assert!(report.module_items > 0);
            if let Some(category) = case.violation {
                assert!(
                    report.violations.iter().any(|d| d.category == category),
                    "{} {report:?}",
                    case.id
                );
            } else {
                assert!(report.violations.is_empty(), "{} {report:?}", case.id);
            }
        }
    }
    assert_eq!(totals, [3, 0, 1, 58]);
    println!(
        "matrix [both reject, P reject/Q accept, P accept/Q reject, both accept] = {totals:?}"
    );
}

fn analyze(text: &str, operations: Option<&[&str]>) -> Report {
    let mut interner = Interner::default();
    let ast = parse(text.as_bytes(), &mut interner).unwrap();
    inspect(&ast, &interner, "fixture.js", operations, 1000)
}

#[test]
fn all_five_direct_sync_and_async_operations_are_structurally_recognized() {
    for name in ["home", "category", "search", "detail", "play"] {
        for prefix in ["", "async "] {
            let text = format!("export {prefix}function {name}(input, context) {{}}");
            let report = analyze(&text, Some(&[name]));
            assert_eq!(report.operations, [name]);
            assert!(report.violations.is_empty());
            assert!(quickjs_accepts(text.as_bytes()));
        }
    }
}

#[test]
fn forbidden_export_forms_are_rejected_from_ast_not_function_values() {
    for case in super::corpus::corpus() {
        if let Some(category) = case.violation
            && [
                "OPERATION_DECLARATION",
                "EXPORT_LIST",
                "ENTRY_REEXPORT",
                "STAR_EXPORT",
                "DEFAULT_EXPORT",
            ]
            .contains(&category)
        {
            let report = analyze(case.text, case.operations);
            assert!(
                report.violations.iter().any(|d| d.category == category),
                "{}",
                case.id
            );
        }
    }
}

#[test]
fn escaped_names_decode_without_unicode_normalization_and_match_quickjs_names() {
    for spelling in [r"h\u006fme", r"h\u{6f}me", "hοme", "hоme", "homé", "homé"] {
        let text = format!("export function {spelling}() {{}}");
        let report = analyze(&text, Some(&["home"]));
        let expected = if spelling.starts_with("h\\") {
            "home"
        } else {
            spelling
        };
        assert_eq!(report.operations, [expected]);
        assert_eq!(report.violations.is_empty(), expected == "home");
        // A declaration-only witness of exported BindingName, not the capture strategy.
        // No operation is called; all other corpus comparisons compile without evaluation.
        let rt = Runtime::new().unwrap();
        let context = Context::full(&rt).unwrap();
        context.with(|ctx| {
            let (module, promise) = Module::declare(ctx, "names.js", text)
                .unwrap()
                .eval()
                .unwrap();
            promise.result::<()>().unwrap().unwrap();
            let names = module
                .namespace()
                .unwrap()
                .keys::<String>()
                .collect::<rquickjs::Result<Vec<_>>>()
                .unwrap();
            assert_eq!(names, [expected]);
        });
        assert!(!rt.is_job_pending());
    }
}

#[test]
fn original_bytes_survive_comments_unicode_line_endings_escapes_and_asi() {
    let source: Box<[u8]> = "/*😀*/\r\nconst café = '雪'\r\nexport\u{00a0}function h\\u006fme() {\r\nreturn\u{2028}café\u{2029}}".as_bytes().into();
    let original = source.to_vec();
    let before_parser = fingerprint(&source);
    let mut interner = Interner::default();
    let ast = parse(&source, &mut interner).unwrap();
    assert!(
        inspect(&ast, &interner, "identity.js", Some(&["home"]), 1000)
            .violations
            .is_empty()
    );
    let compiler_input = source.to_vec();
    assert_eq!(compiler_input, original);
    assert_eq!(fingerprint(&compiler_input), before_parser);
    let rt = Runtime::new().unwrap();
    let context = Context::full(&rt).unwrap();
    context.with(|ctx| {
        Module::declare(ctx, "identity.js", compiler_input).unwrap();
    });
    assert!(!rt.is_job_pending());
}

#[test]
fn nested_syntax_is_visited_independently_of_reachability() {
    for case in super::corpus::corpus() {
        if let Some(category) = case.violation
            && ["DYNAMIC_IMPORT", "IMPORT_META"].contains(&category)
        {
            let report = analyze(case.text, case.operations);
            assert!(
                report
                    .violations
                    .iter()
                    .any(|d| d.category == category && d.span.is_some()),
                "{}",
                case.id
            );
        }
    }
}

#[test]
fn static_specifiers_are_semantic_strings_not_rewritten_source() {
    for text in [
        "import './helper.js'; export function home() {}",
        "import {x} from './helper.js'; export function home() {}",
        "import * as ns from './helper.js'; export function home() {}",
        r"import {x} from '.\u002fhelper.js'; export function home() {}",
    ] {
        let report = analyze(text, Some(&["home"]));
        assert_eq!(report.imports, ["./helper.js"]);
        assert!(quickjs_accepts(text.as_bytes()));
    }
}

#[test]
fn top_level_await_is_distinct_from_function_and_arrow_await() {
    for case in super::corpus::corpus() {
        if case.id.starts_with("top-") {
            assert!(
                analyze(case.text, case.operations)
                    .violations
                    .iter()
                    .any(|d| d.category == "TOP_LEVEL_AWAIT")
            );
        }
        if case.id.starts_with("nested-") && case.id.ends_with("await") {
            assert!(analyze(case.text, case.operations).violations.is_empty());
        }
    }
    assert_eq!(
        analyze(
            "export async function home() { await 1; return async () => await 2; }",
            Some(&["home"])
        )
        .nested_awaits,
        2
    );
}

#[test]
fn dynamic_code_sites_are_observations_not_an_invented_syntax_ban() {
    let report = analyze(
        "export function home() { if (false) { eval('1'); Function(''); new Function(''); } }",
        Some(&["home"]),
    );
    assert!(report.violations.is_empty());
    assert_eq!(report.dynamic_code_sites.len(), 3);
    let shadowed = analyze(
        "export function home() { const Function = () => 1; return Function(); }",
        Some(&["home"]),
    );
    assert!(shadowed.violations.is_empty());
    // Alias/prototype constructor lockdown remains an engine/global policy problem.
}

#[test]
fn empty_attribute_clause_is_lost_in_public_ast_negative_control() {
    for (plain, attributed) in [
        ("import './helper.js';", "import './helper.js' with {};"),
        (
            "export {x} from './helper.js';",
            "export {x} from './helper.js' with {};",
        ),
    ] {
        let mut interner = Interner::default();
        let plain_ast = parse(plain.as_bytes(), &mut interner).unwrap();
        let attributed_ast = parse(attributed.as_bytes(), &mut interner).unwrap();
        assert_eq!(plain_ast.items(), attributed_ast.items());
        assert!(quickjs_accepts(attributed.as_bytes()));
        let report = inspect(&attributed_ast, &interner, "helper.js", None, 1000);
        // EXPECTED GAP: RFC forbids this source, but AST-only check misses it.
        assert!(report.violations.is_empty());
    }
}

#[test]
fn nonempty_attributes_and_old_assertions_have_distinct_rejection_paths() {
    assert!(
        analyze(
            "import './helper.js' with {type:'json'}; export function home() {}",
            Some(&["home"])
        )
        .violations
        .iter()
        .any(|d| d.category == "IMPORT_ATTRIBUTES")
    );
    let text = "import './helper.js' assert {type:'json'};";
    assert!(parse(text.as_bytes(), &mut Interner::default()).is_err());
    assert!(!quickjs_accepts(text.as_bytes()));
}

#[test]
fn diagnostics_have_host_module_category_and_ast_location() {
    let text = "export function home() {\r\n  /*😀*/ import.meta;\r\n}";
    let first = analyze(text, Some(&["home"]));
    let second = analyze(text, Some(&["home"]));
    assert_eq!(first.violations, second.violations);
    let d = &first.violations[0];
    assert_eq!(
        (d.module.as_str(), d.category),
        ("fixture.js", "IMPORT_META")
    );
    let span = d.span.unwrap();
    assert_eq!(span.start().line_number(), 2);
    assert_eq!(span.start().column_number(), 9); // Unicode code points, not UTF-8 bytes.
    assert_eq!(span.end().column_number(), 20);
    let error = parse(b"export function home(] {}", &mut Interner::default()).unwrap_err();
    match error {
        boa_parser::Error::Expected { span, .. } | boa_parser::Error::Unexpected { span, .. } => {
            assert_eq!(span.start().line_number(), 1);
            assert!(span.start().column_number() > 1);
        }
        other => panic!("expected structured syntax span, got {other:?}"),
    }
}

#[test]
fn byte_admission_and_traversal_limits_are_observable_not_heap_quotas() {
    fn admit(bytes: &[u8], limit: usize) -> Result<boa_ast::Module, &'static str> {
        if bytes.len() > limit {
            return Err("SOURCE_BYTES");
        }
        parse(bytes, &mut Interner::default()).map_err(|_| "SYNTAX")
    }
    assert_eq!(
        admit(b"export function home() {}", 8).unwrap_err(),
        "SOURCE_BYTES"
    );
    let mut interner = Interner::default();
    let text = "import './helper.js'; export function home() { return 1 + (2 * (3 + 4)); }";
    let ast = parse(text.as_bytes(), &mut interner).unwrap();
    let bounded = inspect(&ast, &interner, "bounds.js", Some(&["home"]), 2);
    assert!(
        bounded
            .violations
            .iter()
            .any(|d| d.category == "EXPRESSION_LIMIT")
    );
    let report = inspect(&ast, &interner, "bounds.js", Some(&["home"]), 100);
    assert_eq!(report.module_items, 2);
    assert_eq!(report.imports.len(), 1);
    assert!(report.expression_visits > 2);
    assert!(report.max_expression_depth >= 3);
    // Traversal limits run after allocation/parsing; no parser stack/heap claim.
}

#[test]
fn identifier_span_preserves_raw_escape_location_and_semantic_name() {
    use boa_ast::{
        ModuleItem, Spanned,
        declaration::{Declaration, ExportDeclaration},
    };
    let text = r"export function h\u006fme() {}";
    let mut interner = Interner::default();
    let ast = parse(text.as_bytes(), &mut interner).unwrap();
    let ModuleItem::ExportDeclaration(export) = &ast.items().items()[0] else {
        panic!("export")
    };
    let ExportDeclaration::Declaration(Declaration::FunctionDeclaration(function)) =
        export.as_ref()
    else {
        panic!("direct function")
    };
    assert_eq!(symbol(&interner, function.name().sym()), "home");
    let span = function.name().span();
    let raw = &text
        [(span.start().column_number() - 1) as usize..(span.end().column_number() - 1) as usize];
    assert_eq!(raw, r"h\u006fme"); // ASCII-only source: code-point columns equal byte offsets here.
}

#[test]
fn helper_export_rules_do_not_inherit_entry_only_restrictions() {
    for text in [
        "export function* helper() {}",
        "const x = 1; export {x};",
        "export {x} from './helper.js';",
    ] {
        assert!(analyze(text, None).violations.is_empty());
    }
    for (text, category) in [
        ("export default function() {}", "DEFAULT_EXPORT"),
        ("export * from './helper.js';", "STAR_EXPORT"),
    ] {
        assert!(
            analyze(text, None)
                .violations
                .iter()
                .any(|d| d.category == category)
        );
    }
}
