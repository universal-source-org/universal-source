use rquickjs::{
    Context, Ctx, Error, Function, Module, Result, Runtime, Value,
    loader::{BuiltinLoader, BuiltinResolver, ImportAttributes, Loader, Resolver},
    module::{Declared, Exports, ModuleDef},
    promise::PromiseState,
};
use std::{cell::RefCell, rc::Rc};

type Trace = Rc<RefCell<Vec<String>>>;
type Captured<'js> = Rc<RefCell<Vec<(String, Function<'js>)>>>;
const ROOT: &str = "@host/root";
const CAPTURE: &str = "@host/capture";

// Fixed fixture routing only, not path containment or the RFC graph preflight.
struct FixtureResolver(BuiltinResolver);
impl Resolver for FixtureResolver {
    fn resolve<'js>(
        &mut self,
        ctx: &Ctx<'js>,
        base: &str,
        name: &str,
        attributes: Option<ImportAttributes<'js>>,
    ) -> Result<String> {
        let resolved = self.0.resolve(ctx, base, name, attributes)?;
        if resolved.starts_with("@host/") && !(base == ROOT && resolved == CAPTURE) {
            return Err(Error::new_resolving(base, name));
        }
        Ok(resolved)
    }
}

struct FixtureLoader(BuiltinLoader, Trace);
impl Loader for FixtureLoader {
    fn load<'js>(
        &mut self,
        ctx: &Ctx<'js>,
        name: &str,
        attributes: Option<ImportAttributes<'js>>,
    ) -> Result<Module<'js>> {
        self.1.borrow_mut().push(format!("load:{name}"));
        self.0.load(ctx, name, attributes)
    }
}

// This is a host-native, export-free module. Its callback is private runtime
// userdata, never a global or a source export. It executes no source function.
struct CaptureHook;
impl ModuleDef for CaptureHook {
    fn evaluate<'js>(ctx: &Ctx<'js>, _: &Exports<'js>) -> Result<()> {
        let callback = ctx.remove_userdata::<Function<'js>>().unwrap().unwrap();
        callback.call(())
    }
}

fn fixture(dependencies: &[(&str, &str)]) -> (Runtime, Context, Trace) {
    let rt = Runtime::new().unwrap();
    rt.set_memory_limit(16 * 1024 * 1024);
    rt.set_max_stack_size(256 * 1024);
    let trace = Trace::default();
    let mut resolver = BuiltinResolver::default()
        .with_module("entry.js")
        .with_module(CAPTURE);
    let mut loader = BuiltinLoader::default();
    for (name, text) in dependencies {
        resolver.add_module(*name);
        loader.add_module(*name, *text);
    }
    rt.set_loader(
        FixtureResolver(resolver),
        FixtureLoader(loader, trace.clone()),
    );
    let context = Context::full(&rt).unwrap();
    context.with(|ctx| {
        let observed = trace.clone();
        // Test-only body observation, not an RFC source capability.
        let mark = Function::new(ctx.clone(), move |name: String| {
            observed.borrow_mut().push(format!("body:{name}"));
        })
        .unwrap();
        ctx.globals().set("mark", mark).unwrap();
        Module::declare_def::<CaptureHook, _>(ctx, CAPTURE).unwrap();
    });
    (rt, context, trace)
}

fn install_capture<'js>(
    ctx: &Ctx<'js>,
    entry: &Module<'js>,
    operations: &[&str],
    trace: &Trace,
) -> Captured<'js> {
    let saved = Captured::default();
    let output = saved.clone();
    let module = entry.clone();
    let observed = trace.clone();
    let mut expected: Vec<_> = operations.iter().map(|s| s.to_string()).collect();
    expected.sort();
    let callback = Function::new(ctx.clone(), move || -> Result<()> {
        observed.borrow_mut().push("capture:start".into());
        // This is reached AFTER engine linking but BEFORE any source body.
        // Never call namespace() on a merely compile-only, unlinked module.
        let namespace = module.namespace()?;
        let mut names = namespace.keys::<String>().collect::<Result<Vec<_>>>()?;
        names.sort();
        if names != expected {
            return Err(Error::new_from_js_message(
                "module",
                "operations",
                "export name mismatch",
            ));
        }
        let functions = names
            .into_iter()
            .map(|name| namespace.get(name.as_str()).map(|value| (name, value)))
            .collect::<Result<Vec<_>>>()?;
        *output.borrow_mut() = functions;
        observed.borrow_mut().push("capture:done".into());
        Ok(())
    })
    .unwrap();
    assert!(ctx.store_userdata(callback).unwrap().is_none());
    saved
}

