#![forbid(unsafe_code)]
use crate::{ClassBridge, ForeignContext, ObjectClass as C};
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
    let rt = Runtime::new().unwrap();
    rt.set_memory_limit(16 * 1024 * 1024);
    rt.set_max_stack_size(256 * 1024);
    rt
}
fn harden(ctx: &Ctx<'_>) {
    let actual: String = ctx.eval(INVENTORY).unwrap();
    let expected: String = ctx.eval(format!("JSON.stringify({PRISTINE})")).unwrap();
    assert_eq!(actual, expected);
    ctx.eval::<(), _>(DYNAMIC).unwrap();
    let f: Function = ctx.eval(GLOBAL).unwrap();
    let globals: Value = ctx.eval(GLOBALS).unwrap();
    let intrinsics: Value = ctx.eval(format!("({INTRINSICS})")).unwrap();
    f.call::<_, ()>((globals, intrinsics)).unwrap();
}
fn check(ctx: &Ctx<'_>, bridge: &ClassBridge<'_>, source: &str, expected: C) {
    let v: Value = ctx.eval(source).unwrap();
    assert_eq!(bridge.classify(&v).unwrap(), expected, "{source}");
}

#[test]
fn ordinary_class_is_not_complete_record_validation() {
    let rt = runtime();
    let context = Context::full(&rt).unwrap();
    context.with(|ctx| {
        let b = ClassBridge::new(&ctx).unwrap();
        harden(&ctx);
        for s in [
            "({})",
            "Object.create(null)",
            "new (class {#private=1; value=2})()",
            "Object.setPrototypeOf(new (class {#private=1; value=2})(),Object.prototype)",
            "({get x(){throw 1}})",
            "Object.create({custom:true})",
        ] {
            check(&ctx, &b, s, C::OrdinaryObject);
        }
        // Accessor/custom prototype still require rejection by the future converter.
        for s in ["null", "undefined", "true", "1", "'x'", "1n", "Symbol()"] {
            check(&ctx, &b, s, C::NonObject);
        }
    });
}

#[test]
fn arrays_and_impostors_have_distinct_engine_classes() {
    let rt = runtime();
    let context = Context::full(&rt).unwrap();
    context.with(|ctx| {
        let b = ClassBridge::new(&ctx).unwrap();
        harden(&ctx);
        for s in [
            "[]",
            "[1,2]",
            "new (class extends Array {})()",
            "Object.setPrototypeOf([],null)",
            "new Array(2)",
        ] {
            check(&ctx, &b, s, C::Array);
        }
        for s in ["Object.create(Array.prototype)", "({0:'x',length:1})"] {
            check(&ctx, &b, s, C::OrdinaryObject);
        }
        check(
            &ctx,
            &b,
            "Object.setPrototypeOf(new Map(),Array.prototype)",
            C::OtherObject,
        );
        // Holes/subclass prototypes are shape checks, intentionally not class checks.
    });
}

#[test]
fn branded_matrix_stays_rejected_under_every_tested_prototype() {
    let rt = runtime();
    let context = Context::full(&rt).unwrap();
    context.with(|ctx| {
        let b=ClassBridge::new(&ctx).unwrap(); harden(&ctx);
        let fixtures=["new Map()","new Set()","/x/","Promise.resolve(1)","new Error('x')","new TypeError('x')","new EvalError()","new RangeError()","new ReferenceError()","new SyntaxError()","new URIError()","new AggregateError([])","new (class extends Error {})()","new String('x')","new Number(1)","new Boolean(false)","Object(1n)","Object(Symbol())","new WeakMap()","new WeakSet()","function f(){}","(()=>1)","(function*(){})()","[1].values()","new Map().entries()","'x'[Symbol.iterator]()"];
        for s in fixtures {
            for proto in ["Object.prototype","null","Array.prototype"] {
                let fixture=format!("(() => {{const v=({s});Object.setPrototypeOf(v,{proto});Object.defineProperty(v,'constructor',{{value:Object,configurable:true}});Object.defineProperty(v,Symbol.toStringTag,{{value:'Object',configurable:true}});v.record=42;return v;}})()");
                check(&ctx,&b,&fixture,C::OtherObject);
            }
        }
    });
}

