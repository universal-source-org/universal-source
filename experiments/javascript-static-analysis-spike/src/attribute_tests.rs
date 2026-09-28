use super::{analysis::*, attributes::*};
use boa_interner::Interner;
use rquickjs::{
    Context, Module, Runtime,
    loader::{BuiltinLoader, BuiltinResolver},
};

fn compile_original(bytes: &[u8]) -> bool {
    let rt = Runtime::new().unwrap();
    rt.set_memory_limit(16 * 1024 * 1024);
    rt.set_max_stack_size(256 * 1024);
    rt.set_loader(
        BuiltinResolver::default().with_module("helper.js"),
        BuiltinLoader::default().with_module("helper.js", "export const x = 1; export default x;"),
    );
    let context = Context::full(&rt).unwrap();
    let result = context.with(|ctx| {
        let compiler_input = bytes.to_vec();
        assert_eq!(compiler_input, bytes);
        assert_eq!(fingerprint(&compiler_input), fingerprint(bytes));
        Module::declare(ctx, "fixture.js", compiler_input).is_ok()
    });
    assert!(!rt.is_job_pending());
    result
}

fn prefixes() -> [&'static str; 8] {
    [
        "import './helper.js'",
        "import { x } from './helper.js'",
        "import x from './helper.js'",
        "import * as ns from './helper.js'",
        "export { x } from './helper.js'",
        "export { x as y } from './helper.js'",
        "export * from './helper.js'",
        "export * as ns from './helper.js'",
    ]
}

#[test]
fn all_import_and_reexport_forms_preserve_empty_and_nonempty_clause_presence() {
    let mut rows = 0;
    for prefix in prefixes() {
        for (suffix, present, entries) in [
            (";", false, 0),
            (" with {};", true, 0),
            (" with { type: 'json' };", true, 1),
        ] {
            let source: Box<[u8]> = format!("{prefix}{suffix}").into_bytes().into();
            let original = source.to_vec();
            let hash = fingerprint(&source);
            assert!(parse(&source, &mut Interner::default()).is_ok());
            let edges = oxc_edges(&source).unwrap();
            assert_eq!(edges.len(), 1);
            assert_eq!(edges[0].clause.is_some(), present);
            assert_eq!(edges[0].entries, entries);
            let result = preflight(&source, None);
            assert_eq!(result, preflight(&source, None));
            if present {
                assert_eq!(result, Err(Failure::Attributes));
            } else if prefix.starts_with("export *") {
                assert_eq!(result, Err(Failure::OtherProfile)); // Stars remain forbidden.
            } else {
                assert_eq!(result, Ok(()));
            }
            assert_eq!(&*source, original);
            assert_eq!(fingerprint(&source), hash);
            assert!(compile_original(&source));
            println!(
                "attribute-matrix: {} Boa=true Oxc=true QuickJS=true clause={present} preflight={result:?}",
                String::from_utf8_lossy(&source)
            );
            rows += 1;
        }
    }
    assert_eq!(rows, 24);
}

#[test]
fn prior_ast_only_false_acceptance_is_closed_without_changing_old_validator() {
    for source in [
        "import './helper.js' with {}; export function home() {}",
        "export {x} from './helper.js' with {};",
    ] {
        let operations = if source.starts_with("import") {
            Some(&["home"][..])
        } else {
            None
        };
        let mut interner = Interner::default();
        let ast = parse(source.as_bytes(), &mut interner).unwrap();
        assert!(
            inspect(&ast, &interner, "fixture.js", operations, 1000)
                .violations
                .is_empty()
        );
        assert_eq!(
            preflight(source.as_bytes(), operations),
            Err(Failure::Attributes)
        );
        assert!(compile_original(source.as_bytes()));
    }
    assert_eq!(
        preflight(
            b"import './helper.js'; export function home() {}",
            Some(&["home"])
        ),
        Ok(())
    );
}

#[test]
fn lexical_negative_controls_and_asi_do_not_create_clauses() {
    for tail in [
        r#"const a = "with {}";"#,
        r#"const b = 'with { type: "json" }';"#,
        "// with {}\nconst n = 1;",
        "/* with {} */ const n = 1;",
        "const t = `with {}`;",
        "const t = `${\"with\"} {}`;",
        "const t = `${`with ${'{}'}`} import './helper.js' with {}`;",
        r"const r = /with \{\}/;",
        r#"const r = /import ['"]x['"] with \{\}/;"#,
        r"const r = /[/]with\{\}[/]/; const q = 6 / 2 / 3;",
        "const withValue = {}; const obj = {with: 1}; obj.with;",
        "withValue: { const n = 1; }",
        "const {with: value} = {with: 1};",
        "const obj = { with() { return {}; } }; obj.with();",
        "const text = 'with {}'\nconst more = 2",
    ] {
        // Missing semicolon after import deliberately exercises ASI with each next item.
        let source = format!("import './helper.js' /* with {{}} */\n{tail}");
        assert_eq!(preflight(source.as_bytes(), None), Ok(()), "{source}");
        assert!(
            oxc_edges(source.as_bytes())
                .unwrap()
                .iter()
                .all(|e| e.clause.is_none())
        );
        assert!(compile_original(source.as_bytes()), "{source}");
    }
}

#[test]
fn comments_and_line_breaks_inside_and_around_real_clauses_are_preserved() {
    for prefix in prefixes() {
        for suffix in [
            "\n/* comment with {} */\nwith {};",
            " /* before */ with /* gap */ { /* inside */ };",
            "\r\n// before\r\nwith\r\n{\r\n// inside\r\n};",
            "\u{2028}with\u{00a0}{/*😀*/};",
            " with { /* key */ type /* colon */ : /* value */ 'json', };",
        ] {
            let source = format!("{prefix}{suffix}");
            assert_eq!(
                preflight(source.as_bytes(), None),
                Err(Failure::Attributes),
                "{source}"
            );
            assert!(compile_original(source.as_bytes()), "{source}");
        }
    }
}