fn root<'js>(ctx: &Ctx<'js>) -> Module<'js, Declared> {
    Module::declare(
        ctx.clone(),
        ROOT,
        "import '@host/capture'; import 'entry.js';",
    )
    .unwrap()
}

fn identities_match<'js>(entry: &Module<'js>, saved: &Captured<'js>) -> bool {
    saved.borrow().iter().all(|(name, function)| {
        entry
            .get::<_, Value>(name.as_str())
            .is_ok_and(|value| value == *function.as_value())
    })
}

fn clear_callback(ctx: &Ctx<'_>) {
    // Also required on link failure, when the native hook was never reached.
    // Release rooted closure/module references before retiring Context/Runtime.
    ctx.remove_userdata::<Function>().unwrap();
}

#[test]
fn compile_resolves_graph_without_body_or_job_execution() {
    let (rt, context, trace) = fixture(&[("dep.js", "mark('dep'); export const value = 7;")]);
    context.with(|ctx| {
        let entry = Module::declare(ctx.clone(), "entry.js", "import {value} from './dep.js'; mark('entry'); export function home() { return value; }").unwrap();
        assert_eq!(entry.name::<String>().unwrap(), "entry.js");
        assert_eq!(*trace.borrow(), ["load:dep.js"]);
        let saved = install_capture(&ctx, &entry, &["home"], &trace);
        let root = root(&ctx);
        assert_eq!(*trace.borrow(), ["load:dep.js"]);
        assert!(saved.borrow().is_empty());
        // No evaluation and no capture. Dropping compile-only records is safe.
        drop(root);
        clear_callback(&ctx);
    });
    assert!(!rt.is_job_pending());
}

#[test]
fn native_hook_captures_all_direct_sync_and_async_declarations_before_bodies() {
    let (rt, context, trace) = fixture(&[]);
    context.with(|ctx| {
        let entry = Module::declare(ctx.clone(), "entry.js", "mark('entry'); export function home() { return 1; } export async function category() { return 2; } export function search() {} export async function detail() {} export function play() {}").unwrap();
        let saved = install_capture(&ctx, &entry, &["home", "category", "search", "detail", "play"], &trace);
        let (_, promise) = root(&ctx).eval().unwrap();
        assert_eq!(promise.state(), PromiseState::Resolved);
        assert_eq!(*trace.borrow(), ["capture:start", "capture:done", "body:entry"]);
        assert_eq!(saved.borrow().len(), 5);
        assert!(identities_match(&entry, &saved));
        let home = saved.borrow().iter().find(|(name, _)| name == "home").unwrap().1.clone();
        assert_eq!(home.call::<_, i32>(()).unwrap(), 1);
        let category = saved.borrow().iter().find(|(name, _)| name == "category").unwrap().1.clone();
        let result: rquickjs::Promise = category.call(()).unwrap();
        assert_eq!(result.result::<i32>().unwrap().unwrap(), 2);
        clear_callback(&ctx);
    });
    assert!(!rt.is_job_pending());
}

