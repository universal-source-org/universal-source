use super::*;
use std::{collections::BTreeMap, time::Instant};

/// Probe-only threshold for demonstrating a decision point. Not a proposed
/// portable or host policy value.
const PROBE_LIMIT: usize = 4096;

fn sorted(flags: &str) -> String {
    let mut chars: Vec<char> = flags.chars().collect();
    chars.sort_unstable();
    chars.into_iter().collect()
}

#[test]
fn literal_compiles_at_declaration_on_the_non_polling_path() {
    // Control: the same armed handler does interrupt RegExp matching.
    let (rt, context, control) = armed_realm();
    context.with(|ctx| {
        control.armed.set(true);
        let result = ctx.eval::<bool, _>("/^(a+)+$/.test('a'.repeat(30) + '!')");
        control.armed.set(false);
        assert!(result.is_err());
        drop(ctx.catch());
    });
    assert!(control.callbacks.get() >= 1);
    drop(context);
    drop(rt);

    // A literal is compiled by the parser during compile-only declaration:
    // no bytecode runs, so no interrupt site is reached. An invalid tail still
    // pays the forward-reference scans before the syntax error.
    let valid = forward_reference_pattern(4000);
    let invalid = format!("{valid}(");
    for (label, body, expected) in [("valid", valid, true), ("invalid-tail", invalid, false)] {
        let source = format!("export const r = /{body}/;\n");
        let start = Instant::now();
        let (accepted, callbacks) = quickjs_declare(&source);
        eprintln!(
            "LITERAL_DECLARE {label}: pattern_units={} accepted={accepted} callbacks={callbacks} elapsed_ms={}",
            body.encode_utf16().count(),
            start.elapsed().as_millis()
        );
        assert_eq!(accepted, expected, "{label}");
        assert_eq!(callbacks, 0, "{label}");
    }
}

#[test]
fn every_literal_position_is_compiled_at_declaration_and_seen_by_boa() {
    let contexts = [
        ("entry top level", "export const r = @@;"),
        (
            "imported helper module",
            "export function helper(s) { return @@.test(s); }",
        ),
        (
            "unreachable branch",
            "export function f() { if (false) { return @@; } return 0; }",
        ),
        (
            "nested function",
            "export function f() { function g() { return () => @@; } return g; }",
        ),
        (
            "class method and static field",
            "export class C { m() { return @@; } static s = @@; }",
        ),
        (
            "template substitution",
            "export const t = `a${@@.source}b`;",
        ),
        (
            "statement after if",
            "export function f(x) { if (x) @@.test(x); return 0; }",
        ),
        (
            "default parameter",
            "export function f(r = @@) { return r; }",
        ),
        ("generator yield", "export function* g() { yield @@; }"),
        (
            "async await",
            "export async function f() { return await @@; }",
        ),
        ("object and array", "export const o = { k: [@@] };"),
    ];
    for (label, template) in contexts {
        let count = template.matches("@@").count();
        let valid = template.replace("@@", "/ok+/g");
        let invalid = template.replace("@@", "/(/");

        let (accepted, callbacks) = quickjs_declare(&valid);
        assert!(accepted && callbacks == 0, "{label}: valid declare");
        let (accepted, _) = quickjs_declare(&invalid);
        assert!(
            !accepted,
            "{label}: QuickJS must compile the literal at declaration"
        );

        let literals = boa_literals(valid.as_bytes()).unwrap();
        assert_eq!(literals.len(), count, "{label}");
        for literal in &literals {
            assert_eq!(literal.pattern, "ok+", "{label}");
            assert_eq!(literal.flags, "g", "{label}");
        }
        assert!(
            boa_literals(invalid.as_bytes()).is_err(),
            "{label}: Boa regress validation"
        );

        assert!(oxc_accepts(&valid, true), "{label}");
        assert!(
            !oxc_accepts(&invalid, true),
            "{label}: Oxc pattern validation"
        );
        assert!(oxc_accepts(&invalid, false), "{label}: Oxc raw-only mode");
    }
}

