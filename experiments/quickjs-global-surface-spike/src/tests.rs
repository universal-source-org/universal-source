use rquickjs::{Context, Ctx, Function, Module, Runtime, Value};

const INVENTORY: &str = include_str!("inventory.js");
const PRISTINE: &str = include_str!("pristine-inventory.json");
const GLOBALS: &str = include_str!("allowed-globals.json");
const INTRINSICS: &str = include_str!("allowed-intrinsics.json");
const DYNAMIC_HARDEN: &str = include_str!("../../quickjs-dynamic-code-spike/src/harden.js");
const DYNAMIC_PROBES: &str = include_str!("../../quickjs-dynamic-code-spike/src/probes.js");
const HARDEN: &str = include_str!("harden.js");
const PROBES: &str = include_str!("probes.js");

fn runtime() -> Runtime {
    let rt = Runtime::new().unwrap();
    rt.set_memory_limit(32 * 1024 * 1024);
    rt.set_max_stack_size(512 * 1024);
    rt
}

fn eval<'js, T: rquickjs::FromJs<'js>>(ctx: &Ctx<'js>, text: &str) -> T {
    match ctx.eval(text) {
        Ok(v) => v,
        Err(e) => panic!("evaluation failed: {e:?}; exception: {:?}", ctx.catch()),
    }
}

fn inventory_matches(ctx: &Ctx<'_>) -> bool {
    let actual: String = eval(ctx, INVENTORY);
    let expected: String = eval(ctx, &format!("JSON.stringify({PRISTINE})"));
    actual == expected
}

fn harden(ctx: &Ctx<'_>) {
    assert!(
        inventory_matches(ctx),
        "unexpected pristine intrinsic/global drift"
    );
    eval::<()>(ctx, DYNAMIC_HARDEN);
    let harden: Function = eval(ctx, HARDEN);
    let globals: Value = eval(ctx, GLOBALS);
    let intrinsics: Value = eval(ctx, &format!("({INTRINSICS})"));
    harden.call::<_, ()>((globals, intrinsics)).unwrap();
}

fn probe(name: &str) {
    let rt = runtime();
    let c = Context::full(&rt).unwrap();
    c.with(|ctx| {
        harden(&ctx);
        let (m, p) = Module::declare(ctx.clone(), "/fixture/package/probes.js", PROBES)
            .unwrap()
            .eval()
            .unwrap();
        p.result::<()>().unwrap().unwrap();
        let f: Function = m.get(name).unwrap();
        match f.call::<_, bool>(()) {
            Ok(value) => assert!(value, "{name}"),
            Err(e) => panic!("{name}: {e:?}: {:?}", ctx.catch()),
        }
    });
    assert!(!rt.is_job_pending());
}

#[test]
fn pristine_inventory_and_descriptors_match_pinned_snapshot() {
    let rt = runtime();
    let c = Context::full(&rt).unwrap();
    c.with(|ctx| assert!(inventory_matches(&ctx)));
}

#[test]
fn hardened_exact_global_own_keys_and_descriptors() {
    let rt = runtime();
    let c = Context::full(&rt).unwrap();
    c.with(|ctx| {
        harden(&ctx);
        assert!(eval::<bool>(
            &ctx,
            &format!(
                r#"(() => {{
            const expected = {GLOBALS};
            const keys = [Object.getOwnPropertyNames(globalThis), Reflect.ownKeys(globalThis),
                Object.keys(Object.getOwnPropertyDescriptors(globalThis))];
            return keys.every(k => JSON.stringify(k.sort()) === JSON.stringify(expected)) &&
                Object.getOwnPropertySymbols(globalThis).length === 0 &&
                !Object.getOwnPropertyDescriptor(globalThis,'eval').configurable &&
                !Object.getOwnPropertyDescriptor(globalThis,'Function').writable;
        }})()"#
            )
        ));
    });
}

#[test]
fn drift_fails_before_source_and_hardening() {
    for extra in [
        "globalThis.unexpectedEngineGlobal = 1",
        "Math.unexpectedEngineExtension = 1",
    ] {
        let rt = runtime();
        let c = Context::full(&rt).unwrap();
        c.with(|ctx| {
            eval::<()>(&ctx, extra);
            assert!(!inventory_matches(&ctx));
            // Same gate used by harden(): host must retire this realm, never evaluate a package.
        });
    }
}

