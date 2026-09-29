use super::*;
use rquickjs::{Function, Module, Object, Promise, Value, promise::PromiseState};

fn interrupted(source: &str, reason: Reason) {
    let (rt, context) = realm();
    let weak = rt.weak();
    let control = Rc::new(Control::default());
    let effects = Rc::new(Cell::new(0));
    install(&rt, control.clone(), 2, reason);
    context.with(|ctx| {
        let count = effects.clone();
        ctx.globals()
            .set(
                "mark",
                Function::new(ctx.clone(), move || count.set(count.get() + 1)).unwrap(),
            )
            .unwrap();
        let operation: Function = ctx.eval(source).unwrap();
        control.armed.set(true);
        assert!(operation.call::<_, Value>(()).is_err());
        assert_eq!(control.observed.get(), Some(reason), "{source}");
        assert_eq!(control.callbacks.get(), 2);
        clear_exception(&ctx);
        // No subsequent source evaluation/job, including to inspect marker state.
    });
    drop(context);
    drop(rt);
    assert!(weak.try_ref().is_none());
    assert_eq!(effects.get(), 0);
    healthy();
}

#[test]
fn cpu_only_loops_and_finite_work_interrupt_for_three_trusted_reasons() {
    for reason in [Reason::Cancelled, Reason::Timeout, Reason::ResourceLimit] {
        for source in [
            "() => { while (true) {} }",
            "() => { for (;;) {} }",
            "() => { let x=0; for(let i=0;i<1e9;i++) x+=i; return x; }",
            "() => { function f(n) { return n<2 ? n : f(n-1)+f(n-2); } for (;;) f(12); }",
            "() => { for(let i=0;i<1e8;i++) { const o={s:String(i).repeat(8)}; Math.sqrt(o.s.length*i); } }",
        ] {
            interrupted(source, reason);
        }
    }
}

#[test]
fn catch_finally_and_queued_work_cannot_escape_cpu_interrupt() {
    interrupted(
        "() => { Promise.resolve().then(mark); try { for (;;) {} } catch(e) { mark(); return 42; } finally { mark(); } }",
        Reason::Cancelled,
    );
    interrupted(
        "() => { for (;;) { try { for (;;) {} } catch(e) { mark(); } } }",
        Reason::ResourceLimit,
    );
}

#[test]
fn module_initialization_interrupts_in_both_host_phases() {
    #[derive(Debug, PartialEq)]
    enum PhaseOutcome {
        LoadingInterrupted,
        CallStopped(Reason),
        Ready,
        Called,
    }
    for load in [true, false] {
        let (rt, context) = realm();
        let weak = rt.weak();
        let control = Rc::new(Control::default());
        let calls = Rc::new(Cell::new(0));
        install(&rt, control.clone(), 2, Reason::Timeout);
        let outcome = context.with(|ctx| {
            let count = calls.clone();
            ctx.globals()
                .set(
                    "mark",
                    Function::new(ctx.clone(), move || count.set(count.get() + 1)).unwrap(),
                )
                .unwrap();
            let module = Module::declare(
                ctx.clone(),
                "entry.js",
                "while(true) {} export function home() { mark(); return 42; }",
            )
            .unwrap();
            control.armed.set(true);
            let result = module.eval();
            let outcome = if let Some(reason) = control.observed.get() {
                if load {
                    PhaseOutcome::LoadingInterrupted
                } else {
                    PhaseOutcome::CallStopped(reason)
                }
            } else {
                let (module, promise) = result.unwrap();
                assert_eq!(promise.state(), PromiseState::Resolved);
                if load {
                    PhaseOutcome::Ready
                } else {
                    module
                        .get::<_, Function>("home")
                        .unwrap()
                        .call::<_, i32>(())
                        .unwrap();
                    PhaseOutcome::Called
                }
            };
            clear_exception(&ctx);
            outcome
        });
        assert_eq!(
            outcome,
            if load {
                PhaseOutcome::LoadingInterrupted
            } else {
                PhaseOutcome::CallStopped(Reason::Timeout)
            }
        );
        assert_eq!(calls.get(), 0);
        drop(context);
        drop(rt);
        assert!(weak.try_ref().is_none());
        healthy();
    }
}