#[test]
fn top_level_replacement_is_detected_against_real_precaptured_identity() {
    for replacement in ["() => 'replacement'", "42"] {
        let (rt, context, trace) = fixture(&[]);
        context.with(|ctx| {
            let text = format!("mark('entry'); export function home() {{ return 'original'; }} home = {replacement};");
            let entry = Module::declare(ctx.clone(), "entry.js", text).unwrap();
            let saved = install_capture(&ctx, &entry, &["home"], &trace);
            root(&ctx).eval().unwrap().1.result::<()>().unwrap().unwrap();
            assert_eq!(*trace.borrow(), ["capture:start", "capture:done", "body:entry"]);
            assert!(!identities_match(&entry, &saved));
            // Diagnostic-only call proves the root contains the actual original;
            // a runtime must reject initialization instead of dispatching here.
            assert_eq!(saved.borrow()[0].1.call::<_, String>(()).unwrap(), "original");
            clear_callback(&ctx);
        });
        assert!(!rt.is_job_pending());
    }
}

#[test]
fn later_operation_assignment_changes_namespace_but_not_captured_dispatch() {
    let (rt, context, trace) = fixture(&[]);
    context.with(|ctx| {
        let entry = Module::declare(
            ctx.clone(),
            "entry.js",
            "export function home() { home = () => 'replacement'; return 'original'; }",
        )
        .unwrap();
        let saved = install_capture(&ctx, &entry, &["home"], &trace);
        root(&ctx)
            .eval()
            .unwrap()
            .1
            .result::<()>()
            .unwrap()
            .unwrap();
        assert!(identities_match(&entry, &saved));
        assert_eq!(
            saved.borrow()[0].1.call::<_, String>(()).unwrap(),
            "original"
        );
        assert!(!identities_match(&entry, &saved));
        assert_eq!(
            entry
                .get::<_, Function>("home")
                .unwrap()
                .call::<_, String>(())
                .unwrap(),
            "replacement"
        );
        assert_eq!(
            saved.borrow()[0].1.call::<_, String>(()).unwrap(),
            "original"
        );
        clear_callback(&ctx);
    });
    assert!(!rt.is_job_pending());
}

#[test]
fn direct_eval_negative_control_loses_original_before_host_can_read_exports() {
    let (rt, context, trace) = fixture(&[]);
    context.with(|ctx| {
        let entry = Module::declare(ctx, "entry.js", "mark('entry'); export function home() { return 'original'; } home = () => 'replacement';").unwrap();
        assert!(trace.borrow().is_empty());
        let (entry, promise) = entry.eval().unwrap();
        promise.result::<()>().unwrap().unwrap();
        assert_eq!(*trace.borrow(), ["body:entry"]);
        assert_eq!(entry.get::<_, Function>("home").unwrap().call::<_, String>(()).unwrap(), "replacement");
    });
    assert!(!rt.is_job_pending());
}

#[test]
fn lexical_exports_are_uninitialized_at_capture_and_fail_before_bodies() {
    // JS-valid but RFC-invalid: these are engine-phase controls, not accepted entries.
    for declaration in [
        "export const home = 42;",
        "export class home {}",
        "export let home = function original() {}; home = function replacement() {};",
    ] {
        let (rt, context, trace) = fixture(&[]);
        context.with(|ctx| {
            let entry = Module::declare(
                ctx.clone(),
                "entry.js",
                format!("mark('entry'); {declaration}"),
            )
            .unwrap();
            let saved = install_capture(&ctx, &entry, &["home"], &trace);
            let (_, promise) = root(&ctx).eval().unwrap();
            assert_eq!(promise.state(), PromiseState::Rejected);
            assert!(promise.result::<()>().unwrap().is_err());
            let exception = ctx.catch().into_object().unwrap();
            assert_eq!(
                exception.get::<_, String>("name").unwrap(),
                "ReferenceError"
            );
            assert!(saved.borrow().is_empty());
            assert_eq!(*trace.borrow(), ["capture:start"]);
            clear_callback(&ctx);
        });
        assert!(!rt.is_job_pending());
    }
}

