//! Non-production QuickJS-NG primitive probes, including expected incompatibilities.
//! No source packages, Source API dispatcher, services, or production converter.

#[cfg(test)]
mod tests {
    use rquickjs::promise::PromiseState;
    use rquickjs::{Context, Ctx, Function, Object, Promise, Runtime, Value, qjs};
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    fn runtime() -> Runtime {
        let rt = Runtime::new().unwrap();
        rt.set_memory_limit(16 * 1024 * 1024);
        rt.set_max_stack_size(256 * 1024);
        rt
    }

    #[test]
    fn isolated_contexts_call_and_reuse() {
        // Separate runtimes avoid sharing a heap or job queue between instances.
        let a = runtime();
        let b = runtime();
        let ca = Context::full(&a).unwrap();
        let cb = Context::full(&b).unwrap();
        ca.with(|ctx| {
            ctx.eval::<(), _>("globalThis.counter = 0; Object.prototype.privateMarker = 7")
                .unwrap();
            let f: Function = ctx
                .eval("(x) => { counter++; return x + counter }")
                .unwrap();
            assert_eq!(f.call::<_, i32>((40,)).unwrap(), 41);
            assert_eq!(f.call::<_, i32>((40,)).unwrap(), 42);
        });
        cb.with(|ctx| {
            assert!(
                ctx.eval::<bool, _>(
                    "typeof counter === 'undefined' && !('privateMarker' in Object.prototype)"
                )
                .unwrap()
            )
        });
        drop(ca);
        drop(a);
        cb.with(|ctx| assert_eq!(ctx.eval::<i32, _>("6 * 7").unwrap(), 42));
    }

    #[test]
    fn promise_settlement_without_then_or_extra_job() {
        let rt = runtime();
        let context = Context::full(&rt).unwrap();
        context.with(|ctx| {
            let p: Promise = ctx
                .eval(
                    r#"
                globalThis.order = [];
                const p = Promise.resolve().then(() => {
                    order.push(1);
                    Promise.resolve().then(() => order.push(2));
                    return 42;
                });
                p.then = () => { throw new Error('host must not call then'); };
                p
            "#,
                )
                .unwrap();
            assert_eq!(p.state(), PromiseState::Pending);
            assert!(ctx.execute_pending_job());
            assert_eq!(p.state(), PromiseState::Resolved);
            assert_eq!(p.result::<i32>().unwrap().unwrap(), 42);
            assert_eq!(ctx.eval::<i32, _>("order.length").unwrap(), 1);
            // Terminal selection could happen here, before the second queued job.
            // Deliberately pump it as a diagnostic: it has NOT been revoked.
            assert!(ctx.execute_pending_job());
            assert_eq!(ctx.eval::<i32, _>("order[1]").unwrap(), 2);
        });
    }

    #[test]
    fn pending_promise_host_settlement_and_rejection() {
        let rt = runtime();
        let context = Context::full(&rt).unwrap();
        context.with(|ctx| {
            let never: Promise = ctx.eval("new Promise(() => {})").unwrap();
            assert_eq!(never.state(), PromiseState::Pending);
            assert!(!ctx.execute_pending_job()); // Host may check its clock/cancellation now.
            let (p, resolve, _) = Promise::new(&ctx).unwrap();
            resolve.call::<_, ()>((42,)).unwrap();
            assert_eq!(p.result::<i32>().unwrap().unwrap(), 42);
            let rejected: Promise = ctx.eval("Promise.reject('private diagnostic')").unwrap();
            assert_eq!(rejected.state(), PromiseState::Rejected);
            assert!(rejected.result::<Value>().unwrap().is_err());
            let _private_exception = ctx.catch(); // Never format or publish source diagnostics.
        });
    }

    #[test]
    fn retained_reaction_crosses_invocation_boundary_without_engine_support() {
        let rt = runtime();
        let context = Context::full(&rt).unwrap();
        context.with(|ctx| {
            // Synthetic invocation A succeeds synchronously, with no queued jobs.
            assert_eq!(
                ctx.eval::<i32, _>(
                    r#"
                globalThis.oldRan = 0;
                globalThis.release = undefined;
                globalThis.pending = new Promise(r => { release = r; });
                pending.then(() => { oldRan++; });
                1
            "#
                )
                .unwrap(),
                1
            );
            assert!(!ctx.execute_pending_job());
            // Invocation B resolves A's retained Promise. The old reaction runs.
            ctx.eval::<(), _>("release(0)").unwrap();
            assert!(ctx.execute_pending_job());
            assert_eq!(ctx.eval::<i32, _>("oldRan").unwrap(), 1);
        });
        // This PASS reproduces an RFC incompatibility, not conformance.
    }