#[test]
fn division_regexp_ambiguity_escapes_and_flags_match_quickjs() {
    let cases: &[(&str, &[(&str, &str)])] = &[
        ("export const q = 4 /2/ 1;", &[]),
        ("export function f(a, b, g) { return a /(b)/ g; }", &[]),
        ("export const o = ({}) / 2;", &[]),
        ("export let x = 8; x /= 2; export const y = x /2/ 1;", &[]),
        ("{}\n/re/g.test('re');\nexport const z = 0;", &[("re", "g")]),
        (
            "export function f(x) { if (x) /y+/g.test(x); return 0; }",
            &[("y+", "g")],
        ),
        ("export const s = /\\//;", &[("\\/", "")]),
        ("export const c = /[/]/;", &[("[/]", "")]),
        (
            "export const u = /\\u{61}\\u0062/u;",
            &[("\\u{61}\\u0062", "u")],
        ),
        ("export const f = /a/dgimsuy;", &[("a", "dgimsuy")]),
        ("export const n = /é+/;", &[("é+", "")]),
        ("export const d = 1 / 2, e = /3/;", &[("3", "")]),
        (
            "export const k = '/no/' + /*/c/*/ `/t/${'/'}` + /real/;",
            &[("real", "")],
        ),
        ("export const eq = /=a/;", &[("=a", "")]),
    ];
    for (source, expected) in cases {
        let (accepted, callbacks) = quickjs_declare(source);
        assert!(accepted && callbacks == 0, "QuickJS declare: {source}");
        assert!(oxc_accepts(source, true), "Oxc: {source}");
        let literals = boa_literals(source.as_bytes()).unwrap();
        let actual: Vec<(String, String)> = literals
            .iter()
            .map(|l| (l.pattern.clone(), sorted(&l.flags)))
            .collect();
        let wanted: Vec<(String, String)> = expected
            .iter()
            .map(|(p, f)| (p.to_string(), sorted(f)))
            .collect();
        assert_eq!(actual, wanted, "{source}");
        for literal in &literals {
            // The raw body is present verbatim in the unchanged source bytes.
            assert!(
                source.contains(&format!("/{}/", literal.pattern)),
                "{source}"
            );
            assert_eq!(literal.utf16_len, literal.pattern.encode_utf16().count());
        }
    }
    let wide = boa_literals("export const n = /é+/;".as_bytes()).unwrap();
    assert_eq!(wide[0].utf16_len, 2);
}

#[test]
fn oversized_literal_is_measured_and_rejected_before_quickjs() {
    // 16,000 references: ~1.6 s and 64,000: >5 s of non-polling QuickJS
    // compile in the resource spike. Neither is handed to QuickJS here.
    for references in [16_000, 64_000] {
        let body = forward_reference_pattern(references);
        let source = format!("export const r = /{body}/;\n");
        let start = Instant::now();
        let literals = boa_literals(source.as_bytes()).unwrap();
        let boa_ms = start.elapsed().as_millis();
        let start = Instant::now();
        let oxc_raw = oxc_accepts(&source, false);
        let oxc_raw_ms = start.elapsed().as_millis();
        let start = Instant::now();
        let oxc_validated = oxc_accepts(&source, true);
        let oxc_validated_ms = start.elapsed().as_millis();
        eprintln!(
            "OVERSIZED_LITERAL references={references} boa_ms={boa_ms} oxc_raw_ms={oxc_raw_ms} oxc_validated_ms={oxc_validated_ms}"
        );
        assert_eq!(literals.len(), 1);
        assert_eq!(literals[0].utf16_len, references * 5 + 7);
        assert!(oxc_raw && oxc_validated);
        let admitted = literals.iter().all(|l| l.utf16_len <= PROBE_LIMIT);
        assert!(
            !admitted,
            "preflight decision precedes any QuickJS compilation"
        );
    }
}

