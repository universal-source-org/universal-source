#![forbid(unsafe_code)]
use crate::{Boundary, Failure as F, Limits, Portable as P, Usage};
use rquickjs::{Array, Context, Ctx, Function, Object, Runtime, Value};
use std::{cell::Cell, rc::Rc};
const DYNAMIC: &str = include_str!("../../quickjs-dynamic-code-spike/src/harden.js");
const GLOBAL: &str = include_str!("../../quickjs-global-surface-spike/src/harden.js");
const INVENTORY: &str = include_str!("../../quickjs-global-surface-spike/src/inventory.js");
const PRISTINE: &str =
    include_str!("../../quickjs-global-surface-spike/src/pristine-inventory.json");
const GLOBALS: &str = include_str!("../../quickjs-global-surface-spike/src/allowed-globals.json");
const INTRINSICS: &str =
    include_str!("../../quickjs-global-surface-spike/src/allowed-intrinsics.json");
fn runtime() -> Runtime {
    let r = Runtime::new().unwrap();
    r.set_memory_limit(32 * 1024 * 1024);
    r.set_max_stack_size(512 * 1024);
    r
}
fn harden(ctx: &Ctx<'_>) {
    let a: String = ctx.eval(INVENTORY).unwrap();
    let e: String = ctx.eval(format!("JSON.stringify({PRISTINE})")).unwrap();
    assert_eq!(a, e);
    ctx.eval::<(), _>(DYNAMIC).unwrap();
    let f: Function = ctx.eval(GLOBAL).unwrap();
    let g: Value = ctx.eval(GLOBALS).unwrap();
    let i: Value = ctx.eval(format!("({INTRINSICS})")).unwrap();
    f.call::<_, ()>((g, i)).unwrap();
}
fn realm(f: impl for<'js> FnOnce(Ctx<'js>, Boundary<'js>)) {
    let rt = runtime();
    let c = Context::full(&rt).unwrap();
    c.with(|ctx| {
        let b = Boundary::new(&ctx, Limits::default()).unwrap();
        harden(&ctx);
        f(ctx, b)
    });
    assert!(!rt.is_job_pending());
}
fn out<'js>(ctx: &Ctx<'js>, b: &Boundary<'js>, s: &str) -> Result<(P, Usage), F> {
    let v: Value = ctx.eval(s).unwrap();
    b.outgoing(&v)
}
fn record(fields: Vec<(&str, P)>) -> P {
    P::Record(fields.into_iter().map(|(k, v)| (k.into(), v)).collect())
}
fn text(s: &str) -> P {
    P::Text(s.into())
}

#[test]
fn primitive_domain_and_zero_normalization() {
    realm(|ctx, b| {
        for (s, p) in [
            ("null", P::Null),
            ("true", P::Bool(true)),
            ("false", P::Bool(false)),
            ("0", P::Number(0.0)),
            ("-0", P::Number(0.0)),
            ("Number.MAX_VALUE", P::Number(f64::MAX)),
            ("Number.MIN_VALUE", P::Number(f64::from_bits(1))),
            ("9007199254740991", P::Number(9007199254740991.0)),
            ("-12.5", P::Number(-12.5)),
        ] {
            let copied = out(&ctx, &b, s).unwrap().0;
            assert_eq!(copied, p);
            if let P::Number(n) = copied
                && n == 0.0
            {
                assert!(!n.is_sign_negative());
            }
        }
        for s in [
            "undefined",
            "NaN",
            "Infinity",
            "-Infinity",
            "1n",
            "Symbol()",
            "(()=>1)",
            "(async()=>1)",
        ] {
            assert_eq!(out(&ctx, &b, s), Err(F::Invalid), "{s}");
        }
    });
}