#[test]
fn declaration_ownership_and_utf8_byte_spans_point_into_exact_source() {
    let source: Box<[u8]> = "/*雪😀*/\r\nimport './helper.js';\r\nconst s = `with {}`;\r\nexport {x as y} from './helper.js'\r\n/*not the clause*/ with /*gap*/ {/*inner*/};\r\nimport * as ns from './helper.js';".as_bytes().into();
    let before = source.to_vec();
    let hash = fingerprint(&source);
    let edges = oxc_edges(&source).unwrap();
    assert_eq!(edges.len(), 3);
    assert!(edges[0].clause.is_none());
    assert!(edges[2].clause.is_none());
    let clause = edges[1].clause.unwrap();
    assert_eq!(
        &source[clause.start as usize..clause.end as usize],
        b"{/*inner*/}"
    );
    assert!(edges[1].declaration.start <= clause.start);
    assert!(edges[1].declaration.end >= clause.end);
    assert_eq!(
        &source[edges[1].declaration.start as usize..edges[1].declaration.start as usize + 6],
        b"export"
    );
    assert_eq!(preflight(&source, None), Err(Failure::Attributes));
    assert_eq!(source.as_ref(), before);
    assert_eq!(fingerprint(&source), hash);
    assert!(compile_original(&source));
}

#[test]
fn syntax_errors_and_parser_disagreement_fail_closed() {
    let malformed = b"import './helper.js' with {";
    assert_eq!(oxc_edges(malformed), Err(Failure::OxcSyntax));
    assert_eq!(preflight(malformed, None), Err(Failure::BoaSyntax));
    assert!(!compile_original(malformed));
    assert_eq!(oxc_edges(&[0xff]), Err(Failure::Utf8));
    // Injected summary disagreement proves the gate, not a naturally observed grammar bug.
    let edges = oxc_edges(b"import './helper.js';").unwrap();
    assert_eq!(agree(&[], &edges), Err(Failure::EdgeDisagreement));
    assert_eq!(
        agree(&[("reexport", "./helper.js".into())], &edges),
        Err(Failure::EdgeDisagreement)
    );
    assert_eq!(
        agree(&[("import", "./different.js".into())], &edges),
        Err(Failure::EdgeDisagreement)
    );
    let duplicate = b"export const x = 1; export {x};";
    assert_eq!(oxc_edges(duplicate), Err(Failure::OxcSyntax));
    assert_eq!(preflight(duplicate, None), Err(Failure::BoaSyntax));
    assert!(!compile_original(duplicate));
    // A real grammar disagreement: Oxc still represents legacy assertions.
    let assertion = b"import './helper.js' assert {};";
    assert!(oxc_edges(assertion).unwrap()[0].clause.is_some());
    assert_eq!(preflight(assertion, None), Err(Failure::BoaSyntax));
    assert!(!compile_original(assertion));
    let recovered = b"const missingInitializer;";
    assert_eq!(oxc_edges(recovered), Err(Failure::OxcSyntax));
    assert!(preflight(recovered, None).is_err());
}

#[test]
fn naive_text_checks_are_not_a_syntax_strategy() {
    // Even after any comment removal these strings/templates remain false positives.
    for source in [
        r#"const s = "with {}";"#,
        "const s = `with {}`;",
        "/* with {} */",
        "const withValue = 1;",
    ] {
        assert!(source.contains("with")); // Deliberate bad heuristic, never used by preflight.
        assert_eq!(preflight(source.as_bytes(), None), Ok(()));
    }
    assert!(r#"const s = "with {}";"#.contains("with {"));
    let real = b"import './helper.js' with/*comment*/{};";
    assert!(!std::str::from_utf8(real).unwrap().contains("with {"));
    assert_eq!(preflight(real, None), Err(Failure::Attributes));
    // Comment-looking text in regex/string literals is not removable trivia.
    for source in [
        r#"const s = "/* with {} */";"#,
        r"const r = /\/\* with \{\} \*\//;",
    ] {
        assert_eq!(preflight(source.as_bytes(), None), Ok(()));
        assert!(compile_original(source.as_bytes()));
    }
}

#[test]
fn boa_public_lexer_alone_cannot_follow_parser_lexical_goals() {
    use boa_parser::lexer::Lexer;
    let source = b"const ratio = 6 / 2;";
    assert!(parse(source, &mut Interner::default()).is_ok());
    assert_eq!(preflight(source, None), Ok(()));
    assert!(compile_original(source));
    let mut lexer = Lexer::from(&source[..]);
    let mut interner = Interner::default();
    let mut failed = false;
    for _ in 0..32 {
        match lexer.next(&mut interner) {
            Ok(Some(_)) => {}
            Ok(None) => break,
            Err(_) => {
                failed = true;
                break;
            }
        }
    }
    assert!(
        failed,
        "default public lexer treats division as a regexp start"
    );
}

#[test]
fn previous_corpus_has_no_silent_supplemental_acceptance() {
    for case in super::corpus::corpus() {
        let oxc = oxc_edges(case.text.as_bytes());
        let result = preflight(case.text.as_bytes(), case.operations);
        println!(
            "supplemental-corpus {} Oxc={} hybrid={result:?}",
            case.id,
            oxc.is_ok()
        );
        if case.known_gap {
            assert_eq!(result, Err(Failure::Attributes));
        } else if case.parser && case.violation.is_none() {
            assert_eq!(result, Ok(()), "{}", case.id);
        } else {
            assert!(result.is_err(), "{}", case.id);
        }
    }
}
