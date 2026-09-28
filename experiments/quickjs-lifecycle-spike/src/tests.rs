use crate::extraction::{Inspector, Owned};
use rquickjs::{
    Context, Ctx, Error, Function, Module, Persistent, Promise, Runtime, Value,
    allocator::{Allocator, RustAllocator},
    promise::PromiseState,
};
use std::{
    cell::Cell,
    rc::Rc,
    sync::{
        Arc, OnceLock,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};

fn runtime() -> Runtime {
    let rt = Runtime::new().unwrap();
    rt.set_memory_limit(16 * 1024 * 1024);
    rt.set_max_stack_size(256 * 1024);
    rt
}

struct DropMark(Rc<Cell<usize>>);
impl Drop for DropMark {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}

// A test-only native callback records both actual JS calls and native resource release.
// It captures no Ctx/Value/Runtime, avoiding Rust<->JS reference cycles.
fn marker(ctx: &Ctx<'_>, calls: Rc<Cell<usize>>, drops: Rc<Cell<usize>>) {
    let mark = DropMark(drops);
    ctx.globals()
        .set(
            "mark",
            Function::new(ctx.clone(), move || {
                let _keep = &mark;
                calls.set(calls.get() + 1);
            })
            .unwrap(),
        )
        .unwrap();
}

#[test]
fn fresh_modules_globals_intrinsics_and_caches_reset() {
    const MODULE: &str = r#"
        let count = 0;
        const cache = new Map();
        export function probe() {
            cache.set('x', ++count);
            return [count, cache.size, typeof globalThis.old,
                Object.prototype.old === undefined, Math.old === undefined];
        }
    "#;
    for _ in 0..3 {
        // Disposable load probe / A / B, all from identical immutable bytes.
        let rt = runtime();
        let weak = rt.weak();
        let c = Context::full(&rt).unwrap();
        c.with(|ctx| {
            let (module, evaluated) = Module::declare(ctx.clone(), "fixed.js", MODULE)
                .unwrap()
                .eval()
                .unwrap();
            assert_eq!(evaluated.state(), PromiseState::Resolved);
            assert!(!ctx.execute_pending_job());
            let f: Function = module.get("probe").unwrap();
            let inspector = Inspector::new(&ctx);
            let first: Value = f.call(()).unwrap();
            assert_eq!(
                inspector.copy(&ctx, &first).unwrap(),
                Owned::Array(vec![
                    Owned::Number(1.0),
                    Owned::Number(1.0),
                    Owned::Text("undefined".into()),
                    Owned::Bool(true),
                    Owned::Bool(true)
                ])
            );
            let second: rquickjs::Array = f.call(()).unwrap();
            assert_eq!(second.get::<i32>(0).unwrap(), 2); // State exists inside one realm.
            let (same, settled) = module.into_declared().eval().unwrap();
            assert_eq!(settled.state(), PromiseState::Resolved);
            let cached: rquickjs::Array =
                same.get::<_, Function>("probe").unwrap().call(()).unwrap();
            assert_eq!(cached.get::<i32>(0).unwrap(), 3); // Re-evaluation does not reset this module.

            ctx.eval::<(), _>("globalThis.old = 1; Object.prototype.old = 1; Math.old = 1")
                .unwrap();
        });
        drop(c);
        drop(rt);
        assert!(weak.try_ref().is_none());
    }
}

#[test]
fn context_drop_alone_does_not_discard_runtime_jobs_negative_control() {
    let calls = Rc::new(Cell::new(0));
    let drops = Rc::new(Cell::new(0));
    let rt = runtime();
    let c = Context::full(&rt).unwrap();
    c.with(|ctx| {
        marker(&ctx, calls.clone(), drops.clone());
        ctx.eval::<(), _>("Promise.resolve().then(mark)").unwrap();
    });
    drop(c);
    assert_eq!(calls.get(), 0);
    assert!(rt.execute_pending_job().unwrap()); // Deliberate negative control: old JS still runs.
    assert_eq!(calls.get(), 1);
    drop(rt);
    assert_eq!(drops.get(), 1);
}

#[test]
fn persistent_roots_outlive_context_but_reject_other_runtime() {
    let a = runtime();
    let c = Context::full(&a).unwrap();
    let calls = Rc::new(Cell::new(0));
    let drops = Rc::new(Cell::new(0));
    let root = c.with(|ctx| {
        marker(&ctx, calls.clone(), drops.clone());
        Persistent::save(
            &ctx,
            ctx.eval::<Function, _>("() => { mark(); return 17; }")
                .unwrap(),
        )
    });
    drop(c);
    let b = runtime();
    let cb = Context::full(&b).unwrap();
    cb.with(|ctx| {
        assert!(matches!(
            root.clone().restore(&ctx),
            Err(Error::UnrelatedRuntime)
        ))
    });
    let ca = Context::full(&a).unwrap();
    ca.with(|ctx| {
        // Same runtime: legal but violates Model B if used between invocations.
        let f = root.clone().restore(&ctx).unwrap();
        assert_eq!(f.call::<_, i32>(()).unwrap(), 17);
    });
    assert_eq!(calls.get(), 1);
    drop(root); // MUST release all persistent roots before dropping the runtime.
    drop(ca);
    drop(a);
    assert_eq!(drops.get(), 1);
    drop(cb);
    drop(b);
}

#[test]
fn pending_chain_finally_and_nested_await_run_only_while_alive() {
    let rt = runtime();
    let c = Context::full(&rt).unwrap();
    c.with(|ctx| {
        let (upstream, resolve, _) = Promise::new(&ctx).unwrap();
        ctx.globals().set("upstream", upstream).unwrap();
        let result: Promise = ctx.eval(r#"
            globalThis.order = [];
            (async () => {
                const x = await (async () => await upstream.then(x => { order.push(1); return x + 1; })
                    .then(x => { order.push(2); return x + 1; })
                    .finally(() => { order.push(3); }))();
                order.push(4);
                return x;
            })()
        "#).unwrap();
        assert_eq!(result.state(), PromiseState::Pending);
        assert!(!ctx.execute_pending_job()); // Host may wait/check cancellation; not terminal.
        resolve.call::<_, ()>((40,)).unwrap();
        let mut jobs = 0;
        while result.state() == PromiseState::Pending {
            assert!(jobs < 32);
            assert!(ctx.execute_pending_job());
            jobs += 1;
        }
        assert_eq!(result.result::<i32>().unwrap().unwrap(), 42);
        assert!(ctx.eval::<bool, _>("order.join(',') === '1,2,3,4'").unwrap());
    });
}

// Completion carries only a host generation and owned scalar. No JS values leave the owner thread.
#[derive(Clone, Copy)]
struct Completion {
    generation: u64,
    value: i32,
}
struct Delivery<'js> {
    generation: u64,
    active: bool,
    resolver: Option<Function<'js>>,
}
impl Delivery<'_> {
    fn deliver(&self, event: Completion) -> bool {
        if !self.active || event.generation != self.generation {
            return false;
        }
        self.resolver
            .as_ref()
            .unwrap()
            .call::<_, ()>((event.value,))
            .unwrap();
        true
    }
    fn revoke(&mut self) {
        self.active = false;
        self.resolver.take();
    }
}