#[test]
fn promise_reaction_and_after_await_cpu_work_interrupts() {
    for source in [
        "() => Promise.resolve().then(() => { for (;;) {} })",
        "async () => { await 0; for (;;) {} }",
    ] {
        let (rt, context) = realm();
        let control = Rc::new(Control::default());
        install(&rt, control.clone(), 2, Reason::ResourceLimit);
        context.with(|ctx| {
            let op: Function = ctx.eval(source).unwrap();
            control.armed.set(true);
            let promise: Promise = op.call(()).unwrap();
            for _ in 0..8 {
                if control.observed.get().is_some() {
                    break;
                }
                let _ = ctx.execute_pending_job();
            }
            assert_eq!(control.observed.get(), Some(Reason::ResourceLimit));
            assert_ne!(promise.state(), PromiseState::Resolved);
            clear_exception(&ctx);
        });
        drop(context);
        drop(rt);
        healthy();
    }
}

#[test]
fn job_count_and_empty_queue_need_host_checks() {
    for source in [
        "new Promise(() => {})",
        "new Promise(() => { function again() { Promise.resolve().then(again); } again(); })",
        "(async function again() { await 0; return again(); })()",
    ] {
        let (rt, context) = realm();
        let weak = rt.weak();
        context.with(|ctx| {
            let promise: Promise = ctx.eval(source).unwrap();
            for _ in 0..32 {
                if !ctx.execute_pending_job() {
                    break;
                }
            }
            assert_eq!(promise.state(), PromiseState::Pending);
            // Host budget/cancel boundary, never Promise::finish or a final drain.
        });
        drop(context);
        drop(rt);
        assert!(weak.try_ref().is_none());
        healthy();
    }
}

#[test]
fn runtime_heap_limit_returns_control_for_retained_allocations() {
    for body in [
        "let a=[]; for(;;) a.push({x: a.length, y: 'value'});",
        "let a=[]; for(;;) a.push(new Array(1024).fill(42));",
        "return 'x'.repeat(32*1024*1024);",
        "let a={}; for(;;) a={next:a};",
        "let a=[]; for(let i=0;;i++) a.push(() => i);",
        "const a=[]; let p=Promise.resolve(); for(;;) { p=p.then(() => 42); a.push(p); }",
        "let m=new Map(); for(let i=0;;i++) m.set(i, {});",
        "let s=new Set(); for(;;) s.add({});",
    ] {
        eprintln!("memory fixture: {body}");
        let (rt, context) = realm();
        let weak = rt.weak();
        let guard = Rc::new(Control::default());
        install(&rt, guard.clone(), 100, Reason::ResourceLimit);
        context.with(|ctx| {
            let op: Function = ctx.eval(format!("() => {{ {body} }}")).unwrap();
            guard.armed.set(true);
            assert!(op.call::<_, Value>(()).is_err(), "{body}");
            assert_eq!(
                guard.observed.get(),
                None,
                "CPU safeguard fired before heap rejection: {body}"
            );
            clear_exception(&ctx);
        });
        // Aggregate engine accounting, not RSS or a trusted rejection latch.
        let usage = rt.memory_usage();
        assert!(usage.malloc_size > 0);
        assert!(usage.malloc_size < (HEAP + 64 * 1024) as i64);
        drop(context);
        drop(rt);
        assert!(weak.try_ref().is_none());
        healthy();
    }
}