const ROUTES: &str = r#"(() => {
  const NativeRegExp = RegExp;
  let hits = 0;
  // Negative control only: a naive global replacement that refuses everything.
  function Facade() { hits++; throw new TypeError("facade"); }
  Facade.prototype = NativeRegExp.prototype;
  globalThis.RegExp = Facade;
  NativeRegExp.prototype.constructor = Facade;
  const classify = (f) => {
    const before = hits;
    try { f(); return "ok"; }
    catch (e) {
      if (hits > before) return "facade";
      return e instanceof SyntaxError ? "native-compile" : "other:" + e.name;
    }
  };
  const fake = (extra) => Object.assign({ constructor: undefined, flags: "", lastIndex: 0,
    toString() { return "("; } }, extra);
  const out = [];
  const route = (name, f) => out.push(name + "=" + classify(f));
  route("new_RegExp", () => new RegExp("("));
  route("call_RegExp", () => RegExp("("));
  route("reflect_construct", () => Reflect.construct(RegExp, ["("]));
  route("bound_construct", () => new (RegExp.bind(null))("("));
  route("subclass", () => { class X extends RegExp {} return new X("("); });
  route("literal_constructor", () => new (/x/.constructor)("("));
  route("string_match", () => "a(b".match("("));
  route("string_matchAll", () => [..."a(b".matchAll("(")]);
  route("string_search", () => "a(b".search("("));
  route("prototype_compile", () => /x/.compile("("));
  route("species_split_fake", () => NativeRegExp.prototype[Symbol.split].call(fake(), "a(b"));
  route("species_matchAll_fake", () => NativeRegExp.prototype[Symbol.matchAll].call(fake({ flags: "g" }), "a(b"));
  route("species_split_genuine_default", () => "a(b".split(/\(/));
  route("species_split_genuine_source_override", () => {
    const r = /\(/;
    Object.defineProperty(r, "source", { get() { return "("; } });
    r.constructor = undefined;
    return "a(b".split(r);
  });
  route("string_split", () => "a(b".split("("));
  route("string_replace", () => "a(b".replace("(", "-"));
  route("string_replaceAll", () => "a(b".replaceAll("(", "-"));
  route("regexp_exec", () => /x/.exec("("));
  route("regexp_symbol_replace", () => "a(b".replace(/\(/, "-"));
  Object.defineProperty(Facade, Symbol.species, { get() { return this; } });
  route("species_split_genuine_with_facade_species", () => "a(b".split(/\(/));
  return out.join("\n");
})()"#;

#[test]
fn replaced_global_regexp_is_bypassed_by_builtin_compile_routes() {
    let (rt, context, _control) = armed_realm();
    let observed: BTreeMap<String, String> = context.with(|ctx| {
        ctx.eval::<String, _>(ROUTES)
            .unwrap()
            .lines()
            .map(|line| {
                let (k, v) = line.split_once('=').unwrap();
                (k.to_owned(), v.to_owned())
            })
            .collect()
    });
    for (route, class) in &observed {
        eprintln!("ROUTE {route} -> {class}");
    }
    let expected = [
        // Source-visible constructor paths reach the replaced global.
        ("new_RegExp", "facade"),
        ("call_RegExp", "facade"),
        ("reflect_construct", "facade"),
        ("bound_construct", "facade"),
        ("subclass", "facade"),
        ("literal_constructor", "facade"),
        // Internal ctx->regexp_ctor or direct compile: facade bypassed.
        ("string_match", "native-compile"),
        ("string_matchAll", "native-compile"),
        ("string_search", "native-compile"),
        ("prototype_compile", "native-compile"),
        ("species_split_fake", "native-compile"),
        ("species_matchAll_fake", "native-compile"),
        // Genuine receiver: native default reuses the internal source.
        ("species_split_genuine_default", "ok"),
        ("species_split_genuine_source_override", "ok"),
        // String-search and execute-only routes: no compilation of "(".
        ("string_split", "ok"),
        ("string_replace", "ok"),
        ("string_replaceAll", "ok"),
        ("regexp_exec", "ok"),
        ("regexp_symbol_replace", "ok"),
        // A resolvable species reaches the source-visible constructor.
        ("species_split_genuine_with_facade_species", "facade"),
    ];
    assert_eq!(observed.len(), expected.len());
    for (route, class) in expected {
        assert_eq!(observed[route], class, "{route}");
    }
    drop(context);
    drop(rt);
}

#[test]
fn source_coercion_completes_before_native_compilation() {
    let (rt, context, _control) = armed_realm();
    context.with(|ctx| {
        let order: String = ctx
            .eval(
                r#"(() => {
  const log = [];
  const pattern = {
    get [Symbol.match]() { log.push("isRegExp"); return undefined; },
    [Symbol.toPrimitive](hint) { log.push("pattern:" + hint); return "("; },
  };
  const flags = { toString() { log.push("flags"); return "g"; } };
  try { new RegExp(pattern, flags); } catch (e) { log.push(e.name); }
  return log.join(",");
})()"#,
            )
            .unwrap();
        assert_eq!(order, "isRegExp,pattern:string,flags,SyntaxError");

        // ES2023 RegExp(pattern, flags) performs RegExpAlloc(newTarget) before
        // RegExpInitialize's ToString. Pinned QuickJS reads newTarget.prototype
        // only after a successful compile.
        let alloc: String = ctx
            .eval(
                r#"(() => {
  const log = [];
  const nt = new Proxy(function () {}, { get(t, k, r) {
    if (k === "prototype") log.push("prototype"); return Reflect.get(t, k, r); } });
  const pattern = { toString() { log.push("pattern"); return "("; } };
  try { Reflect.construct(RegExp, [pattern], nt); } catch (e) { log.push(e.name); }
  return log.join(",");
})()"#,
            )
            .unwrap();
        assert_eq!(alloc, "pattern,SyntaxError");

        // ES2023 String.prototype.match uses RegExpCreate(regexp): ToString of
        // a genuine RegExp is "/a/". Pinned QuickJS runs its full constructor
        // and reuses the internal source "a".
        let create: String = ctx
            .eval(r#"(() => { const r = /a/; r[Symbol.match] = undefined; return "/a/".match(r)[0]; })()"#)
            .unwrap();
        assert_eq!(create, "a");
    });
    drop(context);
    drop(rt);
}

