use super::*;
use crate::preflight::{self, Graph, GraphResolver, Literal, Rejection};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

const TRAVERSE: &str = include_str!("../../quickjs-global-surface-spike/src/traverse.js");
const REACH: &str = include_str!("reach.js");
/// The preserved 64,000-reference reproducer, built by source at run time.
const REPRODUCER: &str = r#"const P = "\\k<a>".repeat(64000) + "(?<a>x)";"#;
/// Over the bound and syntactically invalid: a native compile would throw a catchable SyntaxError.
const INVALID: &str = r#"const P = "(" + "a".repeat(4096);"#;

struct Outcome {
    ok: bool,
    uncatchable: bool,
    latch: Option<Reason>,
    compiles: usize,
    rejected: usize,
    notes: Vec<String>,
    elapsed: Duration,
}

fn run(realm: &Realm, source: &str) -> Outcome {
    realm.context.with(|ctx| {
        let start = Instant::now();
        let result = ctx.eval::<(), _>(source);
        let elapsed = start.elapsed();
        let (ok, uncatchable) = match result {
            Ok(()) => (true, false),
            Err(_) => (false, ctx.catch().is_uncatchable_error()),
        };
        Outcome {
            ok,
            uncatchable,
            latch: realm.gate.latch.get(),
            compiles: realm.gate.compiles(),
            rejected: realm.gate.rejected.get(),
            notes: realm.gate.notes.borrow().clone(),
            elapsed,
        }
    })
}

fn assert_fresh_realm_healthy() {
    let realm = hardened_realm(L_MAX);
    let out = run(
        &realm,
        "if (new RegExp('a+').test('aa') && /b/.test('b') && 'xay'.search('a') === 1) note('healthy');",
    );
    assert!(out.ok && out.latch.is_none(), "fresh realm unhealthy");
    assert_eq!(out.notes, ["healthy"]);
}

/// Every source-visible compile route from the route inventory.
const ROUTES: &[&str] = &[
    "new RegExp(P)",
    "RegExp(P)",
    "new RegExp(P, 'g')",
    "RegExp({ toString: () => P })",
    "new RegExp({ [Symbol.match]: true, source: P, flags: '' })",
    "RegExp({ [Symbol.match]: true, source: P, flags: '', constructor: null })",
    "const R = RegExp; new R(P)",
    "RegExp.call(null, P)",
    "RegExp.apply(null, [P])",
    "RegExp.bind(null)(P)",
    "new (RegExp.bind(null))(P)",
    "Reflect.construct(RegExp, [P])",
    "Reflect.construct(RegExp, [P], function F() {})",
    "new (class extends RegExp {})(P)",
    "new (/a/).constructor(P)",
    "new RegExp.prototype.constructor(P)",
    "'s'.match(P)",
    "'s'.match({ toString: () => P })",
    "String.prototype.match.call('s', P)",
    "'s'.matchAll(P)",
    "'s'.search(P)",
    "/a/.compile(P)",
    "/a/.compile(P, 'g')",
    "RegExp.prototype.compile.call(/a/, { toString: () => P })",
    "RegExp.prototype[Symbol.split].call({ [Symbol.match]: true, source: P, flags: '', constructor: undefined }, 's')",
    "RegExp.prototype[Symbol.split].call({ [Symbol.match]: true, source: P, flags: '', constructor: { [Symbol.species]: RegExp } }, 's')",
    "RegExp.prototype[Symbol.split].call({ toString: () => P, flags: '', constructor: undefined }, 's')",
    "RegExp.prototype[Symbol.matchAll].call({ [Symbol.match]: true, source: P, flags: 'g', lastIndex: 0, constructor: undefined }, 's')",
    "'s'.split({ [Symbol.split]: RegExp.prototype[Symbol.split], [Symbol.match]: true, source: P, flags: '', constructor: undefined })",
    "'s'.matchAll({ [Symbol.matchAll]: RegExp.prototype[Symbol.matchAll], [Symbol.match]: true, source: P, flags: 'g', lastIndex: 0, constructor: undefined })",
];

