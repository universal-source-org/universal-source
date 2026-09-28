use rquickjs::{Context, Ctx, Function, Module, Runtime, Value, context::intrinsic};

const HARDEN: &str = include_str!("harden.js");
const ROUTES: usize = 80;
const PROBES: &str = include_str!("probes.js");

fn runtime() -> Runtime {
    let rt = Runtime::new().unwrap();
    rt.set_memory_limit(16 * 1024 * 1024);
    rt.set_max_stack_size(512 * 1024);
    rt
}

fn harden(ctx: &Ctx<'_>) {
    ctx.eval::<(), _>(HARDEN).unwrap();
}

#[test]
fn all_dynamic_routes_throw_evalerror_without_payload_execution() {
    let rt = runtime();
    let c = Context::full(&rt).unwrap();
    c.with(|ctx| {
        harden(&ctx);
        let (m, p) = Module::declare(ctx.clone(), "probes.js", PROBES)
            .unwrap()
            .eval()
            .unwrap();
        p.result::<()>().unwrap().unwrap();
        let check: Function = m.get("check").unwrap();
        let count: usize = check.call(()).unwrap();
        println!("blocked dynamic routes: {count}");
        assert_eq!(count, ROUTES);
    });
    assert!(!rt.is_job_pending());
}

#[test]
fn hostile_mutation_cannot_restore_compilation_and_saved_aliases_stay_denied() {
    let rt = runtime();
    let c = Context::full(&rt).unwrap();
    c.with(|ctx| {
        harden(&ctx);
        let (m, p) = Module::declare(ctx.clone(), "probes.js", PROBES)
            .unwrap()
            .eval()
            .unwrap();
        p.result::<()>().unwrap().unwrap();
        let mutate: Function = m.get("mutate").unwrap();
        assert!(mutate.call::<_, bool>(()).unwrap());
        let check: Function = m.get("check").unwrap();
        assert_eq!(check.call::<_, usize>(()).unwrap(), ROUTES);
    });
    assert!(!rt.is_job_pending());
}

#[test]
fn missing_eval_intrinsic_disables_host_compilation_with_wrong_error_class() {
    let rt = runtime();
    let c = Context::custom::<intrinsic::Promise>(&rt).unwrap();
    c.with(|ctx| {
        let type_error: rquickjs::Object = ctx
            .globals()
            .get::<_, Function>("TypeError")
            .unwrap()
            .get("prototype")
            .unwrap();
        assert!(ctx.eval::<Value, _>("1 + 1").is_err());
        assert_eq!(
            ctx.catch().as_object().unwrap().get_prototype(),
            Some(type_error.clone())
        );
        assert!(Module::declare(ctx.clone(), "entry.js", "export function home() {}").is_err());
        assert_eq!(
            ctx.catch().as_object().unwrap().get_prototype(),
            Some(type_error)
        );
    });
}

#[test]
fn unchanged_intrinsic_relationships_and_ordinary_function_behavior() {
    let rt = runtime();
    let c = Context::full(&rt).unwrap();
    c.with(|ctx| {
        harden(&ctx);
        assert!(ctx.eval::<bool,_>(r#"(() => {
            const F = Function;
            const cs = [F, (async function(){}).constructor, (function*(){}).constructor,
                (async function*(){}).constructor];
            const fs = [function(){return 3}, async function(){return 4}, function*(){yield 5}, async function*(){yield 6}];
            for (let i=0;i<4;i++) {
                if (Object.getPrototypeOf(fs[i]) !== cs[i].prototype) return false;
                if (cs[i].prototype.constructor !== cs[i]) return false;
                if (!(fs[i] instanceof F) || !(fs[i] instanceof cs[i])) return false;
                if (i && Object.getPrototypeOf(cs[i]) !== F) return false;
            }
            if (F.name !== 'Function' || F.length !== 1 || eval.name !== 'eval' || eval.length !== 1) return false;
            if (F.prototype() !== undefined || fs[0].call(null) !== 3 || fs[0].bind(null)() !== 3) return false;
            if (fs[2]().next().value !== 5) return false;
            function C(x){this.x=x} class D extends C {}
            if (new D(7).x !== 7 || Reflect.construct(C,[8]).x !== 8) return false;
            Function.prototype.extra = 9;
            return fs[0].extra === 9 && new RegExp('a+').test('aaa');
        })()"#).unwrap());
    });
}