#[test]
fn private_removed_intrinsics_and_unlisted_native_classes_fail_closed() {
    let rt = runtime();
    let context = Context::full(&rt).unwrap();
    context.with(|ctx| {
        let b=ClassBridge::new(&ctx).unwrap();
        // Trusted pristine construction only; no package sees these constructors.
        let fixtures=["new Date(0)","new ArrayBuffer(4)","new SharedArrayBuffer(4)","new DataView(new ArrayBuffer(4))","new Int8Array(1)","new Uint8Array(1)","new Uint8ClampedArray(1)","new Int16Array(1)","new Uint16Array(1)","new Int32Array(1)","new Uint32Array(1)","new Float16Array(1)","new Float32Array(1)","new Float64Array(1)","new BigInt64Array(1)","new BigUint64Array(1)","new WeakRef({})","new FinalizationRegistry(()=>{})","new DisposableStack()","new AsyncDisposableStack()"];
        let candidates:Vec<Value>=fixtures.iter().map(|s|ctx.eval(*s).unwrap()).collect();
        harden(&ctx);
        let mutate:Function=ctx.eval("(v,p)=>{Object.setPrototypeOf(v,p);Object.defineProperty(v,'constructor',{value:Object,configurable:true});v.record=1;return v;}").unwrap();
        let original=Object::new(ctx.clone()).unwrap().get_prototype().unwrap();
        for (s,v) in fixtures.iter().zip(candidates) {
            assert_eq!(b.classify(&v).unwrap(),C::OtherObject,"{s}");
            let v:Value=mutate.call((v,original.clone())).unwrap();
            assert_eq!(b.classify(&v).unwrap(),C::OtherObject,"mutated {s}");
            let v:Value=mutate.call((v,Value::new_null(ctx.clone()))).unwrap();
            assert_eq!(b.classify(&v).unwrap(),C::OtherObject,"null {s}");
        }
        assert!(ctx.eval::<bool,_>("typeof Date==='undefined' && typeof ArrayBuffer==='undefined' && typeof Proxy==='undefined'").unwrap());
    });
}

#[test]
fn proxy_and_revoked_proxy_classification_never_dispatches_traps() {
    let rt = runtime();
    let context = Context::full(&rt).unwrap();
    context.with(|ctx| {
        let b=ClassBridge::new(&ctx).unwrap();
        let proxy:rquickjs::function::Constructor=ctx.globals().get("Proxy").unwrap();
        let revoke_factory:Function=proxy.get("revocable").unwrap();
        harden(&ctx);
        let count=Rc::new(Cell::new(0));let n=count.clone();
        let mark=Function::new(ctx.clone(),move || n.set(n.get()+1)).unwrap();
        let make:Function=ctx.eval("mark=>{const trap=()=>{mark();throw new Error('TRAP');};return {get:trap,getOwnPropertyDescriptor:trap,ownKeys:trap,getPrototypeOf:trap,has:trap,apply:trap,construct:trap};}").unwrap();
        let handler:Value=make.call((mark,)).unwrap();
        for s in ["({})","[]","new Map()","function f(){}"] {
            let target:Value=ctx.eval(format!("({s})")).unwrap();
            let v:Value=proxy.construct((target.clone(),handler.clone())).unwrap();
            assert_eq!(b.classify(&v).unwrap(),C::OtherObject);
            assert!(v.is_proxy()); // Independent supported predicate; not bridge logic.
            let pair:Object=revoke_factory.call((target,handler.clone())).unwrap();
            let revoked:Value=pair.get("proxy").unwrap();
            let revoke:Function=pair.get("revoke").unwrap();revoke.call::<_,()>(()).unwrap();
            assert_eq!(b.classify(&revoked).unwrap(),C::OtherObject);
            assert!(revoked.is_proxy());
        }
        assert_eq!(count.get(),0);
        // Negative control: prototype traversal on an unknown Proxy executes JS.
        let target=Object::new(ctx.clone()).unwrap();
        let v:Value=proxy.construct((target,handler)).unwrap();
        let traverse:Function=ctx.eval("v=>{try{Object.getPrototypeOf(v);return false;}catch{return true;}}").unwrap();
        assert!(traverse.call::<_,bool>((v,)).unwrap());assert_eq!(count.get(),1);
    });
}

