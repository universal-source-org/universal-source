//! Hand-authored expectations: syntax acceptance is separate from RFC acceptance.
#[derive(Clone, Copy)]
pub struct Case {
    pub id: &'static str,
    pub text: &'static str,
    pub operations: Option<&'static [&'static str]>,
    pub parser: bool,
    pub quickjs: bool,
    pub violation: Option<&'static str>,
    pub known_gap: bool,
}

pub fn corpus() -> Vec<Case> {
    let home = Some(&["home"][..]);
    let mut cases = Vec::new();
    for (id, text, violation) in [
        (
            "direct-sync",
            "export function home(input, context) {}",
            None,
        ),
        (
            "direct-async",
            "export async function home(input, context) {}",
            None,
        ),
        (
            "generator",
            "export function* home() {}",
            Some("OPERATION_DECLARATION"),
        ),
        (
            "async-generator",
            "export async function* home() {}",
            Some("OPERATION_DECLARATION"),
        ),
        (
            "same-name-list",
            "function home() {} export {home};",
            Some("EXPORT_LIST"),
        ),
        (
            "alias",
            "function impl() {} export {impl as home};",
            Some("EXPORT_LIST"),
        ),
        (
            "reexport",
            "export {home} from './helper.js';",
            Some("ENTRY_REEXPORT"),
        ),
        (
            "reexport-alias",
            "export {impl as home} from './helper.js';",
            Some("ENTRY_REEXPORT"),
        ),
        ("star", "export * from './helper.js';", Some("STAR_EXPORT")),
        (
            "namespace-star",
            "export * as home from './helper.js';",
            Some("STAR_EXPORT"),
        ),
        (
            "default-named",
            "export default function home() {}",
            Some("DEFAULT_EXPORT"),
        ),
        (
            "default-anonymous",
            "export default function() {}",
            Some("DEFAULT_EXPORT"),
        ),
        (
            "default-async",
            "export default async function home() {}",
            Some("DEFAULT_EXPORT"),
        ),
        (
            "const-arrow",
            "export const home = () => {};",
            Some("OPERATION_DECLARATION"),
        ),
        (
            "let-function",
            "export let home = function() {};",
            Some("OPERATION_DECLARATION"),
        ),
        (
            "class",
            "export class home {}",
            Some("OPERATION_DECLARATION"),
        ),
        (
            "const-number",
            "export const home = 42;",
            Some("OPERATION_DECLARATION"),
        ),
        (
            "var-function",
            "export var home = function() {};",
            Some("OPERATION_DECLARATION"),
        ),
        (
            "extra-export",
            "export function home() {} export function extra() {}",
            Some("OPERATION_NAMES"),
        ),
        (
            "missing-export",
            "function home() {}",
            Some("OPERATION_NAMES"),
        ),
        ("escaped-name", r"export function h\u006fme() {}", None),
        ("escaped-codepoint", r"export function h\u{6f}me() {}", None),
        (
            "greek-confusable",
            "export function hοme() {}",
            Some("OPERATION_NAMES"),
        ),
        (
            "cyrillic-confusable",
            "export function hоme() {}",
            Some("OPERATION_NAMES"),
        ),
        (
            "composed-distinct",
            "export function homé() {}",
            Some("OPERATION_NAMES"),
        ),
        (
            "decomposed-distinct",
            "export function homé() {}",
            Some("OPERATION_NAMES"),
        ),
        (
            "unicode-helpers",
            "const café = 1, café = 2; export function home() { return café + café; }",
            None,
        ),
        (
            "format-asi",
            "/*😀 中文*/\r\nconst label = '雪'\r\nexport\u{00a0}function h\\u006fme() {\r\nreturn\u{2028}label\u{2029}}",
            None,
        ),
        (
            "destructuring-params",
            "export function home({x} = {}, ...rest) { return x ?? rest?.[0]; }",
            None,
        ),
        (
            "es2023-class",
            "class C { #x = 1; static { this.x = 2; } get x() { return this.#x; } } export function home() { return new C().x; }",
            None,
        ),
        (
            "es2023-regexp",
            r"export function home() { return /(?<word>\p{L}+)/du; }",
            None,
        ),
        (
            "dynamic-import-operation",
            "export function home() { return import('./helper.js'); }",
            Some("DYNAMIC_IMPORT"),
        ),
        (
            "dynamic-import-helper",
            "function helper() { return import('./helper.js'); } export function home() {}",
            Some("DYNAMIC_IMPORT"),
        ),
        (
            "dynamic-import-unreachable",
            "export function home() { if (false) import('./helper.js'); }",
            Some("DYNAMIC_IMPORT"),
        ),
        (
            "dynamic-import-arrow",
            "export function home() { return () => import('./helper.js'); }",
            Some("DYNAMIC_IMPORT"),
        ),
        (
            "dynamic-import-class",
            "class C { run() { import('./helper.js'); } } export function home() {}",
            Some("DYNAMIC_IMPORT"),
        ),
        (
            "meta-nested",
            "export function home() { return function helper() { if (false) return import.meta; }; }",
            Some("IMPORT_META"),
        ),
        (
            "meta-class-key",
            "class C { [import.meta.url]() {} } export function home() {}",
            Some("IMPORT_META"),
        ),
        (
            "static-side-effect",
            "import './helper.js'; export function home() {}",
            None,
        ),
        (
            "static-named",
            "import {x} from './helper.js'; export function home() { return x; }",
            None,
        ),
        (
            "static-namespace",
            "import * as helper from './helper.js'; export function home() { return helper.x; }",
            None,
        ),
        (
            "static-escaped-specifier",
            r"import {x} from '.\u002fhelper.js'; export function home() { return x; }",
            None,
        ),
        (
            "attributes",
            "import './helper.js' with {type:'json'}; export function home() {}",
            Some("IMPORT_ATTRIBUTES"),
        ),
        (
            "top-await",
            "await something; export function home() {}",
            Some("TOP_LEVEL_AWAIT"),
        ),
        (
            "top-await-block",
            "if (false) { await something; } export function home() {}",
            Some("TOP_LEVEL_AWAIT"),
        ),
        (
            "top-for-await",
            "for await (const x of []) {} export function home() {}",
            Some("TOP_LEVEL_AWAIT"),
        ),
        (
            "top-await-class-key",
            "class C { [await 1]() {} } export function home() {}",
            Some("TOP_LEVEL_AWAIT"),
        ),
        (
            "nested-await",
            "export async function home() { await something; }",
            None,
        ),
        (
            "nested-arrow-await",
            "export function home() { return async () => await 1; }",
            None,
        ),
        (
            "nested-for-await",
            "export async function home() { for await (const x of []) {} }",
            None,
        ),
        (
            "dynamic-code-runtime",
            "export function home() { try { eval('1'); new Function('return 1'); } catch (e) {} }",
            None,
        ),
        (
            "dynamic-code-alias-runtime",
            "export function home() { const ctor = (() => {}).constructor; return ctor('return 1'); }",
            None,
        ),
        (
            "shadowed-function",
            "export function home() { const Function = () => 1; return Function(); }",
            None,
        ),
    ] {
        cases.push(Case {
            id,
            text,
            operations: home,
            parser: true,
            quickjs: true,
            violation,
            known_gap: false,
        });
    }
    for (id, text) in [
        ("duplicate", "export function home() {} export {home};"),
        ("malformed", "export function home( {"),
        (
            "assertion",
            "import './helper.js' assert {type:'json'}; export function home() {}",
        ),
    ] {
        cases.push(Case {
            id,
            text,
            operations: home,
            parser: false,
            quickjs: false,
            violation: None,
            known_gap: false,
        });
    }
    for (id, text) in [
        ("helper-list", "const x = 1; export {x};"),
        ("helper-reexport", "export {x} from './helper.js';"),
    ] {
        cases.push(Case {
            id,
            text,
            operations: None,
            parser: true,
            quickjs: true,
            violation: None,
            known_gap: false,
        });
    }
    for (id, text, operations) in [
        (
            "empty-import-attributes",
            "import './helper.js' with {}; export function home() {}",
            home,
        ),
        (
            "empty-reexport-attributes",
            "export {x} from './helper.js' with {};",
            None,
        ),
    ] {
        cases.push(Case {
            id,
            text,
            operations,
            parser: true,
            quickjs: true,
            violation: None,
            known_gap: true,
        });
    }
    // Contemporary syntax controls are outside the RFC's ES2023 baseline.
    cases.push(Case {
        id: "import-source-phase",
        text: "export function home() { return import.source('./helper.js'); }",
        operations: home,
        parser: true,
        quickjs: false,
        violation: Some("DYNAMIC_IMPORT"),
        known_gap: false,
    });
    cases.push(Case {
        id: "using-extension",
        text: "export function home() { using x = null; }",
        operations: home,
        parser: true,
        quickjs: true,
        violation: Some("OUTSIDE_ES2023"),
        known_gap: false,
    });
    cases
}