#[test]
fn namespace_callable_checks_cannot_validate_generator_alias_or_reexport_syntax() {
    for declaration in [
        "export function* home() {}",
        "export async function* home() {}",
        "function impl() {} export {impl as home};",
        "function home() {} export {home};",
        "export {home} from './dep.js';",
    ] {
        let (rt, context, trace) = fixture(&[("dep.js", "mark('dep'); export function home() {}")]);
        context.with(|ctx| {
            let entry = Module::declare(
                ctx.clone(),
                "entry.js",
                format!("mark('entry'); {declaration}"),
            )
            .unwrap();
            let saved = install_capture(&ctx, &entry, &["home"], &trace);
            root(&ctx)
                .eval()
                .unwrap()
                .1
                .result::<()>()
                .unwrap()
                .unwrap();
            assert_eq!(saved.borrow().len(), 1);
            assert!(identities_match(&entry, &saved));
            let events = trace.borrow();
            assert!(
                events.iter().position(|e| e == "capture:done").unwrap()
                    < events.iter().position(|e| e.starts_with("body:")).unwrap()
            );
            clear_callback(&ctx);
        });
        assert!(!rt.is_job_pending());
    }
}

#[test]
fn default_extra_and_missing_export_names_fail_before_source_bodies() {
    for declaration in [
        "export default function home() {}",
        "export function home() {} export function extra() {}",
        "function home() {}",
    ] {
        let (rt, context, trace) = fixture(&[]);
        context.with(|ctx| {
            let entry = Module::declare(
                ctx.clone(),
                "entry.js",
                format!("mark('entry'); {declaration}"),
            )
            .unwrap();
            let saved = install_capture(&ctx, &entry, &["home"], &trace);
            let (_, promise) = root(&ctx).eval().unwrap();
            assert!(promise.result::<()>().unwrap().is_err());
            assert!(ctx.catch().is_error());
            assert!(saved.borrow().is_empty());
            assert_eq!(*trace.borrow(), ["capture:start"]);
            clear_callback(&ctx);
        });
        assert!(!rt.is_job_pending());
    }
}

#[test]
fn duplicate_exports_fail_compile_and_missing_import_fails_link_before_capture() {
    let (rt, context, trace) = fixture(&[("dep.js", "mark('dep'); export const existing = 1;")]);
    context.with(|ctx| {
        let duplicate = Module::declare(ctx.clone(), "duplicate.js", "mark('duplicate'); export function home() {} export {home};");
        assert!(duplicate.is_err());
        assert!(ctx.catch().is_error());
        assert!(trace.borrow().is_empty());
        let entry = Module::declare(ctx.clone(), "entry.js", "import {missing} from './dep.js'; mark('entry'); export function home() { return missing; }").unwrap();
        let saved = install_capture(&ctx, &entry, &["home"], &trace);
        assert!(root(&ctx).eval().is_err());
        assert!(ctx.catch().is_error());
        assert!(saved.borrow().is_empty());
        assert_eq!(*trace.borrow(), ["load:dep.js"]);
        clear_callback(&ctx);
    });
    assert!(!rt.is_job_pending());
}

#[test]
fn shared_dependency_loads_and_evaluates_once_after_capture() {
    let (rt, context, trace) = fixture(&[
        (
            "dep.js",
            "import {value} from './leaf.js'; mark('dep'); export function read() { return value; }",
        ),
        ("leaf.js", "mark('leaf'); export const value = 7;"),
    ]);
    context.with(|ctx| {
        let entry = Module::declare(ctx.clone(), "entry.js", "import {read} from './dep.js'; import {value} from './leaf.js'; mark('entry'); export function home() { return read() + value; }").unwrap();
        assert_eq!(*trace.borrow(), ["load:dep.js", "load:leaf.js"]);
        let saved = install_capture(&ctx, &entry, &["home"], &trace);
        root(&ctx).eval().unwrap().1.result::<()>().unwrap().unwrap();
        assert_eq!(*trace.borrow(), ["load:dep.js", "load:leaf.js", "capture:start", "capture:done", "body:leaf", "body:dep", "body:entry"]);
        assert!(identities_match(&entry, &saved));
        assert_eq!(saved.borrow()[0].1.call::<_, i32>(()).unwrap(), 14);
        // Re-evaluation is cached within this realm, never a cross-call policy.
        entry.eval().unwrap().1.result::<()>().unwrap().unwrap();
        assert_eq!(trace.borrow().iter().filter(|e| *e == "body:leaf").count(), 1);
        clear_callback(&ctx);
    });
    assert!(!rt.is_job_pending());
}