#[test]
fn denial_precedes_argument_coercion_and_newtarget_prototype_access() {
    let rt = runtime();
    let c = Context::full(&rt).unwrap();
    c.with(|ctx| {
        harden(&ctx);
        assert!(ctx.eval::<bool,_>(r#"(() => {
            let effects = 0;
            const arg = {toString(){effects++;return 'globalThis.__dynamic_code_executed=true'}};
            const cs=[Function,(async function(){}).constructor,(function*(){}).constructor,(async function*(){}).constructor];
            const E=EvalError;
            for(const C of cs) {
                for(const f of [()=>C(arg),()=>Reflect.construct(C,[arg]),()=>C('bad syntax ???')]) {
                    try{f();return false}catch(e){if(!(e instanceof E))return false}
                }
                const target=new Proxy(function(){},{get(t,k,r){if(k==='prototype')effects++; return Reflect.get(t,k,r)}});
                try{Reflect.construct(C,['return 1'],target);return false}catch(e){if(!(e instanceof E))return false}
            }
            try{eval(arg);return false}catch(e){if(!(e instanceof E))return false}
            return effects===0 && !Object.hasOwn(globalThis,'__dynamic_code_executed');
        })()"#).unwrap());
    });
}

#[test]
fn fresh_invocation_is_hardened_independently_after_hostile_realm_retirement() {
    for hostile in [true, false] {
        let rt = runtime();
        let c = Context::full(&rt).unwrap();
        c.with(|ctx| {
            harden(&ctx);
            assert!(ctx.eval::<bool,_>("!Object.hasOwn(globalThis,'hostileInvocation') && !Object.hasOwn(Function.prototype,'hostile')").unwrap());
            let (m,p)=Module::declare(ctx,"probes.js",PROBES).unwrap().eval().unwrap();
            p.result::<()>().unwrap().unwrap();
            if hostile { assert!(m.get::<_,Function>("mutate").unwrap().call::<_,bool>(()).unwrap()); }
            assert_eq!(m.get::<_,Function>("check").unwrap().call::<_,usize>(()).unwrap(),ROUTES);
        });
        assert!(!rt.is_job_pending());
    }
}

#[test]
fn unhardened_negative_controls_execute_payloads_for_all_four_families() {
    use rquickjs::{Promise, promise::PromiseState};
    let rt = runtime();
    let c = Context::full(&rt).unwrap();
    c.with(|ctx| {
        let p: Promise = ctx.eval(r#"(async () => {
            globalThis.__dynamic_code_executed = 0;
            const text='globalThis.__dynamic_code_executed++; return 123';
            const evalText='globalThis.__dynamic_code_executed++; 123';
            const e=eval;
            if(eval(evalText)!==123 || (0,e)(evalText)!==123) throw new Error('eval control');
            const cs=[Function,(async function(){}).constructor,(function*(){}).constructor,(async function*(){}).constructor];
            for(let i=0;i<4;i++) {
                const fn=Reflect.construct(cs[i],[text]);
                const result=i<2 ? await fn() : (await fn().next()).value;
                if(result!==123)throw new Error('constructor control');
            }
            return globalThis.__dynamic_code_executed;
        })()"#).unwrap();
        for _ in 0..32 {
            if p.state()!=PromiseState::Pending {break}
            assert!(ctx.execute_pending_job());
        }
        assert_eq!(p.result::<i32>().unwrap().unwrap(),6);
    });
    assert!(!rt.is_job_pending());
}

#[test]
fn deleting_only_globals_leaves_constructor_compilation_authority() {
    let rt = runtime();
    let c = Context::full(&rt).unwrap();
    c.with(|ctx| {
        ctx.eval::<(), _>("delete globalThis.eval; delete globalThis.Function;")
            .unwrap();
        assert_eq!(
            ctx.eval::<i32, _>(
                "(()=>{}).constructor('globalThis.__dynamic_code_executed=true; return 123')()"
            )
            .unwrap(),
            123
        );
        assert!(
            ctx.globals()
                .get::<_, bool>("__dynamic_code_executed")
                .unwrap()
        );
    });
}