#[test]
fn browser_node_and_other_host_surfaces_absent_in_pristine_and_hardened_realms() {
    let rt = runtime();
    let c = Context::full(&rt).unwrap();
    c.with(|ctx| {
        for hardened in [false, true] {
            if hardened {
                harden(&ctx);
            }
            assert!(eval::<bool>(
                &ctx,
                r#"[
                'window','document','navigator','location','fetch','XMLHttpRequest','WebSocket',
                'localStorage','sessionStorage','crypto','process','require','module','exports',
                'Buffer','__dirname','__filename','global','console','setTimeout','setInterval',
                'setImmediate','Worker','URL','std','os','scriptArgs','print','load','gc',
                'WebAssembly','Intl','Deno','Bun'
            ].every(k => !(k in globalThis) && !Reflect.ownKeys(globalThis).includes(k))"#
            ));
        }
    });
}

#[test]
fn clocks_timezone_random_locale_and_extension_negative_controls() {
    let rt = runtime();
    let c = Context::full(&rt).unwrap();
    c.with(|ctx| {
        let before = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as f64;
        let now: f64 = eval(&ctx, "Date.now()");
        let after = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as f64;
        assert!(before <= now && now <= after);
        let observation: String = eval(&ctx, r#"JSON.stringify({
            dateCall: Date(), epochString: new Date(0).toString(),
            timezone: new Date(0).getTimezoneOffset(), dateLocale: new Date(0).toLocaleString(),
            localeUpper: 'i'.toLocaleUpperCase(), numberLocale: (1234.5).toLocaleString(),
            bigintLocale: (1234n).toLocaleString(), intl: typeof Intl,
            monotonic: typeof performance.now(), timeOrigin: typeof performance.timeOrigin,
            stack: new Error('x').stack, fileName: (function source(){}).fileName
        })"#);
        println!("NEGATIVE CONTROL (environment-specific): {observation}");
        assert!(eval::<bool>(&ctx, "new Set(Array.from({length:16},()=>Math.random())).size > 1"));
        assert!(eval::<bool>(&ctx, "typeof performance.now === 'function' && typeof Error.captureStackTrace === 'function' && typeof Symbol.dispose === 'symbol'"));
        assert!(eval::<bool>(&ctx, "!Object.getOwnPropertyDescriptor(Symbol,'dispose').configurable && !Reflect.deleteProperty(Symbol,'dispose')"));
    });
}

#[test]
fn wall_clock_and_timezone_paths_blocked() {
    probe("clocks");
}
#[test]
fn randomness_and_locale_methods_throw_exact_typeerror() {
    probe("restrictedMethods");
}
#[test]
fn quickjs_extensions_gc_shared_memory_binary_and_wasm_unavailable() {
    probe("extensions");
}
#[test]
fn error_families_and_engine_exceptions_have_no_stack() {
    probe("stacks");
}
#[test]
fn protected_methods_survive_mutation_and_source_replacements_are_harmless() {
    probe("mutations");
}
#[test]
fn function_reflection_has_no_host_paths_or_privileged_source() {
    probe("reflection");
}

#[test]
fn full_dynamic_corpus_survives_global_hardening_and_hostile_initialization() {
    let rt = runtime();
    let c = Context::full(&rt).unwrap();
    c.with(|ctx| {
        harden(&ctx);
        let (m, p) = Module::declare(ctx.clone(), "dynamic.js", DYNAMIC_PROBES)
            .unwrap()
            .eval()
            .unwrap();
        p.result::<()>().unwrap().unwrap();
        assert_eq!(
            m.get::<_, Function>("check")
                .unwrap()
                .call::<_, usize>(())
                .unwrap(),
            80
        );
        assert!(
            m.get::<_, Function>("mutate")
                .unwrap()
                .call::<_, bool>(())
                .unwrap()
        );
        assert_eq!(
            m.get::<_, Function>("check")
                .unwrap()
                .call::<_, usize>(())
                .unwrap(),
            80
        );
    });
}

#[test]
fn fresh_realms_reset_hostile_globals_and_intrinsics() {
    for hostile in [true, false] {
        let rt = runtime();
        let c = Context::full(&rt).unwrap();
        c.with(|ctx| {
            harden(&ctx);
            assert!(eval::<bool>(&ctx,"!('foo' in globalThis) && !('hostile' in Object.prototype) && typeof Date === 'undefined'"));
            let (m,p)=Module::declare(ctx.clone(),"probes.js",PROBES).unwrap().eval().unwrap();
            p.result::<()>().unwrap().unwrap();
            assert!(m.get::<_,Function>("restrictedMethods").unwrap().call::<_,bool>(()).unwrap());
            if hostile {
                assert!(m.get::<_,Function>("mutations").unwrap().call::<_,bool>(()).unwrap());
                eval::<()>(&ctx,"globalThis.foo = 1; Object.prototype.hostile = true;");
            }
        });
        assert!(!rt.is_job_pending());
    }
}