#[test]
fn every_dynamic_route_rejects_before_native_compile_with_trusted_latch() {
    for prelude in [REPRODUCER, INVALID] {
        for route in ROUTES {
            let realm = hardened_realm(L_MAX);
            let source = format!(
                "{prelude}\ntry {{ {route}; note('returned'); }} catch (e) {{ note('caught'); }} \
                 finally {{ note('finally'); }}\nnote('after');"
            );
            let out = run(&realm, &source);
            let context = format!("route `{route}` with `{prelude}`");
            assert!(
                !out.ok && out.uncatchable,
                "{context}: not an uncatchable stop"
            );
            assert_eq!(out.latch, Some(Reason::ResourceLimit), "{context}");
            assert_eq!(out.compiles, 0, "{context}: native compile admitted");
            assert_eq!(out.rejected, 1, "{context}");
            assert!(
                out.notes.is_empty(),
                "{context}: source ran after stop: {:?}",
                out.notes
            );
            assert!(
                out.elapsed < Duration::from_secs(1),
                "{context}: {:?}",
                out.elapsed
            );
            drop(realm);
            assert_fresh_realm_healthy();
        }
    }
    println!(
        "{} routes x 2 patterns rejected with zero compile",
        ROUTES.len()
    );
}

#[test]
fn non_compiling_routes_and_genuine_internal_source_are_not_rejected() {
    let realm = hardened_realm(L_MAX);
    let out = run(
        &realm,
        &format!(
            "{REPRODUCER}
            's'.replace(P, ''); 's'.replaceAll(P, ''); 's'.split(P); /a/.exec(P);
            /a/[Symbol.replace](P, ''); 's'.includes(P);
            const r = /a/g;
            Object.defineProperty(r, 'source', {{ get() {{ return P; }} }});
            new RegExp(r); new RegExp(r, 'y'); r[Symbol.split]('s'); 's'.match(r);
            [...'aa'.matchAll(r)]; 's'.replace(r, '');
            note('done');"
        ),
    );
    assert!(out.ok && out.latch.is_none(), "{:?}", out.notes);
    assert_eq!(out.notes, ["done"]);
    assert_eq!(realm.gate.admitted.get(), 0);
    // `new RegExp(r, 'y')`, the split species construct and matchAll's construct.
    assert_eq!(realm.gate.readmitted.get(), 3);
}

#[test]
fn bound_edges_and_genuine_representation() {
    let realm = hardened_realm(L_MAX);
    let out = run(
        &realm,
        &format!(
            "new RegExp('a'.repeat({L_MAX})); note('at bound');
            const r = new RegExp('/'.repeat({L_MAX}));
            if (r.source.length > {L_MAX}) note('escaped source exceeds bound');
            const c = new RegExp(r, 'gi');
            if (c.source === r.source && c.flags === 'gi') note('clone with flags');
            if (RegExp(r, 'y').sticky && r[Symbol.split]('//').length === 1) note('species clone');"
        ),
    );
    assert!(out.ok, "{:?}", out.notes);
    assert_eq!(
        out.notes,
        [
            "at bound",
            "escaped source exceeds bound",
            "clone with flags",
            "species clone"
        ]
    );
    assert_eq!(realm.gate.admitted.get(), 2);
    assert_eq!(realm.gate.readmitted.get(), 3);
    let over = hardened_realm(L_MAX);
    let out = run(&over, &format!("new RegExp('a'.repeat({}));", L_MAX + 1));
    assert!(!out.ok && out.uncatchable && out.compiles == 0);
    assert_eq!(out.latch, Some(Reason::ResourceLimit));
}