#[test]
fn hardening_too_late_cannot_revoke_an_already_leaked_native_eval() {
    let rt = runtime();
    let c = Context::full(&rt).unwrap();
    c.with(|ctx| {
        // Deliberately unsafe ordering negative control, never the proposed lifecycle.
        ctx.eval::<(), _>("globalThis.leakedEval = eval").unwrap();
        harden(&ctx);
        assert_eq!(
            ctx.eval::<i32, _>("leakedEval('globalThis.__dynamic_code_executed=true; 123')")
                .unwrap(),
            123
        );
        assert!(
            ctx.globals()
                .get::<_, bool>("__dynamic_code_executed")
                .unwrap()
        );
    });
}

#[test]
fn guards_do_not_require_source_visible_proxy_or_mutable_reflect_helpers() {
    let rt = runtime();
    let c = Context::full(&rt).unwrap();
    c.with(|ctx| {
        harden(&ctx);
        let (m, p) = Module::declare(ctx.clone(), "probes.js", PROBES)
            .unwrap()
            .eval()
            .unwrap();
        p.result::<()>().unwrap().unwrap();
        ctx.eval::<(), _>(
            "delete globalThis.Proxy; Reflect.apply = () => 99; Reflect.construct = () => ({});",
        )
        .unwrap();
        assert_eq!(
            m.get::<_, Function>("check")
                .unwrap()
                .call::<_, usize>(())
                .unwrap(),
            ROUTES
        );
    });
}

#[test]
fn public_proxy_cannot_unwrap_a_guard_and_alternative_prototype_mutation_does_not_compile() {
    let rt = runtime();
    let c = Context::full(&rt).unwrap();
    c.with(|ctx| {
        harden(&ctx);
        assert!(ctx.eval::<bool,_>(r#"(() => {
            const E=EvalError, F=Function;
            const cs=[F,(async function(){}).constructor,(function*(){}).constructor,(async function*(){}).constructor];
            for(const C of cs) {
                // Proxy is only a stronger-than-profile diagnostic surface; RFC excludes its global.
                const wrapped=new Proxy(C,{});
                const handler={apply(t,that,args){return Reflect.apply(t,that,args)},construct(t,args){return Reflect.construct(t,args)}};
                const wrapped2=new Proxy(C,handler);
                for(const f of [()=>wrapped('return 123'),()=>new wrapped2('return 123')]) {
                    try{f();return false}catch(e){if(!(e instanceof E))return false}
                }
                Object.setPrototypeOf(C, null);
                try{Reflect.construct(C,['globalThis.__dynamic_code_executed=true']);return false}catch(e){if(!(e instanceof E))return false}
            }
            Object.setPrototypeOf(F.prototype,null);
            try{(()=>{}).constructor('return 123');return false}catch(e){if(!(e instanceof E))return false}
            return !Object.hasOwn(globalThis,'__dynamic_code_executed');
        })()"#).unwrap());
    });
}