#[test]
fn transitive_descriptor_graph_excludes_captured_forbidden_identities() {
    let rt = runtime();
    let c = Context::full(&rt).unwrap();
    c.with(|ctx| {
        // Test-private witness retains forbidden identities only in a host-held closure.
        // No package receives this closure or its captured references.
        let make_witness: Function = eval(&ctx, include_str!("traverse.js"));
        let globals: Value = eval(&ctx, GLOBALS);
        let witness: Function = make_witness.call((globals,)).unwrap();
        // Negative control proves the witness detects the original authority graph.
        assert!(
            witness
                .call::<_, String>((false,))
                .unwrap()
                .starts_with("forbidden:")
        );
        harden(&ctx);
        let stats: String = witness.call((true,)).unwrap();
        println!("completed bounded intrinsic traversal: {stats}");
        assert!(stats.starts_with("clean:"));
    });
}

#[test]
fn saved_pre_hardening_stack_and_random_aliases_are_unsafe_negative_control() {
    let rt = runtime();
    let c = Context::full(&rt).unwrap();
    c.with(|ctx| {
        // Held only by host test code. This intentionally shows why ordering is mandatory.
        let random: Function = eval(&ctx,"Math.random");
        let leak: Function = eval(&ctx,"(() => {const get = Object.getOwnPropertyDescriptor(Error.prototype,'stack').get; return () => get.call(new Error('leak'));})()");
        harden(&ctx);
        let random_value: f64 = random.call(()).unwrap();
        assert!((0.0..1.0).contains(&random_value));
        let stack: String = leak.call(()).unwrap();
        assert!(stack.contains("eval_script"));
    });
}

struct PrivateResolver(rquickjs::loader::BuiltinResolver);
impl rquickjs::loader::Resolver for PrivateResolver {
    fn resolve<'js>(
        &mut self,
        ctx: &Ctx<'js>,
        base: &str,
        name: &str,
        attributes: Option<rquickjs::loader::ImportAttributes<'js>>,
    ) -> rquickjs::Result<String> {
        let resolved = self.0.resolve(ctx, base, name, attributes)?;
        if resolved.starts_with("@host/") && !(base == "@host/root" && resolved == "@host/capture")
        {
            return Err(rquickjs::Error::new_resolving(base, name));
        }
        Ok(resolved)
    }
}
struct CaptureHook;
impl rquickjs::module::ModuleDef for CaptureHook {
    fn evaluate<'js>(ctx: &Ctx<'js>, _: &rquickjs::module::Exports<'js>) -> rquickjs::Result<()> {
        ctx.remove_userdata::<Function<'js>>()
            .unwrap()
            .unwrap()
            .call(())
    }
}

#[test]
fn host_binding_loader_capture_order_and_allowed_language_work_together() {
    use rquickjs::{
        Promise,
        loader::{BuiltinLoader, BuiltinResolver},
        promise::PromiseState,
    };
    use std::{cell::RefCell, rc::Rc};
    let rt = runtime();
    rt.set_loader(
        PrivateResolver(
            BuiltinResolver::default()
                .with_module("entry.js")
                .with_module("helper.js")
                .with_module("@host/capture"),
        ),
        BuiltinLoader::default().with_module("helper.js", "export const value = 40;"),
    );
    let c = Context::full(&rt).unwrap();
    c.with(|ctx| {
        let make_witness:Function=eval(&ctx,include_str!("traverse.js"));
        let globals:Value=eval(&ctx,GLOBALS);
        let witness:Function=make_witness.call((globals,)).unwrap();
        harden(&ctx);
        // Kept in a scoped Rust handle; no global host bridge or registry is installed.
        let binding=Function::new(ctx.clone(),|v:i32| v+2).unwrap();
        let wrap:Function=eval(&ctx,include_str!("binding.js"));
        let context:Value=wrap.call((binding,)).unwrap();
        let stats:String=witness.call((true,context.clone())).unwrap();
        assert!(stats.starts_with("clean:"));
        println!("binding traversal: {stats}");
        let entry_bytes=include_bytes!("entry.js").to_vec();
        assert_eq!(entry_bytes,include_bytes!("entry.js"));
        let entry=Module::declare(ctx.clone(),"entry.js",entry_bytes).unwrap();
        let captured=Rc::new(RefCell::new(None::<Function>));
        let saved=captured.clone();
        let m=entry.clone();
        let observer=ctx.clone();
        let callback=Function::new(ctx.clone(),move || -> rquickjs::Result<()> {
            assert!(!observer.globals().contains_key("packageEvaluated")?);
            *saved.borrow_mut()=Some(m.get("home")?);
            Ok(())
        }).unwrap();
        ctx.store_userdata(callback).unwrap();
        Module::declare_def::<CaptureHook,_>(ctx.clone(),"@host/capture").unwrap();
        let (_,p)=Module::declare(ctx.clone(),"@host/root","import '@host/capture'; import 'entry.js';")
            .unwrap().eval().unwrap();
        p.result::<()>().unwrap().unwrap();
        assert!(!ctx.execute_pending_job());
        assert!(ctx.globals().get::<_,bool>("packageEvaluated").unwrap());
        let f=captured.borrow_mut().take().unwrap();
        assert_eq!(f,entry.get::<_,Function>("home").unwrap());
        let p:Promise=f.call((2,context)).unwrap();
        for _ in 0..16 {
            if p.state()!=PromiseState::Pending {break;}
            assert!(ctx.execute_pending_job());
        }
        assert_eq!(p.result::<i32>().unwrap().unwrap(),42);
        assert!(ctx.remove_userdata::<Function>().unwrap().is_none());
        assert!(eval::<bool>(&ctx,"['resolver','loader','snapshot','capture','context','runtime','services','testPrivate'].every(k=>!(k in globalThis))"));
        // Host machinery is registered, but importing it from package text is rejected.
        for text in ["import '@host/capture';", "import '@host/root';", "import 'std';", "import 'os';", "import '/etc/passwd';"] {
            assert!(Module::declare(ctx.clone(),"hostile.js",text).is_err());
            ctx.catch();
        }
    });
    assert!(!rt.is_job_pending());
}