#[test]
fn facade_preserves_identity_shape_and_es2023_semantics() {
    let realm = hardened_realm(L_MAX);
    realm.context.with(|ctx| {
        let failed: String = eval(
            &ctx,
            r#"const checks = {
                name: RegExp.name === 'RegExp', length: RegExp.length === 2,
                typeof: typeof RegExp === 'function',
                literalPrototype: Object.getPrototypeOf(/x/) === RegExp.prototype,
                instanceofLiteral: /x/ instanceof RegExp,
                constructorLink: /x/.constructor === RegExp && RegExp.prototype.constructor === RegExp,
                species: RegExp[Symbol.species] === RegExp,
                construct: new RegExp('a', 'g').flags === 'g' && new RegExp('a').test('a'),
                callFormSame: (() => { const r = /x/g; return RegExp(r) === r; })(),
                callFormFlags: (() => { const r = /x/g; const c = RegExp(r, 'i');
                    return c !== r && c.flags === 'i' && c.source === 'x'; })(),
                clone: (() => { const r = /x/g; r.lastIndex = 2; const c = new RegExp(r);
                    return c !== r && c.flags === 'g' && c.lastIndex === 0; })(),
                subclass: (() => { class S extends RegExp { get tag() { return 1; } }
                    const s = new S('a', 'g');
                    return s instanceof S && s instanceof RegExp && s.tag === 1 && s.test('a')
                        && Object.getPrototypeOf(S) === RegExp; })(),
                reflectNewTarget: (() => { class S extends RegExp {}
                    return Object.getPrototypeOf(Reflect.construct(RegExp, ['a'], S)) === S.prototype; })(),
                lastIndexDescriptor: JSON.stringify(Object.getOwnPropertyDescriptor(new RegExp('a'), 'lastIndex'))
                    === '{"value":0,"writable":true,"enumerable":false,"configurable":false}',
                globalDescriptor: (() => { const d = Object.getOwnPropertyDescriptor(globalThis, 'RegExp');
                    return d.writable && !d.enumerable && d.configurable; })(),
                methodShapes: String.prototype.match.name === 'match' && String.prototype.matchAll.length === 1
                    && RegExp.prototype.compile.length === 2 && RegExp.prototype[Symbol.split].name === '[Symbol.split]'
                    && RegExp.prototype[Symbol.matchAll].length === 1,
                match: 'abc'.match('b')[0] === 'b' && 'abc'.match(/c/)[0] === 'c',
                matchAll: [...'a1a2'.matchAll('a')].length === 2 && [...'a1a2'.matchAll(/\d/g)].length === 2,
                matchAllIterator: Object.prototype.toString.call(''.matchAll(/a/g)) === '[object RegExp String Iterator]',
                search: 'abc'.search('c') === 2,
                split: 'a,b'.split(/,/).join() === 'a,b' && 'a,b,c'.split(/,/, 2).join() === 'a,b',
                compile: (() => { const r = /a/g; r.lastIndex = 3; r.compile('b', 'i');
                    return r.source === 'b' && r.flags === 'i' && r.lastIndex === 0; })(),
                syntaxError: (() => { try { new RegExp('['); } catch (e) { return e instanceof SyntaxError; } })(),
                flagsError: (() => { try { new RegExp('a', 'gg'); } catch (e) { return e instanceof SyntaxError; } })(),
                compileBrand: (() => { try { RegExp.prototype.compile.call({}); } catch (e) { return e instanceof TypeError; } })(),
                matchAllNonGlobal: (() => { try { 'a'.matchAll(/a/); } catch (e) { return e instanceof TypeError; } })(),
                // ES2023 RegExpCreate stringifies instead of cloning.
                regExpCreate: (() => { const r = /a/; r[Symbol.match] = undefined; return '/a/'.match(r)[0] === '/a/'; })(),
            };
            Object.entries(checks).filter(([, v]) => v !== true).map(([k]) => k).join()"#,
        );
        assert_eq!(failed, "", "identity/semantics checks failed");
    });
    assert!(realm.gate.latch.get().is_none());
}

/// Observable order for a RegExp-like pattern with an accessor `prototype` on newTarget.
const ORDER: &str = r#"(() => {
    const log = [];
    const p = { get [Symbol.match]() { log.push('match'); return true; },
        get source() { log.push('source'); return { toString() { log.push('toString:P'); return 'a'; } }; },
        get flags() { log.push('flags'); return { toString() { log.push('toString:F'); return 'g'; } }; } };
    const nt = (function () {}).bind();
    Object.defineProperty(nt, 'prototype', { get() { log.push('prototype'); return RegExp.prototype; } });
    const r = Reflect.construct(RegExp, [p], nt);
    return log.join() + '|' + r.source + '/' + r.flags;
})()"#;

#[test]
fn facade_coerces_once_in_es2023_order_unlike_pinned_native() {
    let realm = hardened_realm(L_MAX);
    let facade: String = realm.context.with(|ctx| eval(&ctx, ORDER));
    assert_eq!(
        facade,
        "match,source,flags,prototype,toString:P,toString:F|a/g"
    );
    let rt = Runtime::new().unwrap();
    let native: String = Context::full(&rt).unwrap().with(|ctx| eval(&ctx, ORDER));
    println!("facade: {facade}\nnative: {native}");
    assert_eq!(
        native,
        "match,source,flags,toString:P,toString:F,prototype|a/g"
    );
}