#[test]
fn initialization_jobs_are_detected_and_abandoned_without_pumping() {
    let (rt, context, trace) = fixture(&[]);
    context.with(|ctx| {
        let entry = Module::declare(ctx.clone(), "entry.js", "mark('entry'); Promise.resolve().then(() => mark('forbidden-job')); export function home() {}").unwrap();
        let saved = install_capture(&ctx, &entry, &["home"], &trace);
        root(&ctx).eval().unwrap().1.result::<()>().unwrap().unwrap();
        assert!(identities_match(&entry, &saved));
        assert_eq!(*trace.borrow(), ["capture:start", "capture:done", "body:entry"]);
        clear_callback(&ctx);
    });
    assert!(rt.is_job_pending()); // Reject initialization; never execute_pending_job.
    drop(context);
    drop(rt);
    assert_eq!(
        *trace.borrow(),
        ["capture:start", "capture:done", "body:entry"]
    );
}

#[test]
fn inert_promise_and_source_throw_keep_initialization_synchronous() {
    for (body, expected) in [
        ("new Promise(() => {});", PromiseState::Resolved),
        ("throw new Error('initialization');", PromiseState::Rejected),
    ] {
        let (rt, context, trace) = fixture(&[]);
        context.with(|ctx| {
            let entry = Module::declare(
                ctx.clone(),
                "entry.js",
                format!("mark('entry'); {body} export function home() {{}}"),
            )
            .unwrap();
            let saved = install_capture(&ctx, &entry, &["home"], &trace);
            let (_, promise) = root(&ctx).eval().unwrap();
            assert_eq!(promise.state(), expected);
            assert_eq!(saved.borrow().len(), 1);
            assert_eq!(
                *trace.borrow(),
                ["capture:start", "capture:done", "body:entry"]
            );
            clear_callback(&ctx);
        });
        assert!(!rt.is_job_pending());
    }
}

#[test]
fn source_cannot_import_private_capture_module() {
    for name in ["@host/capture", "./@host/capture", "./dir/../@host/capture"] {
        let (rt, context, trace) = fixture(&[]);
        context.with(|ctx| {
            let text = format!("import '{name}'; mark('entry'); export function home() {{}}");
            let entry = Module::declare(ctx.clone(), "entry.js", text);
            assert!(entry.is_err());
            assert!(ctx.catch().is_error());
            assert!(trace.borrow().is_empty());
        });
        assert!(!rt.is_job_pending());
    }
}

#[test]
fn lexical_values_appear_only_after_evaluation_negative_control() {
    let (rt, context, trace) = fixture(&[]);
    context.with(|ctx| {
        let entry = Module::declare(ctx.clone(), "entry.js", "mark('entry'); export let home = function original() { return 'original'; }; home = function replacement() { return 'replacement'; };").unwrap();
        assert!(trace.borrow().is_empty());
        let (entry, promise) = entry.eval().unwrap();
        promise.result::<()>().unwrap().unwrap();
        assert_eq!(entry.get::<_, Function>("home").unwrap().call::<_, String>(()).unwrap(), "replacement");
        let (number, promise) = Module::declare(ctx.clone(), "number.js", "export const home = 42;").unwrap().eval().unwrap();
        promise.result::<()>().unwrap().unwrap();
        assert_eq!(number.get::<_, i32>("home").unwrap(), 42);
        assert!(number.get::<_, Function>("home").is_err());
        let (class, promise) = Module::declare(ctx, "class.js", "export class home {}").unwrap().eval().unwrap();
        promise.result::<()>().unwrap().unwrap();
        // A function-shaped value alone also does not exclude class constructors.
        assert!(class.get::<_, Value>("home").unwrap().is_function());
    });
    assert!(!rt.is_job_pending());
}