#[test]
fn hostile_module_initialization_reuses_all_eighty_dynamic_routes() {
    use rquickjs::loader::{BuiltinLoader, BuiltinResolver};
    let rt = runtime();
    rt.set_loader(
        BuiltinResolver::default().with_module("dynamic.js"),
        BuiltinLoader::default().with_module("dynamic.js", DYNAMIC_PROBES),
    );
    let c = Context::full(&rt).unwrap();
    c.with(|ctx| {
        harden(&ctx);
        let (_,p)=Module::declare(ctx.clone(),"hostile-init.js",
            "import {mutate,check} from './dynamic.js'; mutate(); if(check()!==80)throw new Error('routes');")
            .unwrap().eval().unwrap();
        p.result::<()>().unwrap().unwrap();
        assert!(!ctx.execute_pending_job());
    });
}

#[test]
fn hidden_native_error_family_prototypes_cannot_recover_extension_constructors() {
    let rt = runtime();
    let c = Context::full(&rt).unwrap();
    c.with(|ctx| {
        // Test-private injection stands in for engine-created InternalError paths.
        // Retained constructors never enter a source global or a host binding result.
        let internal:Function=eval(&ctx,"InternalError");
        let suppressed:Function=eval(&ctx,"SuppressedError");
        harden(&ctx);
        let inspect:Function=eval(&ctx,"e => e.stack === undefined && !('stack' in e) && e.constructor === Error && e.name === 'Error'");
        let e:Value=internal.call(("internal fixture",)).unwrap();
        assert!(inspect.call::<_,bool>((e,)).unwrap());
        let e:Value=suppressed.call((1,2,"suppressed fixture")).unwrap();
        assert!(inspect.call::<_,bool>((e,)).unwrap());
    });
}

#[test]
fn pristine_module_nested_and_native_error_stacks_leak_locations_negative_control() {
    let rt = runtime();
    let c = Context::full(&rt).unwrap();
    c.with(|ctx| {
        let binding = Function::new(ctx.clone(), |v: i32| v).unwrap();
        let (m, p) = Module::declare(
            ctx.clone(),
            "/fixture/private/package/source.js",
            r#"
            export const initial = new Error('initial').stack;
            export function inspect(binding) {
                function nested(){try{binding({});}catch(e){return e.stack;}}
                return nested();
            }
        "#,
        )
        .unwrap()
        .eval()
        .unwrap();
        p.result::<()>().unwrap().unwrap();
        let initial: String = m.get("initial").unwrap();
        let nested: String = m
            .get::<_, Function>("inspect")
            .unwrap()
            .call((binding,))
            .unwrap();
        println!("NEGATIVE CONTROL module stack: {initial}; native/nested stack: {nested}");
        assert!(initial.contains("/fixture/private/package/source.js"));
        assert!(nested.contains("/fixture/private/package/source.js") && nested.contains("nested"));
    });
}