    #[test]
    fn retained_await_bypasses_source_then_wrapper() {
        let rt = runtime();
        let context = Context::full(&rt).unwrap();
        context.with(|ctx| {
            ctx.eval::<(), _>(
                r#"
                globalThis.oldRan = 0;
                globalThis.thenCalls = 0;
                globalThis.release = undefined;
                const originalThen = Promise.prototype.then;
                Promise.prototype.then = function(...args) {
                    thenCalls++;
                    return originalThen.apply(this, args);
                };
                const pending = new Promise(r => { release = r; });
                (async () => { await pending; oldRan++; })();
            "#,
            )
            .unwrap();
            assert!(!ctx.execute_pending_job());
            ctx.eval::<(), _>("release(0)").unwrap();
            assert!(ctx.execute_pending_job());
            assert_eq!(ctx.eval::<i32, _>("oldRan").unwrap(), 1);
            assert_eq!(ctx.eval::<i32, _>("thenCalls").unwrap(), 0);
        });
    }

    #[test]
    fn infinite_loop_interrupt_and_discard_prevent_continuation() {
        let rt = runtime();
        let checkpoints = Arc::new(AtomicUsize::new(0));
        let observed = checkpoints.clone();
        rt.set_interrupt_handler(Some(Box::new(move || {
            observed.fetch_add(1, Ordering::SeqCst) >= 4
        })));
        let after = Arc::new(AtomicUsize::new(0));
        let context = Context::full(&rt).unwrap();
        context.with(|ctx| {
            let observed = after.clone();
            ctx.globals()
                .set(
                    "after",
                    Function::new(ctx.clone(), move || {
                        observed.fetch_add(1, Ordering::SeqCst);
                    })
                    .unwrap(),
                )
                .unwrap();
            assert!(
                ctx.eval::<(), _>(
                    "Promise.resolve().then(after); try { while (true) {} } catch (e) { after(); } after();",
                )
                    .is_err()
            );
            let _ = ctx.catch();
        });
        assert!(checkpoints.load(Ordering::SeqCst) >= 5);
        drop(context);
        drop(rt); // No reuse or job draining after interruption.
        assert_eq!(after.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn regexp_work_reaches_interrupt_hook() {
        let rt = runtime();
        let context = Context::full(&rt).unwrap();
        // Compile/setup first: the hook must be reached inside the subsequent regexp work.
        context.with(|ctx| {
            ctx.eval::<(), _>("globalThis.runRegexp = () => /^(a+)+$/.test('a'.repeat(100) + '!')")
                .unwrap()
        });
        let checkpoints = Arc::new(AtomicUsize::new(0));
        let observed = checkpoints.clone();
        rt.set_interrupt_handler(Some(Box::new(move || {
            observed.fetch_add(1, Ordering::SeqCst) >= 4
        })));
        context.with(|ctx| {
            let f: Function = ctx.globals().get("runRegexp").unwrap();
            assert!(f.call::<_, bool>(()).is_err());
            let _ = ctx.catch();
        });
        assert!(checkpoints.load(Ordering::SeqCst) >= 5);
    }

    // Tiny descriptor probe, NOT a recursive converter. Callers use known ordinary objects.
    fn descriptor(ctx: &Ctx<'_>, value: &Value<'_>, key: &std::ffi::CStr) -> (bool, Option<i32>) {
        // SAFETY: ctx/value are live in the same scoped context. Atoms and all three
        // owned descriptor values are released exactly once. Reject Proxy before inspection.
        unsafe {
            assert!(!qjs::JS_IsProxy(value.as_raw()));
            let raw = ctx.as_raw().as_ptr();
            let atom = qjs::JS_NewAtom(raw, key.as_ptr());
            let mut desc = std::mem::MaybeUninit::<qjs::JSPropertyDescriptor>::uninit();
            let found = qjs::JS_GetOwnProperty(raw, desc.as_mut_ptr(), value.as_raw(), atom);
            qjs::JS_FreeAtom(raw, atom);
            assert_eq!(found, 1);
            let desc = desc.assume_init();
            let accessor = desc.flags & qjs::JS_PROP_GETSET as i32 != 0;
            let copied = Value::from_raw(ctx.clone(), desc.value);
            qjs::JS_FreeValue(raw, desc.getter);
            qjs::JS_FreeValue(raw, desc.setter);
            (accessor, copied.as_int())
        }
    }

    #[test]
    fn structural_descriptors_accessors_and_proxy_precheck() {
        let rt = runtime();
        let context = Context::full(&rt).unwrap();
        context.with(|ctx| {
            let plain: Object = ctx
                .eval(
                    r#"
                globalThis.getterCalls = 0;
                ({ answer: 42, get secret() { getterCalls++; return 9; } })
            "#,
                )
                .unwrap();
            let keys: Vec<String> = plain.keys().collect::<Result<_, _>>().unwrap();
            assert_eq!(keys, ["answer", "secret"]);
            assert_eq!(
                descriptor(&ctx, plain.as_value(), c"answer"),
                (false, Some(42))
            );
            assert_eq!(descriptor(&ctx, plain.as_value(), c"secret"), (true, None));
            assert_eq!(ctx.eval::<i32, _>("getterCalls").unwrap(), 0);
            assert_eq!(
                plain.get_prototype(),
                Some(ctx.eval::<Object, _>("Object.prototype").unwrap())
            );
            let proxy: Object = ctx
                .eval(
                    r#"
                globalThis.traps = 0;
                new Proxy({}, {ownKeys() { traps++; return []; }})
            "#,
                )
                .unwrap();
            // SAFETY: JS_IsProxy is a non-executing class check on a live value.
            assert!(unsafe { qjs::JS_IsProxy(proxy.as_value().as_raw()) });
            assert_eq!(ctx.eval::<i32, _>("traps").unwrap(), 0);
            // Deliberately unsafe policy (not memory-unsafe): enumeration invokes a trap.
            let _: Vec<String> = proxy.keys().collect::<Result<_, _>>().unwrap();
            assert_eq!(ctx.eval::<i32, _>("traps").unwrap(), 1);
        });
    }

    #[test]
    fn dynamic_constructor_routes_after_local_bootstrap() {
        let rt = runtime();
        let context = Context::full(&rt).unwrap();
        context.with(|ctx| {
            // Local experiment, not audited production hardening. No source runs before this.
            ctx.eval::<(), _>(
                r#"
                (() => {
                    const deny = function() { throw new EvalError('disabled'); };
                    const constructors = [Function, (async function(){}).constructor,
                        (function*(){}).constructor, (async function*(){}).constructor];
                    for (const c of constructors) {
                        Object.defineProperty(c.prototype, 'constructor', {
                            value: deny, writable: false, configurable: false
                        });
                    }
                    Object.defineProperty(deny, 'prototype', {value: Function.prototype});
                    for (const key of ['eval', 'Function']) Object.defineProperty(globalThis, key, {
                        value: deny, writable: false, configurable: false
                    });
                })();
            "#,
            )
            .unwrap();
            let count: i32 = ctx
                .eval(
                    r#"
                globalThis.compiled = 0;
                const body = 'globalThis.compiled++; return 1';
                const attempts = [
                    () => eval(body), () => (0, eval)(body), () => globalThis.eval(body),
                    () => Function(body), () => new Function(body),
                    () => (function(){}).constructor(body),
                    () => (async function(){}).constructor(body),
                    () => (function*(){}).constructor(body),
                    () => (async function*(){}).constructor(body),
                    () => Object.getPrototypeOf(async () => {}).constructor(body),
                    () => Reflect.construct((function(){}).constructor, [body]),
                    () => ({}).constructor.constructor(body)
                ];
                Reflect.set(globalThis, 'Function', Object);
                Reflect.set(Object.getPrototypeOf(function(){}), 'constructor', Object);
                let denied = 0;
                for (const f of attempts) {
                    try { f(); } catch (e) { if (e instanceof EvalError) denied++; }
                }
                denied
            "#,
                )
                .unwrap();
            assert_eq!(count, 12);
            assert_eq!(ctx.eval::<i32, _>("compiled").unwrap(), 0);
        });
    }

    #[test]
    fn ambient_authority_absent_but_profile_is_not_default() {
        let rt = runtime();
        let context = Context::full(&rt).unwrap();
        context.with(|ctx| {
            assert!(
                ctx.eval::<bool, _>(
                    r#"
                ['process', 'require', 'Buffer', 'fetch', 'XMLHttpRequest', 'window',
                 'document', 'navigator', 'setTimeout', 'Worker', 'console', 'std', 'os']
                .every(k => typeof globalThis[k] === 'undefined')
            "#
                )
                .unwrap()
            );
            // Exact RFC allowlist and stack suppression are NOT provided by Context::full.
            assert!(
                ctx.eval::<bool, _>(
                    r#"
                typeof Date === 'function' && typeof Proxy === 'function' &&
                typeof Math.random === 'function' && typeof ArrayBuffer === 'function' &&
                typeof new Error().stack === 'string'
            "#
                )
                .unwrap()
            );
        });
    }
}