#[test]
fn hardened_host_compile_link_capture_and_package_initialization_order() {
    use rquickjs::{
        Result,
        loader::{BuiltinLoader, BuiltinResolver},
        module::{Exports, ModuleDef},
    };
    use std::{cell::RefCell, rc::Rc};
    struct Capture;
    impl ModuleDef for Capture {
        fn evaluate<'js>(ctx: &Ctx<'js>, _: &Exports<'js>) -> Result<()> {
            ctx.remove_userdata::<Function<'js>>()
                .unwrap()
                .unwrap()
                .call(())
        }
    }
    const ENTRY: &[u8] = br#"
        // immutable package fixture: captures source aliases at its first body statements
        const e=eval, F=Function;
        mark('package');
        for(const f of [()=>e('globalThis.__pwned=true'),()=>F('return 123'),
            ()=>mark.constructor('return 123')]) {
            try { f(); throw new Error('unblocked init'); }
            catch(e) { if(!(e instanceof EvalError))throw e; }
        }
        export function home(){return 42}
    "#;
    let rt = runtime();
    rt.set_loader(
        BuiltinResolver::default()
            .with_module("entry.js")
            .with_module("@capture"),
        BuiltinLoader::default(),
    );
    let c = Context::full(&rt).unwrap();
    c.with(|ctx| {
        let trace = Rc::new(RefCell::new(Vec::<String>::new()));
        harden(&ctx);
        trace.borrow_mut().push("harden".into());
        let t = trace.clone();
        ctx.globals()
            .set(
                "mark",
                Function::new(ctx.clone(), move |s: String| {
                    t.borrow_mut().push(s);
                })
                .unwrap(),
            )
            .unwrap();
        trace.borrow_mut().push("bindings".into());
        let bytes = ENTRY.to_vec();
        assert_eq!(bytes, ENTRY);
        let entry = Module::declare(ctx.clone(), "entry.js", bytes).unwrap();
        trace.borrow_mut().push("compile".into());
        assert!(!ctx.execute_pending_job());
        let captured = Rc::new(RefCell::new(None::<Function>));
        let saved = captured.clone();
        let m = entry.clone();
        let t = trace.clone();
        let callback = Function::new(ctx.clone(), move || -> Result<()> {
            // Public post-link native hook from the prior spike, not namespace access before linking.
            let f: Function = m.get("home")?;
            let proto = f.as_object().unwrap().get_prototype().unwrap();
            let ctor: Function = proto.get("constructor")?;
            // Call the compiler guard, never the source operation, before package evaluation.
            assert!(ctor.call::<_, Value>(("return 123",)).is_err());
            let error = f.ctx().catch();
            let expected: rquickjs::Object = f
                .ctx()
                .globals()
                .get::<_, Function>("EvalError")?
                .get("prototype")?;
            assert_eq!(error.as_object().unwrap().get_prototype(), Some(expected));
            *saved.borrow_mut() = Some(f);
            t.borrow_mut().push("capture".into());
            Ok(())
        })
        .unwrap();
        ctx.store_userdata(callback).unwrap();
        Module::declare_def::<Capture, _>(ctx.clone(), "@capture").unwrap();
        let (_, p) = Module::declare(
            ctx.clone(),
            "@root",
            "import '@capture'; import 'entry.js';",
        )
        .unwrap()
        .eval()
        .unwrap();
        p.result::<()>().unwrap().unwrap();
        assert_eq!(
            &*trace.borrow(),
            &["harden", "bindings", "compile", "capture", "package"]
        );
        assert_eq!(
            captured
                .borrow()
                .as_ref()
                .unwrap()
                .call::<_, i32>(())
                .unwrap(),
            42
        );
        assert!(
            ctx.eval::<bool, _>("!Object.hasOwn(globalThis,'__pwned')")
                .unwrap()
        );
    });
    assert!(!rt.is_job_pending());
}

#[test]
fn failing_to_redirect_specialized_constructor_parents_leaks_native_function() {
    let rt = runtime();
    let c = Context::full(&rt).unwrap();
    c.with(|ctx| {
        // Intentionally incomplete negative control: globals and constructor properties alone.
        assert_eq!(ctx.eval::<i32,_>(r#"(() => {
            const cs=[Function,(async function(){}).constructor,(function*(){}).constructor,(async function*(){}).constructor];
            const deny=()=>{throw new EvalError('disabled')};
            const ps=cs.map(c=>new Proxy(c,{apply:deny,construct:deny}));
            cs.forEach((c,i)=>Object.defineProperty(c.prototype,'constructor',{value:ps[i]}));
            globalThis.Function=ps[0];
            const escaped=Object.getPrototypeOf((async function(){}).constructor);
            return escaped('globalThis.__dynamic_code_executed=true; return 123')();
        })()"#).unwrap(),123);
        assert!(ctx.globals().get::<_,bool>("__dynamic_code_executed").unwrap());
    });
}

#[test]
fn hostile_package_top_level_mutation_keeps_initialization_aliases_guarded() {
    use rquickjs::loader::{BuiltinLoader, BuiltinResolver};
    const ENTRY: &[u8] =
        b"import {mutate,check} from './probes.js'; mutate(); export const blocked = check();";
    let rt = runtime();
    rt.set_loader(
        BuiltinResolver::default().with_module("probes.js"),
        BuiltinLoader::default().with_module("probes.js", PROBES),
    );
    let c = Context::full(&rt).unwrap();
    c.with(|ctx| {
        harden(&ctx);
        let bytes = ENTRY.to_vec();
        assert_eq!(bytes, ENTRY);
        let (m, p) = Module::declare(ctx, "entry.js", bytes)
            .unwrap()
            .eval()
            .unwrap();
        p.result::<()>().unwrap().unwrap();
        assert_eq!(m.get::<_, usize>("blocked").unwrap(), ROUTES);
    });
    assert!(!rt.is_job_pending());
}