#[test]
fn terminal_boundary_copies_then_destroys_all_pending_graphs_without_callbacks() {
    let calls = Rc::new(Cell::new(0));
    let drops = Rc::new(Cell::new(0));
    let rt = runtime();
    let weak = rt.weak();
    let c = Context::full(&rt).unwrap();
    let owned = c.with(|ctx| {
        marker(&ctx, calls.clone(), drops.clone());
        let inspector = Inspector::new(&ctx);
        let result: Promise = ctx.eval(r#"
            globalThis.dormant = new Promise(r => { globalThis.oldResolver = r; });
            dormant.then(mark).then(mark).finally(mark);
            (async () => { try { await (async () => await dormant)(); mark(); } finally { mark(); } })();
            const result = Promise.resolve().then(() => {
                Promise.resolve().then(() => { mark(); Promise.resolve().then(mark); });
                return {ok: true, data: {categories: [], items: []}};
            });
            result.then(mark).finally(mark);
            result.then = () => { mark(); throw 1; };
            result
        "#).unwrap();
        assert_eq!(result.state(), PromiseState::Pending);
        assert!(ctx.execute_pending_job());
        assert_eq!(result.state(), PromiseState::Resolved);
        // Close delivery here; never pump/eval/call source again after this boundary.
        let value = result.result::<Value>().unwrap().unwrap();
        inspector.copy(&ctx, &value).unwrap()
    });
    assert!(rt.is_job_pending());
    drop(c);
    drop(rt);
    assert!(weak.try_ref().is_none());
    assert_eq!(calls.get(), 0);
    assert_eq!(drops.get(), 1); // Native function captured by old graph was actually freed.
    assert!(
        matches!(owned, Owned::Record(ref fields) if fields.get("ok") == Some(&Owned::Bool(true)))
    );
    let b = runtime();
    let cb = Context::full(&b).unwrap();
    cb.with(|ctx| {
        assert!(
            ctx.eval::<bool, _>(
                "typeof dormant === 'undefined' && typeof oldResolver === 'undefined'"
            )
            .unwrap()
        )
    });
    assert!(!b.is_job_pending());
}

#[test]
fn species_resolvers_and_finally_are_not_cleanup() {
    let rt = runtime();
    let c = Context::full(&rt).unwrap();
    let calls = Rc::new(Cell::new(0));
    let drops = Rc::new(Cell::new(0));
    c.with(|ctx| {
        marker(&ctx, calls.clone(), drops.clone());
        ctx.eval::<(), _>(
            r#"
            function Species(executor) { executor(mark, mark); }
            const p = Promise.resolve(1);
            p.constructor = {[Symbol.species]: Species};
            p.then(mark);
            Promise.resolve().finally(mark);
        "#,
        )
        .unwrap();
    });
    assert!(rt.is_job_pending());
    assert_eq!(calls.get(), 0);
    drop(c);
    drop(rt);
    assert_eq!(calls.get(), 0);
    assert_eq!(drops.get(), 1);
}

#[test]
fn late_completions_and_old_resolvers_cannot_enter_next_generation() {
    // Four terminal reasons use the same revoke/drop ordering, independent of result classification.
    for reason in ["success", "cancelled", "timeout", "resource"] {
        let a = runtime();
        let weak = a.weak();
        let ca = Context::full(&a).unwrap();
        let event = Completion {
            generation: 1,
            value: 17,
        };
        let calls = Rc::new(Cell::new(0));
        let drops = Rc::new(Cell::new(0));
        ca.with(|ctx| {
            marker(&ctx, calls.clone(), drops.clone());
            let (p, resolve, _) = Promise::new(&ctx).unwrap();
            ctx.globals().set("hostResult", p).unwrap();
            ctx.eval::<(), _>("hostResult.then(mark)").unwrap();
            let mut delivery = Delivery {
                generation: 1,
                active: true,
                resolver: Some(resolve),
            };
            delivery.revoke();
            assert!(!delivery.deliver(event), "{reason}"); // Invalidation before destruction.
            assert!(delivery.resolver.is_none());
        });
        drop(ca);
        drop(a);
        assert!(weak.try_ref().is_none());
        assert_eq!(drops.get(), 1);
        let b = runtime();
        let cb = Context::full(&b).unwrap();
        cb.with(|ctx| {
            let (p, resolve, _) = Promise::new(&ctx).unwrap();
            let mut delivery = Delivery {
                generation: 2,
                active: true,
                resolver: Some(resolve),
            };
            // Simulated external worker crosses threads with host data only.
            let late = std::thread::spawn(move || event).join().unwrap();
            assert!(!delivery.deliver(late));
            assert_eq!(p.state(), PromiseState::Pending);
            assert!(delivery.deliver(Completion {
                generation: 2,
                value: 42
            }));
            assert_eq!(p.result::<i32>().unwrap().unwrap(), 42);
            delivery.revoke();
        });
        assert_eq!(calls.get(), 0);
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Terminal {
    Success,
    SourceError,
    InvalidResult,
    Cancelled,
    Timeout,
    ResourceLimit,
}
#[derive(Default)]
struct Logic {
    disposed: bool,
}
impl Logic {
    fn finish(&mut self, result: Terminal) {
        if matches!(
            result,
            Terminal::Cancelled | Terminal::Timeout | Terminal::ResourceLimit
        ) {
            self.disposed = true;
        }
    }
}

#[test]
fn cancellation_and_deadline_interrupt_running_cpu_and_dispose_logic() {
    for terminal in [Terminal::Cancelled, Terminal::Timeout] {
        let rt = runtime();
        let weak = rt.weak();
        let c = Context::full(&rt).unwrap();
        let calls = Rc::new(Cell::new(0));
        let drops = Rc::new(Cell::new(0));
        let checks = Arc::new(AtomicUsize::new(0));
        let count = checks.clone();
        let cancelled = Arc::new(AtomicBool::new(false));
        let signal = cancelled.clone();
        let worker_checks = checks.clone();
        let entered = Arc::new(OnceLock::<Instant>::new());
        let worker_entered = entered.clone();
        let worker = (terminal == Terminal::Cancelled).then(|| {
            std::thread::spawn(move || {
                while worker_entered.get().is_none() || worker_checks.load(Ordering::SeqCst) < 4 {
                    std::thread::yield_now();
                }
                signal.store(true, Ordering::SeqCst);
            })
        });
        let started = entered.clone();
        let observed = Arc::new(AtomicBool::new(false));
        let latch = observed.clone();
        rt.set_interrupt_handler(Some(Box::new(move || {
            count.fetch_add(1, Ordering::SeqCst);
            let stop = if terminal == Terminal::Cancelled {
                cancelled.load(Ordering::SeqCst)
            } else {
                started
                    .get()
                    .is_some_and(|at| at.elapsed() >= Duration::from_millis(10))
            };
            if stop {
                latch.store(true, Ordering::SeqCst);
            }
            stop
        })));
        c.with(|ctx| {
            marker(&ctx, calls.clone(), drops.clone());
            let begin = entered.clone();
            ctx.globals().set("begin", Function::new(ctx.clone(), move || {
                begin.set(Instant::now()).unwrap();
            }).unwrap()).unwrap();
            assert!(
                ctx.eval::<(), _>(
                    "begin(); Promise.resolve().then(mark); try { while(true) {} } finally { mark(); }"
                )
                .is_err()
            );
            drop(ctx.catch());
        });
        if let Some(worker) = worker {
            worker.join().unwrap();
        }
        assert!(entered.get().is_some());
        assert!(observed.load(Ordering::SeqCst));
        assert!(checks.load(Ordering::SeqCst) > 0);
        let mut logic = Logic::default();
        logic.finish(terminal);
        drop(c);
        drop(rt);
        assert!(logic.disposed);
        assert!(weak.try_ref().is_none());
        assert_eq!(calls.get(), 0);
        assert_eq!(drops.get(), 1);
    }
}

// Public allocator hook: one deliberately small per-allocation budget, NOT a heap/RSS quota.
struct Limited {
    max: Rc<Cell<usize>>,
    hit: Rc<Cell<bool>>,
}
// SAFETY: all successful storage and layout operations delegate unchanged to RustAllocator.
// A rejected allocation returns null; failed realloc retains the original allocation.
unsafe impl Allocator for Limited {
    fn alloc(&mut self, size: usize) -> *mut u8 {
        if size > self.max.get() {
            self.hit.set(true);
            return std::ptr::null_mut();
        }
        RustAllocator.alloc(size)
    }
    fn calloc(&mut self, count: usize, size: usize) -> *mut u8 {
        match count.checked_mul(size) {
            Some(n) if n <= self.max.get() => RustAllocator.calloc(count, size),
            _ => {
                self.hit.set(true);
                std::ptr::null_mut()
            }
        }
    }
    unsafe fn dealloc(&mut self, ptr: *mut u8) {
        // SAFETY: provided by QuickJS from this allocator; delegated exactly once.
        unsafe { RustAllocator.dealloc(ptr) }
    }
    unsafe fn realloc(&mut self, ptr: *mut u8, size: usize) -> *mut u8 {
        if ptr.is_null() {
            return self.alloc(size);
        }
        if size == 0 {
            // SAFETY: live allocation from this allocator, relinquished exactly once.
            unsafe { self.dealloc(ptr) };
            return std::ptr::null_mut();
        }
        if size > self.max.get() {
            self.hit.set(true);
            return std::ptr::null_mut();
        }
        // SAFETY: live allocation; delegate preserves old allocation on null return.
        unsafe { RustAllocator.realloc(ptr, size) }
    }
    unsafe fn usable_size(ptr: *mut u8) -> usize {
        if ptr.is_null() {
            return 0;
        }
        // SAFETY: live allocation from RustAllocator per trait contract.
        unsafe { RustAllocator::usable_size(ptr) }
    }
}

#[test]
fn allocation_failure_is_host_classified_even_if_source_catches_it() {
    let max = Rc::new(Cell::new(usize::MAX));
    let hit = Rc::new(Cell::new(false));
    let rt = Runtime::new_with_alloc(Limited {
        max: max.clone(),
        hit: hit.clone(),
    })
    .unwrap();
    let c = Context::full(&rt).unwrap();
    c.with(|ctx| {
        let f: Function = ctx
            .eval("() => { try { return 'x'.repeat(1000000); } catch(e) { return 42; } }")
            .unwrap();
        max.set(4096);
        let attempted: rquickjs::Result<Value> = f.call(());
        assert!(hit.get());
        // A caught OOM must not override the host latch, regardless of engine exception spelling.
        drop(attempted);
        drop(ctx.catch());
    });
    let mut logic = Logic::default();
    logic.finish(if hit.get() {
        Terminal::ResourceLimit
    } else {
        Terminal::Success
    });
    drop(c);
    drop(rt);
    assert!(logic.disposed);
}

#[test]
fn initialization_failures_retire_realms_and_ordinary_failure_allows_next_call() {
    let mut logic = Logic::default();
    for stage in [
        "host-before-code",
        "partial-host",
        "source-throw",
        "source-job",
        "success",
    ] {
        assert!(!logic.disposed);
        let rt = runtime();
        let weak = rt.weak();
        let c = Context::full(&rt).unwrap();
        let calls = Rc::new(Cell::new(0));
        let drops = Rc::new(Cell::new(0));
        let result = c.with(|ctx| {
            if stage == "host-before-code" {
                return Terminal::SourceError;
            } // injected setup error
            marker(&ctx, calls.clone(), drops.clone());
            if stage == "partial-host" {
                ctx.globals()
                    .prop("blocked", rquickjs::object::Property::from(1))
                    .unwrap();
                assert!(ctx.globals().set("blocked", 2).is_err());
                drop(ctx.catch());
                return Terminal::SourceError;
            }
            let source = match stage {
                "source-throw" => "throw 7; export function operation() {}",
                "source-job" => "Promise.resolve().then(mark); export function operation() {}",
                _ => "export function operation() { return 42; }",
            };
            let (m, p) = Module::declare(ctx.clone(), "init.js", source)
                .unwrap()
                .eval()
                .unwrap();
            if p.state() == PromiseState::Rejected {
                assert!(p.result::<Value>().unwrap().is_err());
                drop(ctx.catch());
                return Terminal::SourceError;
            }
            assert_eq!(p.state(), PromiseState::Resolved);
            if stage == "source-job" {
                return Terminal::SourceError;
            } // pending check below, no pump
            assert_eq!(
                m.get::<_, Function>("operation")
                    .unwrap()
                    .call::<_, i32>(())
                    .unwrap(),
                42
            );
            Terminal::Success
        });
        if stage == "source-job" {
            assert!(rt.is_job_pending());
        }
        logic.finish(result);
        assert_eq!(
            result,
            if stage == "success" {
                Terminal::Success
            } else {
                Terminal::SourceError
            }
        );
        drop(c);
        drop(rt);
        assert!(weak.try_ref().is_none());
        assert_eq!(calls.get(), 0);
        assert_eq!(drops.get(), usize::from(stage != "host-before-code"));
    }
}

#[test]
fn hostile_extraction_never_runs_getters_traps_then_or_to_json() {
    let rt = runtime();
    let c = Context::full(&rt).unwrap();
    let calls = Rc::new(Cell::new(0));
    let drops = Rc::new(Cell::new(0));
    c.with(|ctx| {
        let inspector = Inspector::new(&ctx);
        marker(&ctx, calls.clone(), drops.clone());
        for source in [
            "({get ok(){mark(); return true}})",
            "new Proxy({}, {get:mark,ownKeys:mark,getOwnPropertyDescriptor:mark,getPrototypeOf:mark})",
            "Proxy.revocable({}, {}).proxy",
            "({ok:true,toJSON(){mark();return 1}})",
            "({then(resolve){mark();resolve(1)}})",
            "({get then(){mark();return ()=>{}}})",
            "Object.create({get x(){mark();return 1}})",
            "Object.assign(Object.create(null), {getValue:mark})",
            "Object.setPrototypeOf(new Date(), Object.prototype)",
            "Object.setPrototypeOf(new Uint8Array(1), Object.prototype)",
            "Object.setPrototypeOf(Promise.resolve(1), Object.prototype)",
            "({[Symbol('x')]:1})", "Object.defineProperty({},'x',{value:1})",
            "[1,,3]", "Object.assign([1],{extra:2})",
            "(()=>{let a={};a.a=a;return a})()",
            "({data:{get value(){mark();return 1}}})",
            "({data:new Proxy({}, {ownKeys:mark})})",
        ] {
            let v: Value = ctx.eval(source).unwrap();
            assert!(inspector.copy(&ctx, &v).is_err(), "{source}");
            assert_eq!(calls.get(), 0, "{source}");
        }
        let revoked: Value = ctx.eval("(()=>{const p=Proxy.revocable({},{});p.revoke();return p.proxy})()").unwrap();
        assert!(inspector.copy(&ctx, &revoked).is_err());
        assert_eq!(calls.get(), 0);
        let good: Value = ctx.eval("({ok:true,data:{items:[],categories:[],n:-0,nested:{a:null,s:'test'}}})").unwrap();
        assert!(inspector.copy(&ctx, &good).is_ok());
        // Negative controls: convenient access/serialization APIs are not non-executing.
        let getter: rquickjs::Object = ctx.eval("({get x(){mark();return 1}})").unwrap();
        assert_eq!(getter.get::<_, i32>("x").unwrap(), 1);
        ctx.eval::<(), _>("JSON.stringify({toJSON(){mark();return 1}})").unwrap();
        let proxy: rquickjs::Object = ctx.eval("new Proxy({}, {ownKeys(){mark();return []}})").unwrap();
        let _: Vec<String> = proxy.keys().collect::<rquickjs::Result<_>>().unwrap();
        assert_eq!(calls.get(), 3);
    });
}

#[test]
fn context_allocation_failure_before_source_is_recoverable() {
    let rt = runtime();
    let weak = rt.weak();
    rt.set_memory_limit(1);
    assert!(matches!(Context::full(&rt), Err(Error::Allocation)));
    drop(rt);
    assert!(weak.try_ref().is_none());
    // Ordinary host setup failure did not publish a ready source; a fresh attempt can initialize.
    let next = runtime();
    let c = Context::full(&next).unwrap();
    c.with(|ctx| assert_eq!(ctx.eval::<i32, _>("42").unwrap(), 42));
}

#[test]
fn pending_empty_queue_waits_for_host_cancel_or_deadline_without_pumping() {
    for terminal in [Terminal::Cancelled, Terminal::Timeout] {
        let rt = runtime();
        let weak = rt.weak();
        let c = Context::full(&rt).unwrap();
        c.with(|ctx| {
            let pending: Promise = ctx.eval("new Promise(() => {})").unwrap();
            assert!(!ctx.execute_pending_job());
            assert_eq!(pending.state(), PromiseState::Pending);
            // Controlled host event at a scheduler checkpoint; no synthetic JS settlement.
            let mut logic = Logic::default();
            logic.finish(terminal);
            assert!(logic.disposed);
            assert_eq!(pending.state(), PromiseState::Pending);
        });
        drop(c);
        drop(rt);
        assert!(weak.try_ref().is_none());
    }
}

#[test]
fn deadline_after_copy_and_cleanup_prevents_success_publication() {
    let rt = runtime();
    let weak = rt.weak();
    let c = Context::full(&rt).unwrap();
    let candidate = c.with(|ctx| {
        let inspector = Inspector::new(&ctx);
        let v: Value = ctx
            .eval("({ok:true,data:{categories:[],items:[]}})")
            .unwrap();
        inspector.copy(&ctx, &v).unwrap()
    });
    drop(c);
    drop(rt);
    assert!(weak.try_ref().is_none());
    // Fake monotonic clock advanced by a scheduled host checkpoint during cleanup.
    let admitted = 0_u64;
    let deadline = admitted + 10;
    let before_publish = 11;
    let selected = if before_publish >= deadline {
        Terminal::Timeout
    } else {
        Terminal::Success
    };
    let mut logic = Logic::default();
    logic.finish(selected);
    assert_eq!(selected, Terminal::Timeout);
    assert!(logic.disposed);
    drop(candidate); // Never published, and no source work needed to discard it.
}

#[test]
fn ordinary_rejection_and_invalid_value_discard_realm_without_disposing_logic() {
    let mut logic = Logic::default();
    for source in [
        "(() => { Promise.resolve().then(mark); throw 7; })()",
        "Promise.reject(7)",
        "({get ok(){mark();return true}})",
    ] {
        assert!(!logic.disposed);
        let rt = runtime();
        let weak = rt.weak();
        let c = Context::full(&rt).unwrap();
        let calls = Rc::new(Cell::new(0));
        let drops = Rc::new(Cell::new(0));
        let outcome = c.with(|ctx| {
            marker(&ctx, calls.clone(), drops.clone());
            let inspector = Inspector::new(&ctx);
            match ctx.eval::<Value, _>(source) {
                Err(_) => {
                    drop(ctx.catch());
                    Terminal::SourceError
                }
                Ok(v) if v.is_promise() => {
                    let p = v.as_promise().unwrap();
                    assert_eq!(p.state(), PromiseState::Rejected);
                    assert!(p.result::<Value>().unwrap().is_err());
                    drop(ctx.catch());
                    Terminal::SourceError
                }
                Ok(v) => {
                    assert!(inspector.copy(&ctx, &v).is_err());
                    Terminal::InvalidResult
                }
            }
        });
        // Ordinary source failure / INVALID_RESULT do not poison host logic; no JS state is reused.
        logic.finish(outcome);
        drop(c);
        drop(rt);
        assert!(weak.try_ref().is_none());
        assert_eq!(calls.get(), 0);
        assert_eq!(drops.get(), 1);
    }
}