#[test]
fn hostile_hooks_remain_unread_by_classification() {
    let rt = runtime();
    let context = Context::full(&rt).unwrap();
    context.with(|ctx| {
        let b=ClassBridge::new(&ctx).unwrap();harden(&ctx);
        let count=Rc::new(Cell::new(0));let n=count.clone();
        let mark=Function::new(ctx.clone(),move || n.set(n.get()+1)).unwrap();
        let make:Function=ctx.eval(r#"mark=>{
            const die=()=>{mark();throw new Error('HOOK');};
            const proto=Object.create(null);
            Object.defineProperty(proto,'inherited',{get:die});
            const o=Object.create(proto);
            for(const key of ['value','constructor','__proto__','toString','valueOf','toJSON','then',Symbol.toPrimitive,Symbol.toStringTag,Symbol.iterator,Symbol.species])
                Object.defineProperty(o,key,{get:die,set:die,configurable:true});
            const a=[];Object.defineProperty(a,'0',{get:die});
            const p=Promise.resolve(1);Object.defineProperty(p,'then',{get:die});Object.defineProperty(p,'constructor',{get:die});
            return [o,a,p];
        }"#).unwrap();
        let fixtures:Array=make.call((mark,)).unwrap();
        for (i,expected) in [C::OrdinaryObject,C::Array,C::OtherObject].into_iter().enumerate() {
            let v:Value=fixtures.get(i).unwrap();assert_eq!(b.classify(&v).unwrap(),expected);
        }
        assert_eq!(count.get(),0);
        // Negative controls explicitly separated from the bridge.
        let o:Value=fixtures.get(0).unwrap();
        for source in ["v=>v.constructor.name","v=>Object.prototype.toString.call(v)","v=>Reflect.get(v,'value')","v=>String(v)","v=>[...v]"] {
            let f:Function=ctx.eval(source).unwrap();assert!(f.call::<_,Value>((o.clone(),)).is_err());let _=ctx.catch();
        }
        assert_eq!(count.get(),5);
    });
}

#[test]
fn instanceof_and_constructor_controls_are_not_engine_brand_checks() {
    let rt = runtime();
    let context = Context::full(&rt).unwrap();
    context.with(|ctx| {
        let b=ClassBridge::new(&ctx).unwrap();harden(&ctx);
        let count=Rc::new(Cell::new(0));let n=count.clone();
        let mark=Function::new(ctx.clone(),move || n.set(n.get()+1)).unwrap();
        let f:Function=ctx.eval("mark=>{const c={[Symbol.hasInstance](){mark();return true;}};return ({}) instanceof c;}").unwrap();
        assert!(f.call::<_,bool>((mark,)).unwrap());assert_eq!(count.get(),1);
        assert!(ctx.eval::<bool,_>("Object.setPrototypeOf(new Map(),Object.prototype) instanceof Object").unwrap());
        check(&ctx,&b,"Object.setPrototypeOf(new Map(),Object.prototype)",C::OtherObject);
    });
}