/// Generic receivers: the facade's split/matchAll must match the native observable sequence.
const GENERIC: &str = r#"(() => {
    const log = [];
    const L = s => log.push(s);
    const splitRx = {
        get constructor() { L('constructor'); return { get [Symbol.species]() { L('species');
            return function (r, f) { L('construct:' + f); return new RegExp(',', f); }; } }; },
        get flags() { L('flags'); return 'g'; } };
    const parts = RegExp.prototype[Symbol.split].call(splitRx,
        { toString() { L('S'); return 'a,b,c'; } }, { valueOf() { L('limit'); return 2; } });
    const matchRx = {
        get constructor() { L('constructor'); return { [Symbol.species]: function (r, f) {
            L('construct:' + f); return new RegExp('a', f); } }; },
        get flags() { L('flags'); return 'g'; },
        get lastIndex() { L('lastIndex'); return { valueOf() { L('ToLength'); return 1; } }; } };
    const it = RegExp.prototype[Symbol.matchAll].call(matchRx, { toString() { L('S'); return 'aaa'; } });
    const indexes = [...it].map(m => m.index);
    const plain = RegExp.prototype[Symbol.split].call({ constructor: undefined, flags: '',
        [Symbol.match]: true, source: ',' }, 'x,y');
    return JSON.stringify([log, parts, indexes, plain, Object.prototype.toString.call(it)]);
})()"#;