#[test]
fn default_heap_rejection_is_catchable_and_not_a_host_reason_latch() {
    let (rt, context) = realm();
    let control = Rc::new(Control::default());
    install(&rt, control.clone(), usize::MAX, Reason::ResourceLimit);
    context.with(|ctx| {
        let op: Function = ctx
            .eval("() => { try { return 'x'.repeat(32*1024*1024); } catch(e) { return 42; } }")
            .unwrap();
        control.armed.set(true);
        assert_eq!(op.call::<_, i32>(()).unwrap(), 42);
        assert_eq!(control.observed.get(), None);
    });
    drop(context);
    drop(rt);
    healthy();
}

#[test]
fn stack_limit_is_recoverable_but_rangeerror_is_not_authentication() {
    for source in [
        "() => { function f() { return f()+1; } return f(); }",
        "() => { const a=[]; for(let i=0;i<1000;i++) a.push(() => a[i+1]()); return a[0](); }",
    ] {
        let (rt, context) = realm();
        context.with(|ctx| {
            let op: Function = ctx.eval(source).unwrap();
            assert!(op.call::<_, Value>(()).is_err());
            clear_exception(&ctx);
        });
        drop(context);
        drop(rt);
        healthy();
    }
    let (rt, context) = realm();
    context.with(|ctx| {
        // Both yield a normal RangeError and can be converted into success by source.
        for source in [
            "(() => { try { (function f(){f()})(); } catch(e) { return e instanceof RangeError; } })()",
            "(() => { try { throw new RangeError('Maximum call stack size exceeded'); } catch(e) { return e instanceof RangeError; } })()",
        ] {
            assert!(ctx.eval::<bool, _>(source).unwrap());
        }
    });
    drop(context);
    drop(rt);
    healthy();
}

#[test]
fn regexp_matching_has_its_own_effective_interrupt_checkpoints() {
    for source in [
        "() => /^(a+)+$/.test('a'.repeat(28)+'!')",
        "() => /^(a|aa)+$/.test('a'.repeat(36)+'!')",
        "() => /^(?:a?){1000000000}$/.test('')",
        "() => { for(;;) /a+/.test('a'.repeat(100)); }",
    ] {
        interrupted(source, Reason::Timeout);
    }
}

#[test]
fn ordinary_exception_text_cannot_forge_trusted_termination() {
    let (rt, context) = realm();
    let control = Rc::new(Control::default());
    install(&rt, control.clone(), 2, Reason::Cancelled);
    context.with(|ctx| {
        let op: Function = ctx
            .eval(
                "() => { throw {code:'CANCELLED', message:'interrupted', name:'InternalError'}; }",
            )
            .unwrap();
        control.armed.set(true);
        assert!(op.call::<_, Value>(()).is_err());
        clear_exception(&ctx);
        assert_eq!(control.observed.get(), None);
    });
    drop(context);
    drop(rt);
}

#[test]
fn candidate_and_settled_promise_are_discarded_on_host_observation() {
    for asynchronous in [false, true] {
        let (rt, context) = realm();
        let weak = rt.weak();
        let state = Rc::new(Control::default());
        let candidate = context.with(|ctx| {
            let value: Object = if asynchronous {
                let p: Promise = ctx.eval("Promise.resolve({ok:true})").unwrap();
                assert_eq!(p.state(), PromiseState::Resolved);
                p.result::<Object>().unwrap().unwrap()
            } else {
                ctx.eval("({ok:true})").unwrap()
            };
            value.get::<_, bool>("ok").unwrap() // fixed fixture only, not a converter
        });
        // Delivery closes here; no more JS execution. Simulated host observation
        // during conversion/cleanup overrides the already computed candidate.
        state.observed.set(Some(if asynchronous {
            Reason::Cancelled
        } else {
            Reason::Timeout
        }));
        drop(context);
        drop(rt);
        assert!(weak.try_ref().is_none());
        let terminal = match state.observed.get() {
            Some(reason) => Err(reason),
            None => Ok(candidate),
        };
        assert_eq!(
            terminal,
            Err(if asynchronous {
                Reason::Cancelled
            } else {
                Reason::Timeout
            })
        );
        healthy();
    }
}