#[test]
fn native_global_match_loops_reach_the_interrupt_hook() {
    // Observation only, not a matching bound. Each lre_exec call restarts its
    // own 10,000-step poll counter; these native loops are nevertheless
    // interrupted, consistent with their per-match exec calls entering
    // JS_CallInternal's shared poll counter.
    for (label, body) in [
        ("replace", "input.replace(/a/g, 'b')"),
        ("split", "input.split(/a/)"),
        ("match", "input.match(/a/g)"),
        ("matchAll", "[...input.matchAll(/a/g)]"),
    ] {
        let (rt, context, control) = armed_realm();
        context.with(|ctx| {
            let run: rquickjs::Function = ctx.eval(format!("(input) => {body}")).unwrap();
            let input: String = ctx.eval("'a'.repeat(200000)").unwrap();
            control.armed.set(true);
            let start = Instant::now();
            let result = run.call::<_, rquickjs::Value>((input,));
            let elapsed = start.elapsed().as_millis();
            control.armed.set(false);
            eprintln!(
                "GLOBAL_MATCH_LOOP {label}: interrupted={} callbacks={} elapsed_ms={elapsed}",
                result.is_err(),
                control.callbacks.get()
            );
            assert!(result.is_err(), "{label}");
            assert_eq!(control.callbacks.get(), 1, "{label}");
            drop(ctx.catch());
        });
        drop(context);
        drop(rt);
    }
}

#[test]
fn post_coercion_length_check_stops_reproducer_before_compile() {
    // Ordering witness only, not an admission facade: the exact reproducer's
    // text exists as an ordinary string before any native compiler entry.
    let (rt, context, control) = armed_realm();
    context.with(|ctx| {
        let gate: rquickjs::Function = ctx
            .eval(format!(
                r#"(references) => {{
  const NativeRegExp = RegExp;
  const pattern = {{ toString() {{ return "\\k<a>".repeat(references) + "(?<a>x)"; }} }};
  const text = String(pattern);
  if (text.length > {PROBE_LIMIT}) return "rejected:" + text.length;
  return "admitted:" + new NativeRegExp(text).source.length;
}}"#
            ))
            .unwrap();
        control.armed.set(true);
        let start = Instant::now();
        let big: String = gate.call((64_000u32,)).unwrap();
        let elapsed = start.elapsed().as_millis();
        let small: String = gate.call((10u32,)).unwrap();
        control.armed.set(false);
        eprintln!("POST_COERCION big={big} elapsed_ms={elapsed} small={small}");
        assert_eq!(big, "rejected:320007");
        assert!(small.starts_with("admitted:"));
        assert_eq!(control.callbacks.get(), 0);
    });
    drop(context);
    drop(rt);
}