#[test]
fn generic_split_and_match_all_are_observably_faithful_to_native() {
    let realm = hardened_realm(L_MAX);
    let facade: String = realm.context.with(|ctx| eval(&ctx, GENERIC));
    let rt = Runtime::new().unwrap();
    let native: String = Context::full(&rt).unwrap().with(|ctx| eval(&ctx, GENERIC));
    println!("generic receivers: {facade}");
    assert_eq!(facade, native);
    assert!(facade.contains(r#"["S","constructor","species","flags","construct:gy","limit""#));
}

#[test]
fn native_constructor_and_wrapped_natives_are_unreachable() {
    let gate = Rc::new(Gate::default());
    gate.bound.set(L_MAX);
    let rt = runtime(&gate);
    let context = Context::full(&rt).unwrap();
    context.with(|ctx| {
        let installer = capture(&ctx);
        // Host-held witnesses built before hardening; no package receives them.
        let make_traverse: Function = eval(&ctx, TRAVERSE);
        let globals: Value = eval(&ctx, GLOBALS);
        let traverse: Function = make_traverse.call((globals,)).unwrap();
        let natives: Value = eval(
            &ctx,
            "[[RegExp, 'native RegExp'], [RegExp.prototype.compile, 'native compile'],
             [RegExp.prototype[Symbol.split], 'native @@split'],
             [RegExp.prototype[Symbol.matchAll], 'native @@matchAll'],
             [String.prototype.match, 'native match'], [String.prototype.matchAll, 'native matchAll'],
             [String.prototype.search, 'native search']]",
        );
        let make_reach: Function = eval(&ctx, REACH);
        let reach: Function = make_reach.call((natives,)).unwrap();
        let before: String = reach.call((false,)).unwrap();
        assert!(before.starts_with("forbidden:"), "{before}");
        harden(&ctx);
        let unwrapped: String = reach.call((false,)).unwrap();
        assert!(unwrapped.starts_with("forbidden:"), "{unwrapped}");
        install(&ctx, installer, &gate);
        let after: String = reach.call((true,)).unwrap();
        let hardened: String = traverse.call((true,)).unwrap();
        println!("negative control: {before}\nfacade walk: {after}\nglobal-surface walk: {hardened}");
        assert!(after.starts_with("clean:"), "{after}");
        assert!(hardened.starts_with("clean:"), "{hardened}");
    });
    assert!(gate.latch.get().is_none());
}

#[test]
fn stop_is_uncatchable_through_catch_finally_async_and_reactions() {
    let cases = [
        "try { new RegExp(P); } catch { note('caught'); } finally { note('finally'); }",
        "(async () => { try { await null; new RegExp(P); } catch { note('caught'); } finally { note('finally'); } })();",
        "Promise.resolve().then(() => new RegExp(P)).catch(() => note('caught')).finally(() => note('finally'));",
        "function* g() { try { yield 1; new RegExp(P); } finally { note('finally'); } } const it = g(); it.next(); try { it.next(); } catch { note('caught'); }",
        "try { Error.prepareStackTrace = () => note('prepare'); } catch {} try { Error.stackTraceLimit = { valueOf() { note('limit'); return 1; } }; } catch {} new RegExp(P);",
        "[1].forEach(() => { try { new RegExp(P); } finally { note('finally'); } }); note('after');",
        "for (const x of { [Symbol.iterator]() { return { next: () => ({ done: false }), return() { note('return'); return {}; } }; } }) { new RegExp(P); }",
    ];
    for case in cases {
        let realm = hardened_realm(L_MAX);
        let source = format!("{REPRODUCER}\n{case}");
        realm
            .context
            .with(|ctx| drop(ctx.eval::<(), _>(source.as_str())));
        for _ in 0..16 {
            if realm.gate.latch.get().is_some() || !realm.rt.is_job_pending() {
                break;
            }
            drop(realm.rt.execute_pending_job());
        }
        assert_eq!(
            realm.gate.latch.get(),
            Some(Reason::ResourceLimit),
            "{case}"
        );
        assert_eq!(realm.gate.compiles(), 0, "{case}");
        assert!(
            realm.gate.notes.borrow().is_empty(),
            "{case}: {:?}",
            realm.gate.notes.borrow()
        );
    }
}

/// Outcome-deciding counterexamples: pinned promise machinery converts an uncatchable
/// error into a rejection (quickjs.c js_promise_constructor; resolve-function `then`
/// lookup at 55801, reached by `await` through the internal %Promise% at 21516).
const SWALLOW: &[&str] = &[
    "new Promise(() => { STOP; });",
    "new Promise(resolve => resolve({ get then() { STOP; } }));",
    "Promise.resolve({ get then() { STOP; } });",
    "(async () => { await { get then() { STOP; } }; })();",
    "Promise.all({ [Symbol.iterator]() { STOP; } });",
];

#[test]
fn promise_machinery_swallows_gate_and_deadline_stops_in_process() {
    const N: usize = 1000;
    for shape in SWALLOW {
        // The facade's uncatchable stop alone, isolated from latched polling: it becomes a
        // rejection, synchronous source continues and a reaction receives the stop object.
        let realm = hardened_realm(L_MAX);
        realm.gate.quiet_latch.set(true);
        let body = shape.replace("STOP", "new RegExp(P)");
        let out = run(
            &realm,
            &format!(
                "{REPRODUCER}\nconst p = (() => {{ {body} }})(); note('continued after stop');
                 new Promise(() => {{ new RegExp(P); }}).catch(e => note('reaction got ' + e.message));"
            ),
        );
        assert!(out.ok, "{shape}: expected normal completion after the stop");
        assert_eq!(out.latch, Some(Reason::ResourceLimit));
        assert_eq!(out.compiles, 0);
        assert_eq!(out.notes, ["continued after stop"]);
        for _ in 0..4 {
            drop(realm.rt.execute_pending_job());
        }
        assert_eq!(
            *realm.gate.notes.borrow(),
            ["continued after stop", "reaction got host stop"]
        );

        // Production configuration: latched polls interrupt, and each interrupt raised
        // inside the same machinery is swallowed too, so source runs on indefinitely.
        let realm = hardened_realm(L_MAX);
        let body = shape.replace("STOP", "expire(); for (;;) {}");
        let arm = realm.gate.clone();
        realm.context.with(|ctx| {
            let f = Function::new(ctx.clone(), move || arm.pending.set(Some(Reason::Timeout)));
            ctx.globals().set("expire", f.unwrap()).unwrap();
        });
        let out = run(
            &realm,
            &format!("for (let i = 0; i < {N}; i++) {{ {body} }} note('completed');"),
        );
        println!(
            "{shape}: gate and deadline stops swallowed {N} times; deadline callbacks={}",
            realm.gate.callbacks.get()
        );
        assert!(out.ok, "{shape}: deadline interrupt was not swallowed");
        assert_eq!(out.latch, Some(Reason::Timeout));
        assert!(realm.gate.callbacks.get() >= N);
        assert_eq!(out.notes, ["completed"]);
    }
}

#[test]
fn await_resolution_ignores_the_global_promise_binding() {
    let realm = hardened_realm(L_MAX);
    let out = run(
        &realm,
        "globalThis.Promise = undefined;
         (async () => { await { get then() { note('then read synchronously'); } }; })();
         note('after');",
    );
    assert!(out.ok);
    assert_eq!(out.notes, ["then read synchronously", "after"]);
}

#[test]
fn gate_reads_deadline_and_cancellation_before_every_admitted_compile() {
    let realm = hardened_realm(L_MAX);
    // Two checks per construction (entry, admission): the seventh is the fourth construction's entry.
    realm.gate.expire_at_entry.set(Some(7));
    let out = run(
        &realm,
        "try { for (let i = 0; ; i++) new RegExp('a' + (i % 10)); } finally { note('finally'); }",
    );
    assert!(!out.ok && out.uncatchable);
    assert_eq!(out.latch, Some(Reason::Timeout));
    assert_eq!(out.compiles, 3);
    assert!(out.notes.is_empty());

    let realm = hardened_realm(L_MAX);
    realm.gate.pending.set(Some(Reason::Cancelled));
    let out = run(
        &realm,
        "try { new RegExp('a'); } finally { note('finally'); }",
    );
    assert!(!out.ok && out.uncatchable);
    assert_eq!(out.latch, Some(Reason::Cancelled));
    assert_eq!(out.compiles, 0);
}

fn literal_module(body: &str, flags: &str) -> String {
    format!("export const r = /{body}/{flags};\n")
}

#[test]
fn literal_preflight_bounds_before_boa_and_quickjs() {
    let at = "a".repeat(L_MAX);
    assert!(preflight::preflight(&literal_module(&at, "g"), L_MAX).is_ok());
    let over = "a".repeat(L_MAX + 1);
    assert_eq!(
        preflight::preflight(&literal_module(&over, ""), L_MAX),
        Err(Rejection::LiteralBound { units: L_MAX + 1 })
    );
    // UTF-16 units, not bytes: U+1F600 counts two, U+00E9 one.
    let astral = "\u{1F600}".repeat(L_MAX / 2);
    assert!(preflight::preflight(&literal_module(&astral, "u"), L_MAX).is_ok());
    let wide = format!("{astral}\u{e9}");
    assert!(matches!(
        preflight::preflight(&literal_module(&wide, "u"), L_MAX),
        Err(Rejection::LiteralBound { .. })
    ));
    // Preserved reproducer as a literal: rejected by the raw token measure alone.
    let reproducer = forward_reference_pattern(64_000);
    let start = Instant::now();
    let result = preflight::preflight(&literal_module(&reproducer, ""), L_MAX);
    let elapsed = start.elapsed();
    println!("64,000-reference literal preflight rejection: {elapsed:?}");
    assert_eq!(result, Err(Rejection::LiteralBound { units: 320_007 }));
    assert!(result.unwrap_err().is_resource());
    assert!(elapsed < Duration::from_millis(500));
    let big = format!("// {}\n", "x".repeat(preflight::MODULE_BYTES));
    assert!(matches!(
        preflight::preflight(&big, L_MAX),
        Err(Rejection::ModuleBytes(_))
    ));
}

#[test]
fn literal_preflight_agreement_is_exact_and_fail_closed() {
    let fixtures: &[(&str, &[&str])] = &[
        (
            "let a = 1, b = 1, g = 1; export const v = [4 /2/ 1, a /(b)/ g, ({}) / 2];",
            &[],
        ),
        ("let x = 4; x /= 2; export const v = x /2/ 1;", &[]),
        ("{} /y+/g.test('y'); export {};", &["y+"]),
        ("let x = 1; if (x) /y+/g.test(''); export {};", &["y+"]),
        (
            "export const v = [/=a/, /\\//, /[/]/, /\\u{61}\\u0062/u, /x/dgimsuy];",
            &["=a", "\\/", "[/]", "\\u{61}\\u0062", "x"],
        ),
        (
            "// /x/\n/* /y/ */ export const v = ['/z/', `/w/${/q/.source}`];",
            &["q"],
        ),
        (
            "export function f() { if (false) { return () => /n/; } } class C { static s = /s/; m() { return /m/; } } export { C };",
            &["n", "s", "m"],
        ),
    ];
    for (source, expected) in fixtures {
        let literals =
            preflight::preflight(source, L_MAX).unwrap_or_else(|e| panic!("{source}: {e:?}"));
        let bodies: Vec<&str> = literals.iter().map(|l| l.body.as_str()).collect();
        assert_eq!(bodies, *expected, "{source}");
    }
    let lit = |body: &str, flags: &str| Literal {
        body: body.into(),
        flags: flags.into(),
    };
    assert_eq!(
        preflight::agree(&[lit("a", "")], &[]),
        Err(Rejection::Disagreement)
    );
    assert_eq!(
        preflight::agree(&[lit("a", "")], &[lit("b", "")]),
        Err(Rejection::Disagreement)
    );
    assert_eq!(
        preflight::agree(&[lit("a", "g")], &[lit("a", "")]),
        Err(Rejection::Disagreement)
    );
    // Invalid pattern within the bound: raw Oxc accepts, Boa validation rejects, never declared.
    assert!(matches!(
        preflight::preflight("export const r = /(/;", L_MAX),
        Err(Rejection::BoaSyntax(_))
    ));
    assert_eq!(
        preflight::preflight("export const r = /unterminated", L_MAX),
        Err(Rejection::OxcSyntax)
    );
}

#[test]
fn every_module_in_the_static_graph_is_preflighted_before_declaration() {
    for (helper, admitted) in [
        (literal_module("a".repeat(L_MAX).as_str(), ""), true),
        (
            literal_module(&forward_reference_pattern(64_000), ""),
            false,
        ),
    ] {
        let realm = hardened_realm(L_MAX);
        let rejection = Rc::new(RefCell::new(None));
        let entry = "import { r } from './helper.js'; note('entry ran ' + r.source.length);";
        realm.rt.set_loader(
            GraphResolver(vec!["entry.js".into(), "helper.js".into()]),
            Graph {
                sources: HashMap::from([("helper.js".to_owned(), helper)]),
                bound: L_MAX,
                rejection: rejection.clone(),
            },
        );
        let start = Instant::now();
        let evaluated = realm.context.with(|ctx| {
            // Entry passes its own preflight; declaration then loads (and preflights) the helper.
            let declared = preflight::declare(&ctx, "entry.js", entry, L_MAX).unwrap();
            let result = declared
                .and_then(|m| m.eval())
                .and_then(|(_, p)| p.finish::<()>());
            drop(ctx.catch());
            result.is_ok()
        });
        let notes = realm.gate.notes.borrow().clone();
        if admitted {
            assert!(evaluated && rejection.borrow().is_none());
            assert_eq!(notes, [format!("entry ran {L_MAX}")]);
        } else {
            assert!(!evaluated && notes.is_empty());
            let (name, why) = rejection.borrow().clone().unwrap();
            assert_eq!(name, "helper.js");
            assert!(why.is_resource(), "{why:?}");
            assert!(start.elapsed() < Duration::from_millis(500));
        }
    }
}

/// Worst-case families at the candidate bound, each generated to at most `l` units.
fn families(l: usize) -> Vec<(&'static str, String)> {
    let fit = |unit: &str, tail: &str| unit.repeat((l - tail.len()) / unit.len()) + tail;
    let refs = 3 * l / 20;
    vec![
        ("forward references", fit("\\k<a>", "(?<a>x)")),
        (
            "references + 3-byte padding",
            format!(
                "{}{}(?<a>x)",
                "\\k<a>".repeat(refs),
                "\u{ff71}".repeat(l - 5 * refs - 7)
            ),
        ),
        ("alternation", fit("a|", "a")),
        ("lookbehind reversal", format!("(?<={})", "a".repeat(l - 5))),
        ("capture groups (255 max)", fit("(a)*", "")),
        ("quantified groups", fit("(?:a)*", "")),
        ("nested quantifiers, full length", {
            let d = (l - 1) / 5;
            format!("{}a{}", "(?:".repeat(d), ")*".repeat(d))
        }),
        ("nested quantifiers, depth 100", {
            let core = format!("{}a{}", "(?:".repeat(100), ")*".repeat(100));
            format!("{core}{}", "a".repeat(l - core.len()))
        }),
        ("property classes", fit("\\p{L}", "")),
        ("set operations", fit("[\\p{L}--\\p{N}]", "")),
        ("case-folded ranges", fit("[\\u0000-\\uffff]", "")),
    ]
}

fn compile_time(ctx: &Ctx<'_>, pattern: &str, flags: &str) -> (Duration, bool) {
    ctx.globals().set("p", pattern).unwrap();
    ctx.globals().set("f", flags).unwrap();
    let start = Instant::now();
    let ok = ctx.eval::<(), _>("new RegExp(p, f)").is_ok();
    let elapsed = start.elapsed();
    drop(ctx.catch());
    (elapsed, ok)
}

#[test]
fn calibrate_worst_case_families_and_flags_at_candidate_bound() {
    let rt = Runtime::new().unwrap();
    rt.set_memory_limit(HEAP);
    rt.set_max_stack_size(STACK);
    let context = Context::full(&rt).unwrap();
    let mut worst = (Duration::ZERO, "", "");
    context.with(|ctx| {
        for (l, label) in [(L_MAX, "L_MAX"), (2 * L_MAX, "2*L_MAX")] {
            for (family, pattern) in families(l) {
                assert!(pattern.encode_utf16().count() <= l, "{family}");
                let mut row = String::new();
                for flags in ["", "i", "u", "iu", "v", "iv"] {
                    let (t, ok) = compile_time(&ctx, &pattern, flags);
                    row += &format!(
                        " {flags:>2}={:>7.2}ms{}",
                        t.as_secs_f64() * 1e3,
                        if ok { "" } else { "!" }
                    );
                    if l == L_MAX && t > worst.0 {
                        worst = (t, family, flags);
                    }
                }
                println!("{label:>7} {family:<32}{row}");
            }
        }
    });
    println!(
        "worst at L_MAX={L_MAX}: {:?} ({}, flags {:?}); '!' = SyntaxError after compile work",
        worst.0, worst.1, worst.2
    );
    assert!(worst.0 < Duration::from_millis(250), "{worst:?}");
}

/// Non-stopping recording handler: longest wall-clock gap between interrupt callbacks.
fn poll_gaps(source: &str) -> (usize, Duration, Duration) {
    let rt = Runtime::new().unwrap();
    rt.set_memory_limit(64 * 1024 * 1024);
    rt.set_max_stack_size(STACK);
    let state = Rc::new(RefCell::new((0usize, Instant::now(), Duration::ZERO)));
    let handler = state.clone();
    rt.set_interrupt_handler(Some(Box::new(move || {
        let mut s = handler.borrow_mut();
        let now = Instant::now();
        let gap = now - s.1;
        s.0 += 1;
        s.1 = now;
        s.2 = s.2.max(gap);
        false
    })));
    let context = Context::full(&rt).unwrap();
    let start = Instant::now();
    state.borrow_mut().1 = start;
    context.with(|ctx| eval::<()>(&ctx, source));
    let s = state.borrow();
    let tail = s.1.elapsed();
    (s.0, s.2.max(tail), start.elapsed())
}

#[test]
fn matching_checkpoint_interval_audit() {
    let cases = [
        (
            "200k short global matches",
            "'a'.repeat(200000).replace(/a/g, '');".to_owned(),
        ),
        (
            "JS loop of short tests",
            "const r = /a/; for (let i = 0; i < 200000; i++) r.test('a');".to_owned(),
        ),
        (
            "catastrophic backtracking",
            "/(a+)+b/.test('a'.repeat(24));".to_owned(),
        ),
        (
            "long back-reference compares",
            "/^(a*)\\1*b/.test('a'.repeat(10000));".to_owned(),
        ),
        (
            "non-polling char run at L_MAX per start position",
            format!(
                "new RegExp('a'.repeat({}) + 'b').test('a'.repeat(60000));",
                L_MAX - 1
            ),
        ),
    ];
    for (label, source) in cases {
        let (callbacks, max_gap, total) = poll_gaps(&source);
        println!(
            "{label:<50} callbacks={callbacks:>6} max_gap={max_gap:>12.3?} total={total:>12.3?}"
        );
        assert!(callbacks > 0, "{label}: no poll reached");
    }
}