#[test]
fn replaced_globals_and_poisoned_links_cannot_change_captured_ids() {
    let rt = runtime();
    let context = Context::full(&rt).unwrap();
    context.with(|ctx| {
        let b=ClassBridge::new(&ctx).unwrap();harden(&ctx);
        let values:Array=ctx.eval("[{},[],new Map(),Object.setPrototypeOf(new Set(),null)]").unwrap();
        ctx.eval::<(),_>(r#"const poison=()=>{throw new Error('poison');};
          Object.getPrototypeOf=poison;Object.getOwnPropertyDescriptor=poison;Reflect.ownKeys=poison;
          Object.defineProperty(Object.prototype,'constructor',{get:poison,configurable:true});
          Object.defineProperty(Array.prototype,'constructor',{get:poison,configurable:true});
          Object.defineProperty(Object.prototype,Symbol.toStringTag,{get:poison,configurable:true});
          globalThis.Object=poison;globalThis.Array=poison;globalThis.Map=poison;globalThis.Set=poison;globalThis.Reflect=poison;"#).unwrap();
        for (i,c) in [C::OrdinaryObject,C::Array,C::OtherObject,C::OtherObject].into_iter().enumerate() {
            let v:Value=values.get(i).unwrap();assert_eq!(b.classify(&v).unwrap(),c);
        }
        // Even safe native allocation does not consult poisoned constructors.
        let fresh=ClassBridge::new(&ctx).unwrap();
        assert_eq!(fresh.classify(Object::new(ctx.clone()).unwrap().as_value()).unwrap(),C::OrdinaryObject);
        assert!(ctx.eval::<bool,_>("(()=>{try{eval('1');return false;}catch(e){return e instanceof EvalError;}})()").unwrap());
    });
}

#[test]
fn native_host_closure_opaque_payload_is_not_an_ordinary_record() {
    let rt = runtime();
    let context = Context::full(&rt).unwrap();
    context.with(|ctx| {
        let b = ClassBridge::new(&ctx).unwrap();
        harden(&ctx);
        // Function::new uses rquickjs's registered callable Rust class. Its opaque
        // Rust payload has no JS properties and no unsafe fixture implementation.
        let payload = Rc::new(Cell::new(17));
        let p = payload.clone();
        let host = Function::new(ctx.clone(), move || {
            p.set(p.get() + 1);
            p.get()
        })
        .unwrap();
        assert_eq!(b.classify(host.as_value()).unwrap(), C::OtherObject);
        let mutate: Function = ctx
            .eval("v=>{Object.setPrototypeOf(v,null);v.record=1;return v;}")
            .unwrap();
        let changed: Value = mutate.call((host,)).unwrap();
        assert_eq!(b.classify(&changed).unwrap(), C::OtherObject);
        assert_eq!(payload.get(), 17);
        // A host-authored plain wrapper has no engine brand. Classification must
        // not pretend to infer provenance; future converter needs a host registry.
        let wrapper = Object::new_proto(ctx.clone(), None).unwrap();
        assert_eq!(b.classify(wrapper.as_value()).unwrap(), C::OrdinaryObject);
    });
}

#[test]
fn context_mismatch_is_rejected_and_fresh_realms_recapture() {
    let rt = runtime();
    let a = Context::full(&rt).unwrap();
    let c = Context::full(&rt).unwrap();
    let saved = c.with(|cc| rquickjs::Persistent::save(&cc, Object::new(cc.clone()).unwrap()));
    a.with(|ac| {
        let b = ClassBridge::new(&ac).unwrap();
        // Supported restore within the same runtime retains the original Ctx.
        let foreign = saved.restore(&ac).unwrap();
        assert_eq!(b.classify(foreign.as_value()), Err(ForeignContext));
    });
    let other_rt = runtime();
    let other = Context::full(&other_rt).unwrap();
    a.with(|ac| {
        other.with(|oc| {
            let b = ClassBridge::new(&ac).unwrap();
            let foreign = Object::new(oc).unwrap();
            assert_eq!(b.classify(foreign.as_value()), Err(ForeignContext));
        })
    });
    for _ in 0..2 {
        let rt = runtime();
        let context = Context::full(&rt).unwrap();
        context.with(|ctx| {
            let b = ClassBridge::new(&ctx).unwrap();
            harden(&ctx);
            check(&ctx, &b, "({})", C::OrdinaryObject);
        });
    }
}