#[test]
fn exact_unicode_values_and_keys_including_nul() {
    realm(|ctx, b| {
        for (s, expected) in [
            (r#"''"#, ""),
            (r#"' ASCII '"#, " ASCII "),
            (r#"'é中'"#, "é中"),
            (r#"'\uD83D\uDE00'"#, "😀"),
            (r#"'a中\uD83D\uDE00z'"#, "a中😀z"),
            (r#"'e\u0301'"#, "e\u{301}"),
            (r#"'a\0b'"#, "a\0b"),
        ] {
            assert_eq!(out(&ctx, &b, s).unwrap().0, text(expected));
        }
        let s = r#"JSON.parse('{"__proto__":1,"prototype":2,"constructor":3,"toString":4,"valueOf":5,"hasOwnProperty":6,"0":7,"01":8,"":9,"😀":10,"a\\u0000b":11}')"#;
        let copied = out(&ctx, &b, s).unwrap().0;
        let P::Record(fields) = copied else { panic!() };
        assert_eq!(fields.len(), 11);
        assert_eq!(fields["a\0b"], P::Number(11.0));
        assert_eq!(fields["😀"], P::Number(10.0));
        assert_eq!(fields["__proto__"], P::Number(1.0));
        // JSON.parse above is fixture creation only; converter never calls JSON.
    });
}

#[test]
fn invalid_surrogate_values_and_keys_are_not_repaired() {
    realm(|ctx, b| {
        for units in [
            r"\uD800",
            r"\uDC00",
            r"\uD800x",
            r"\uDC00\uD800",
            r"\uD800\uD800",
            r"\uD800\uDC00\uDC00",
        ] {
            for source in [format!("'{units}'"), format!("({{['{units}']:1}})")] {
                assert_eq!(out(&ctx, &b, &source), Err(F::Invalid), "{source}");
            }
        }
        // Rope strings must preserve the same UTF16 semantics, not just flat strings.
        assert_eq!(out(&ctx, &b, "'x'.repeat(100)+'\\uD800'"), Err(F::Invalid));
    });
}

#[test]
fn record_descriptor_rules_and_structural_class_instances() {
    realm(|ctx, b| {
        for s in [
            "Object.assign(Object.create(null),{x:1})",
            "Object.freeze({x:1})",
            "Object.setPrototypeOf(new (class {#hidden=9;x=1;method(){throw 1}})(),Object.prototype)",
            "Object.setPrototypeOf(new (class {#hidden=9;x=1})(),null)",
        ] {
            assert_eq!(
                out(&ctx, &b, s).unwrap().0,
                record(vec![("x", P::Number(1.0))])
            );
        }
        for s in [
            "new (class{x=1})()",
            "Object.create({})",
            "({get x(){throw 1}})",
            "({set x(v){throw 1}})",
            "Object.defineProperty({},'x',{value:1})",
            "({[Symbol()]:1})",
        ] {
            assert_eq!(out(&ctx, &b, s), Err(F::Invalid), "{s}");
        }
    });
}

#[test]
fn dense_arrays_length_rules_and_impostors() {
    realm(|ctx, b| {
        for s in [
            "[1,null,true]",
            "Object.freeze([1,null,true])",
            "Object.defineProperty([1,null,true],'0',{enumerable:false})",
        ] {
            assert_eq!(
                out(&ctx, &b, s).unwrap().0,
                P::Array(vec![P::Number(1.0), P::Null, P::Bool(true)])
            );
        }
        for s in [
            "[,]",
            "[1,,3]",
            "(()=>{const a=[1];delete a[0];return a})()",
            "Object.defineProperty([1],'0',{get(){throw 1}})",
            "Object.assign([],{x:1})",
            "Object.assign([],{['01']:1})",
            "Object.defineProperty([],'x',{value:1})",
            "Object.assign([],{[Symbol()]:1})",
            "Object.setPrototypeOf([],null)",
            "new (class extends Array{})()",
            "Object.create(Array.prototype)",
            "Object.setPrototypeOf(new Map(),Array.prototype)",
            "Object.assign(new Array(2),{0:1,x:2})",
        ] {
            assert_eq!(out(&ctx, &b, s), Err(F::Invalid), "{s}");
        }
        // Plain numeric-key/length records remain records; not coerced into arrays.
        assert_eq!(
            out(&ctx, &b, "({0:1,length:1})").unwrap().0,
            record(vec![("0", P::Number(1.0)), ("length", P::Number(1.0))])
        );
    });
}

#[test]
fn native_brands_rejected_after_prototype_and_tag_mutation() {
    realm(|ctx, b| {
        for s in [
            "new Map()",
            "new Set()",
            "new WeakMap()",
            "new WeakSet()",
            "/x/",
            "Promise.resolve(1)",
            "new Error()",
            "new TypeError()",
            "new EvalError()",
            "new RangeError()",
            "new ReferenceError()",
            "new SyntaxError()",
            "new URIError()",
            "new AggregateError([])",
            "new (class extends Error{})()",
            "new Boolean(false)",
            "new Number(1)",
            "new String('x')",
            "Object(1n)",
            "Object(Symbol())",
            "[1].values()",
            "(function*(){})()",
        ] {
            for proto in ["Object.prototype", "null"] {
                let source = format!(
                    "(()=>{{let v=({s});Object.setPrototypeOf(v,{proto});Object.defineProperty(v,'constructor',{{value:Object,configurable:true}});Object.defineProperty(v,Symbol.toStringTag,{{value:'Object',configurable:true}});v.x=1;return v}})()"
                );
                assert_eq!(out(&ctx, &b, &source), Err(F::Invalid), "{s}");
            }
        }
        let calls = Rc::new(Cell::new(0));
        let c = calls.clone();
        let native = Function::new(ctx.clone(), move || c.set(c.get() + 1)).unwrap();
        assert_eq!(b.outgoing(native.as_value()), Err(F::Invalid));
        assert_eq!(calls.get(), 0);
    });
}

#[test]
fn removed_native_intrinsics_remain_rejected_when_privately_injected() {
    let rt = runtime();
    let c = Context::full(&rt).unwrap();
    c.with(|ctx| {
        let b = Boundary::new(&ctx, Limits::default()).unwrap();
        let sources = [
            "new Date(0)",
            "new ArrayBuffer(4)",
            "new SharedArrayBuffer(4)",
            "new DataView(new ArrayBuffer(4))",
            "new Int8Array(1)",
            "new Uint8Array(1)",
            "new Uint8ClampedArray(1)",
            "new Int16Array(1)",
            "new Uint16Array(1)",
            "new Int32Array(1)",
            "new Uint32Array(1)",
            "new Float16Array(1)",
            "new Float32Array(1)",
            "new Float64Array(1)",
            "new BigInt64Array(1)",
            "new BigUint64Array(1)",
        ];
        let values: Vec<Value> = sources.iter().map(|s| ctx.eval(*s).unwrap()).collect();
        harden(&ctx);
        let mutate: Function = ctx
            .eval("v=>{Object.setPrototypeOf(v,Object.prototype);v.x=1;return v}")
            .unwrap();
        for v in values {
            assert_eq!(b.outgoing(&v), Err(F::Invalid));
            let v: Value = mutate.call((v,)).unwrap();
            assert_eq!(b.outgoing(&v), Err(F::Invalid));
        }
    });
}

#[test]
fn proxy_traps_never_execute_in_complete_outgoing_path() {
    let rt = runtime();
    let c = Context::full(&rt).unwrap();
    c.with(|ctx| {
        let b = Boundary::new(&ctx, Limits::default()).unwrap();
        let proxy: rquickjs::function::Constructor = ctx.globals().get("Proxy").unwrap();
        let rev: Function = proxy.get("revocable").unwrap();
        harden(&ctx);
        let count = Rc::new(Cell::new(0));
        let n = count.clone();
        let mark = Function::new(ctx.clone(), move || n.set(n.get() + 1)).unwrap();
        let make: Function = ctx
            .eval(
                r#"mark => {
            const trap = () => { mark(); throw 1 };
            return {get:trap, has:trap, getPrototypeOf:trap, ownKeys:trap,
                getOwnPropertyDescriptor:trap, apply:trap, construct:trap};
        }"#,
            )
            .unwrap();
        let handler: Value = make.call((mark,)).unwrap();
        for s in ["({})", "[]", "new Map()", "(()=>1)"] {
            let target: Value = ctx.eval(s).unwrap();
            let v: Value = proxy.construct((target.clone(), handler.clone())).unwrap();
            assert_eq!(b.outgoing(&v), Err(F::Invalid));
            let nested: Value = proxy.construct((v, handler.clone())).unwrap();
            assert_eq!(b.outgoing(&nested), Err(F::Invalid));
            let pair: Object = rev.call((target, handler.clone())).unwrap();
            let v: Value = pair.get("proxy").unwrap();
            let revoke: Function = pair.get("revoke").unwrap();
            revoke.call::<_, ()>(()).unwrap();
            assert_eq!(b.outgoing(&v), Err(F::Invalid));
        }
        assert_eq!(count.get(), 0);
        // Negative control: native enumeration BEFORE the class gate calls ownKeys.
        let v: Object = proxy
            .construct((Object::new(ctx.clone()).unwrap(), handler))
            .unwrap();
        assert!(
            v.own_keys::<String>(rquickjs::object::Filter::new().string())
                .next()
                .unwrap()
                .is_err()
        );
        let _ = ctx.catch();
        assert_eq!(count.get(), 1);
    });
}

#[test]
fn helper_identity_exclusion_survives_erased_markers_and_nested_returns() {
    realm(|ctx, mut b| {
        let context = Object::new(ctx.clone()).unwrap();
        let registry = Object::new_proto(ctx.clone(), None).unwrap();
        let wrapper = Object::new(ctx.clone()).unwrap();
        let nested = Object::new(ctx.clone()).unwrap();
        registry.set("wrapper", wrapper.clone()).unwrap();
        context.set("services", registry.clone()).unwrap();
        wrapper.set("nested", nested.clone()).unwrap();
        for helper in [&context, &registry, &wrapper, &nested] {
            b.exclude_helper(helper.as_value()).unwrap();
        }
        let erase:Function=ctx.eval("v=>{for(const k of Reflect.ownKeys(v))delete v[k];Object.setPrototypeOf(v,null);v.x=1;return {helper:v}}").unwrap();
        for helper in [&context, &registry, &wrapper, &nested] {
            let returned: Value = erase.call((helper.clone(),)).unwrap();
            assert_eq!(b.outgoing(helper.as_value()), Err(F::Invalid));
            assert_eq!(b.outgoing(&returned), Err(F::Invalid));
        }
        let data = Object::new(ctx.clone()).unwrap();
        data.set("x", 1).unwrap();
        assert_eq!(
            b.outgoing(data.as_value()).unwrap().0,
            record(vec![("x", P::Number(1.0))])
        );
        assert_eq!(
            out(&ctx, &b, "({x:1})").unwrap().0,
            record(vec![("x", P::Number(1.0))])
        );
        // A NEW source record copying harmless helper data has no retained authority.
        let copy: Function = ctx.eval("v=>({x:v.x})").unwrap();
        let data: Value = copy.call((wrapper,)).unwrap();
        assert!(b.outgoing(&data).is_ok());
    });
}

#[test]
fn ordinary_intrinsic_identities_cannot_be_flattened_into_records() {
    for name in [
        "Math",
        "JSON",
        "Reflect",
        "Object.prototype",
        "globalThis",
        "Object.getPrototypeOf([][Symbol.iterator]())",
        "Object.getPrototypeOf(''[Symbol.iterator]())",
        "Object.getPrototypeOf(new Map().entries())",
        "Object.getPrototypeOf(new Set().values())",
        "Object.getPrototypeOf(''.matchAll(/x/g))",
        "Object.getPrototypeOf(Object.getPrototypeOf((function*(){})()))",
        "Object.getPrototypeOf(Object.getPrototypeOf((async function*(){})()))",
    ] {
        realm(|ctx, b| {
            let s = format!(
                "(()=>{{const v={name}, keys=Reflect.ownKeys, desc=Object.getOwnPropertyDescriptor, proto=Object.setPrototypeOf; for(const k of keys(v)){{if(desc(v,k).configurable)delete v[k]}}proto(v,null);return v}})()"
            );
            assert_eq!(out(&ctx, &b, &s), Err(F::Invalid), "{name}");
            // Fresh source data is eligible despite looking like an erased intrinsic.
            assert_eq!(out(&ctx, &b, "({})").unwrap().0, record(vec![]));
        });
    }
}

#[test]
fn hostile_hooks_and_poisoned_reflection_do_not_execute() {
    realm(|ctx, b| {
        let count = Rc::new(Cell::new(0));
        let n = count.clone();
        let mark = Function::new(ctx.clone(), move || n.set(n.get() + 1)).unwrap();
        let make:Function=ctx.eval(r#"mark=>{
        const die=()=>{mark();throw new Error('hook')};const list=[];
        for(const key of ['x','constructor','__proto__','toJSON','toString','valueOf','then',Symbol.toPrimitive,Symbol.iterator,Symbol.asyncIterator,Symbol.toStringTag,Symbol.species]){
            const v={};Object.defineProperty(v,key,{get:die,set:die,enumerable:true});list.push(v);
        }
        list.push({toJSON:die},{valueOf:die},{toString:die},{then:die});
        const p=Promise.resolve(1);Object.defineProperty(p,'then',{get:die});Object.defineProperty(p,'constructor',{get:die});list.push(p);
        const proto={};Object.defineProperty(proto,'x',{get:die});list.push(Object.create(proto));
        Object.getOwnPropertyDescriptor=die;Object.getOwnPropertyDescriptors=die;Object.keys=die;Reflect.ownKeys=die;
        return list;
    }"#).unwrap();
        let values: Array = make.call((mark,)).unwrap();
        for i in 0..values.len() {
            let v: Value = values.get(i).unwrap();
            assert_eq!(b.outgoing(&v), Err(F::Invalid));
        }
        assert_eq!(count.get(), 0);
        assert_eq!(
            out(&ctx, &b, "({safe:[1,'x']})").unwrap().0,
            record(vec![("safe", P::Array(vec![P::Number(1.0), text("x")]))])
        );
        assert_eq!(count.get(), 0);
    });
}

#[test]
fn naive_serialization_iteration_coercion_and_reads_execute_negative_controls() {
    realm(|ctx, b| {
        let count = Rc::new(Cell::new(0));
        let n = count.clone();
        let mark = Function::new(ctx.clone(), move || n.set(n.get() + 1)).unwrap();
        let make:Function=ctx.eval("mark=>({get x(){mark();return 1},toJSON(){mark();return 1},valueOf(){mark();return 1},[Symbol.iterator](){mark();return [][Symbol.iterator]()}})").unwrap();
        let v: Value = make.call((mark,)).unwrap();
        assert_eq!(b.outgoing(&v), Err(F::Invalid));
        assert_eq!(count.get(), 0);
        for s in ["v=>v.x", "v=>JSON.stringify(v)", "v=>+v", "v=>[...v]"] {
            let f: Function = ctx.eval(s).unwrap();
            let _: Value = f.call((v.clone(),)).unwrap();
        }
        assert_eq!(count.get(), 4);
    });
}

#[test]
fn cycles_fail_but_repeated_references_expand_independently() {
    realm(|ctx, b| {
        for s in [
            "(()=>{let a={};a.a=a;return a})()",
            "(()=>{let a={},b={a};a.b=b;return a})()",
            "(()=>{let a=[];a.push(a);return a})()",
            "(()=>{let a={};a.b=[a];return a})()",
            "(()=>{let a={},b=[{c:[a]}];a.b=b;return a})()",
        ] {
            assert_eq!(out(&ctx, &b, s), Err(F::Invalid));
        }
        let (p, u) = out(&ctx, &b, "(()=>{let x={x:1};return {a:x,b:x}})()").unwrap();
        let P::Record(mut fields) = p else { panic!() };
        assert_eq!(fields["a"], fields["b"]);
        assert_eq!(u.nodes, 5);
        assert_eq!(u.text_bytes, 4);
        assert_eq!(u.expanded_bytes, 44);
        let P::Record(a) = fields.get_mut("a").unwrap() else {
            panic!()
        };
        a.insert("x".into(), P::Number(2.0));
        assert_eq!(fields["b"], record(vec![("x", P::Number(1.0))]));
    });
}

fn limited(l: Limits, source: &str, host: P, expected: Result<Usage, F>) {
    let rt = runtime();
    let c = Context::full(&rt).unwrap();
    c.with(|ctx| {
        let b = Boundary::new(&ctx, l).unwrap();
        harden(&ctx);
        assert_eq!(out(&ctx, &b, source).map(|(_, u)| u), expected);
        assert_eq!(b.incoming(&host).map(|(_, u)| u), expected);
    });
}
#[test]
fn exact_depth_and_node_bounds_both_directions() {
    let host = P::Array(vec![P::Array(vec![P::Null])]);
    let usage = Usage {
        nodes: 3,
        text_bytes: 0,
        expanded_bytes: 24,
    };
    limited(
        Limits {
            depth: 2,
            nodes: 3,
            ..Limits::default()
        },
        "[[null]]",
        host.clone(),
        Ok(usage),
    );
    limited(
        Limits {
            depth: 1,
            ..Limits::default()
        },
        "[[null]]",
        host.clone(),
        Err(F::ResourceLimit),
    );
    limited(
        Limits {
            nodes: 2,
            ..Limits::default()
        },
        "[[null]]",
        host,
        Err(F::ResourceLimit),
    );
}
#[test]
fn exact_text_key_piece_and_expanded_bounds_both_directions() {
    let host = record(vec![("😀", text("é"))]);
    let u = Usage {
        nodes: 2,
        text_bytes: 6,
        expanded_bytes: 22,
    };
    limited(
        Limits {
            piece_bytes: 4,
            text_bytes: 6,
            expanded_bytes: 22,
            ..Limits::default()
        },
        "({'😀':'é'})",
        host.clone(),
        Ok(u),
    );
    for l in [
        Limits {
            piece_bytes: 3,
            ..Limits::default()
        },
        Limits {
            text_bytes: 5,
            ..Limits::default()
        },
        Limits {
            expanded_bytes: 21,
            ..Limits::default()
        },
    ] {
        limited(l, "({'😀':'é'})", host.clone(), Err(F::ResourceLimit));
    }
}
#[test]
fn shared_object_is_charged_each_time_not_once() {
    let rt = runtime();
    let c = Context::full(&rt).unwrap();
    c.with(|ctx| {
        let b = Boundary::new(
            &ctx,
            Limits {
                expanded_bytes: 200,
                ..Limits::default()
            },
        )
        .unwrap();
        harden(&ctx);
        assert!(out(&ctx, &b, "({x:'a'.repeat(64)})").is_ok());
        // Unique identities: root array + shared object only. Expanded data is 100 copies.
        assert_eq!(
            out(
                &ctx,
                &b,
                "(()=>{const x={x:'a'.repeat(64)};return new Array(100).fill(x)})()"
            ),
            Err(F::ResourceLimit)
        );
    });
}

#[test]
fn incoming_roundtrip_and_scalar_order_vs_integer_enumeration() {
    realm(|ctx, b| {
        let host = record(vec![
            ("😀", text("a中😀")),
            ("\u{e000}", P::Null),
            ("2", P::Number(f64::MAX)),
            ("10", P::Number(f64::from_bits(1))),
            ("01", P::Bool(true)),
            (
                "",
                P::Array(vec![
                    record(vec![("x", P::Number(1.0))]),
                    record(vec![("x", P::Number(1.0))]),
                ]),
            ),
            ("__proto__", text("plain")),
            ("constructor", P::Null),
            ("toString", text("own")),
            ("a\0b", text("e\u{301}")),
        ]);
        let P::Record(fields) = &host else { panic!() };
        let insertion: Vec<_> = fields.keys().cloned().collect();
        assert_eq!(&insertion[..4], ["", "01", "10", "2"]);
        assert!(
            insertion.iter().position(|s| s == "\u{e000}").unwrap()
                < insertion.iter().position(|s| s == "😀").unwrap()
        );
        let (v, in_u) = b.incoming(&host).unwrap();
        let (out, out_u) = b.outgoing(&v).unwrap();
        assert_eq!(out, host);
        assert_eq!(in_u, out_u);
        let keys: Function = ctx.eval("v=>Object.keys(v).join('|')").unwrap();
        let actual: String = keys.call((v.clone(),)).unwrap();
        assert!(actual.starts_with("2|10||01|"));
        let inspect:Function=ctx.eval("v=>Object.getPrototypeOf(v)===Object.prototype && Object.getOwnPropertyDescriptor(v,'__proto__').value==='plain'").unwrap();
        assert!(inspect.call::<_, bool>((v,)).unwrap());
    });
}

#[test]
fn incoming_uses_own_definitions_despite_poisoned_globals_and_setters() {
    realm(|ctx, b| {
        let count = Rc::new(Cell::new(0));
        let n = count.clone();
        let mark = Function::new(ctx.clone(), move || n.set(n.get() + 1)).unwrap();
        let poison:Function=ctx.eval(r#"mark=>{const die=()=>{mark();throw 1};
        for(const name of ['__proto__','constructor','prototype','toString','valueOf','hasOwnProperty','x'])Object.defineProperty(Object.prototype,name,{get:die,set:die,configurable:true});
        Object.defineProperty(Array.prototype,'0',{get:die,set:die,configurable:true});
        Object.defineProperty(Object.prototype,Symbol.iterator,{get:die,configurable:true});
        Object.getOwnPropertyDescriptor=die;Object.defineProperty=die;Reflect.ownKeys=die;Reflect.defineProperty=die;
        globalThis.Object=die;globalThis.Array=die;globalThis.Reflect=die;
    }"#).unwrap();
        poison.call::<_, ()>((mark,)).unwrap();
        let host = record(vec![
            ("__proto__", P::Null),
            ("constructor", P::Bool(false)),
            ("prototype", P::Number(1.0)),
            ("toString", text("ok")),
            ("x", P::Array(vec![record(vec![("x", text("😀"))])])),
        ]);
        let (v, u) = b.incoming(&host).unwrap();
        assert_eq!(b.outgoing(&v).unwrap(), (host, u));
        assert_eq!(count.get(), 0);
    });
}

#[test]
fn incoming_rejects_invalid_host_number_before_constructing_and_normalizes_zero() {
    realm(|_ctx, b| {
        for n in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(matches!(
                b.incoming(&P::Array(vec![P::Null, P::Number(n)])),
                Err(F::Invalid)
            ));
        }
        let (v, _) = b.incoming(&P::Number(-0.0)).unwrap();
        assert_eq!(v.as_number().unwrap().to_bits(), 0f64.to_bits());
        // Rust String/BTreeMap/owned tree make invalid UTF8/surrogates, duplicate keys,
        // JS functions and cyclic host references unrepresentable in this safe input type.
    });
}

#[test]
fn failure_never_publishes_partial_copy_and_fresh_realm_is_independent() {
    for _ in 0..2 {
        realm(|ctx, b| {
            let published = Rc::new(Cell::new(0));
            for s in [
                "({first:1,last:undefined})",
                "(()=>{let a={};a.self=a;return a})()",
            ] {
                if b.outgoing(&ctx.eval::<Value, _>(s).unwrap()).is_ok() {
                    published.set(published.get() + 1);
                }
            }
            assert_eq!(published.get(), 0);
            assert_eq!(
                out(&ctx, &b, "({ok:1})").unwrap().0,
                record(vec![("ok", P::Number(1.0))])
            );
            ctx.eval::<(), _>("globalThis.Object=0;Array.prototype.poison=1")
                .unwrap();
        });
    }
}
