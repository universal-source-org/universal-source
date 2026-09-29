//! Public API diagnostics, deliberately asserting the blocker rather than a fix.
use super::*;

fn probe(body: &str, config: Config) -> Report {
    assert_eq!(std::env::var("INTEGRATED_ENGINE_MODE").unwrap(), "patched");
    assert_eq!(
        std::env::var("INTEGRATED_ENGINE_SHA").unwrap(),
        "9a926c4ed02517c84ecfbc9d925b6ceb782b3119f6d05a3d58280c66363e1f3f"
    );
    invoke(
        &format!("export function home(input, context) {{ const t=context.services.log; {body} }}"),
        config,
        None,
    )
}

fn retired_and_healthy(r: &Report) {
    assert!(r.destroyed && r.late_rejected == 2, "{r:?}");
    let next = invoke(
        "export function home() { return 42; }",
        Config::default(),
        Some(r.generation),
    );
    assert_eq!(next.outcome, Outcome::Value(Portable::Number(42.0)));
    assert!(next.destroyed && !next.disposed && next.late_rejected == 3);
    assert_ne!(r.generation, next.generation);
}

#[test]
fn same_request_selects_engine_or_allocator_but_neither_prevents_catch() {
    // Same 1 MiB request; no actual process OOM. Both ceilings are finite here.
    let body =
        "try { 'x'.repeat(1048576); return 0; } catch (_) { t.note(77); t.action(); return 42; }";
    let healthy = probe(body, Config::default());
    assert_eq!(healthy.outcome, Outcome::Value(Portable::Number(0.0)));
    assert!(healthy.notes.is_empty() && healthy.allocation_rejects == 0);
    retired_and_healthy(&healthy);
    for (heap, allocator_hit) in [(512 * 1024, false), (HEAP, true)] {
        let r = probe(
            body,
            Config {
                call_heap_limit: Some(heap),
                single_allocation: Some(256 * 1024),
                ..Config::default()
            },
        );
        assert_eq!(r.notes, [77], "{r:?}");
        assert_eq!(r.allocation_rejects > 0, allocator_hit, "{r:?}");
        if allocator_hit {
            assert_eq!(r.outcome, Outcome::Stop(Reason::ResourceLimit));
            assert!(r.disposed);
            assert_eq!(r.action, 0);
        } else {
            assert_eq!(r.outcome, Outcome::Value(Portable::Number(42.0)));
            assert!(!r.disposed);
            assert_eq!(r.action, 1);
        }
        println!("PUBLIC API paired heap={heap} {r:?}");
        retired_and_healthy(&r);
    }
}

#[test]
fn removing_engine_ceiling_moves_rejection_to_allocator_without_stopping_source() {
    let body = "try { 'x'.repeat(33554432); } catch (_) { t.note(77); t.action(); return 42; }";
    for heap in [HEAP, 0] {
        let r = probe(
            body,
            Config {
                call_heap_limit: Some(heap),
                ..Config::default()
            },
        );
        assert_eq!(r.notes, [77]);
        assert_eq!(r.allocation_rejects > 0, heap == 0, "{r:?}");
        assert_eq!(r.disposed, heap == 0);
        assert_eq!(r.action, usize::from(heap != 0));
        assert_eq!(
            r.outcome,
            if heap == 0 {
                Outcome::Stop(Reason::ResourceLimit)
            } else {
                Outcome::Value(Portable::Number(42.0))
            }
        );
        println!("PUBLIC API ceiling override={heap} {r:?}");
        retired_and_healthy(&r);
    }
}

#[test]
fn allocator_and_heap_failures_continue_inside_promise_async_and_finally() {
    // note is an intentionally ungated witness, not a capability action. Markers
    // prove source execution; action separately tests the real authority gate.
    let failure = "try { 'x'.repeat(1048576); } catch (_) { t.note(77); t.action(); } finally { t.note(78); } return 42;";
    for (name, shape) in [
        ("direct", "FAIL"),
        (
            "executor",
            "const p=new Promise(resolve=>{resolve((()=>{FAIL})());}); t.note(79); return p;",
        ),
        ("reaction", "return Promise.resolve().then(()=>{FAIL});"),
        ("await", "return (async()=>{await 0; FAIL})();"),
        (
            "thenable",
            "return Promise.resolve({then(resolve){resolve((()=>{FAIL})());}});",
        ),
        (
            "async-generator",
            "return (async function*(){await 0; FAIL})().next().then(x=>x.value);",
        ),
    ] {
        for (heap, allocator_hit) in [(512 * 1024, false), (HEAP, true)] {
            let r = probe(
                &shape.replace("FAIL", failure),
                Config {
                    call_heap_limit: Some(heap),
                    single_allocation: Some(256 * 1024),
                    ..Config::default()
                },
            );
            assert!(r.notes.starts_with(&[77, 78]), "{name} {r:?}");
            assert_eq!(r.notes.contains(&79), name == "executor", "{r:?}");
            assert_eq!(r.allocation_rejects > 0, allocator_hit, "{r:?}");
            assert_eq!(r.action, usize::from(!allocator_hit), "{r:?}");
            assert_eq!(r.disposed, allocator_hit);
            assert_eq!(
                r.outcome,
                if allocator_hit {
                    Outcome::Stop(Reason::ResourceLimit)
                } else {
                    Outcome::Value(Portable::Number(42.0))
                }
            );
            println!("PUBLIC API continuation route={name} allocator={allocator_hit} {r:?}");
            retired_and_healthy(&r);
        }
    }
}

#[test]
fn allocator_latch_interrupts_at_a_later_poll_after_catch_already_ran() {
    let r = probe(
        "try {'x'.repeat(1048576)} catch (_) {t.note(77);t.action();for(;;){}} t.note(99);return 42;",
        Config {
            single_allocation: Some(256 * 1024),
            ..Config::default()
        },
    );
    assert_eq!(r.outcome, Outcome::Stop(Reason::ResourceLimit));
    assert!(r.allocation_rejects > 0 && r.disposed);
    assert_eq!(r.notes, [77]);
    assert_eq!(r.action, 0);
    println!("PUBLIC API later interrupt {r:?}");
    retired_and_healthy(&r);
}

#[test]
fn uncaught_oom_becomes_ordinary_rejection_before_host_observation() {
    for shape in [
        "return new Promise(()=>{FAIL}).catch(()=>{t.note(77); t.action(); return 42;});",
        "return Promise.resolve().then(()=>{FAIL}).catch(()=>{t.note(77); t.action(); return 42;});",
        "return (async()=>{await 0; FAIL})().catch(()=>{t.note(77); t.action(); return 42;});",
    ] {
        for (heap, allocator_hit) in [(512 * 1024, false), (HEAP, true)] {
            let r = probe(
                &shape.replace("FAIL", "'x'.repeat(1048576);"),
                Config {
                    call_heap_limit: Some(heap),
                    single_allocation: Some(256 * 1024),
                    ..Config::default()
                },
            );
            assert_eq!(r.allocation_rejects > 0, allocator_hit, "{r:?}");
            if allocator_hit {
                // Host sees its latch before pumping the NEXT catch reaction.
                assert!(r.notes.is_empty());
                assert_eq!(r.action, 0);
                assert_eq!(r.outcome, Outcome::Stop(Reason::ResourceLimit));
                assert!(r.queued_at_retirement && r.disposed);
            } else {
                assert_eq!(r.notes, [77]);
                assert_eq!(r.action, 1);
                assert_eq!(r.outcome, Outcome::Value(Portable::Number(42.0)));
                assert!(!r.disposed);
            }
            retired_and_healthy(&r);
        }
    }
}

#[test]
fn ordinary_forged_and_stack_exceptions_remain_catchable() {
    for failure in [
        "throw null;",
        "throw new Error('out of memory');",
        "throw {name:'InternalError',message:'out of memory',code:'RESOURCE_LIMIT'};",
        "throw new RangeError('Maximum call stack size exceeded');",
        "(function recurse(){recurse()})();",
    ] {
        for shape in [
            "try { FAIL } catch (_) { t.note(77); t.action(); return 42; }",
            "return new Promise(()=>{FAIL}).catch(()=>{t.note(77); t.action(); return 42;});",
            "return (async()=>{await 0; try {FAIL} catch(_){t.note(77);t.action();return 42;}})();",
        ] {
            let r = probe(&shape.replace("FAIL", failure), Config::default());
            assert_eq!(r.outcome, Outcome::Value(Portable::Number(42.0)), "{r:?}");
            assert_eq!(r.notes, [77]);
            assert_eq!(r.action, 1);
            assert_eq!(r.allocation_rejects, 0);
            assert!(!r.disposed);
            retired_and_healthy(&r);
        }
        let r = probe(failure, Config::default());
        assert_eq!(r.outcome, Outcome::SourceError);
        assert_eq!(r.allocation_rejects, 0);
        assert!(!r.disposed);
        retired_and_healthy(&r);
    }
}

#[test]
fn public_error_marking_is_value_local_and_after_catch_is_too_late() {
    let rt = Runtime::new().unwrap();
    rt.set_memory_limit(HEAP);
    let weak = rt.weak();
    let context = Context::full(&rt).unwrap();
    context.with(|ctx| {
        ffi::leak_checks(&ctx);
        assert!(ctx.eval::<Value, _>("globalThis.continued=0; try {'x'.repeat(33554432)} catch(e) {continued=77; throw e}").is_err());
        let error = ctx.catch();
        assert_eq!(ctx.globals().get::<_, i32>("continued").unwrap(), 77);
        assert_eq!(ffi::mark_error(&ctx, &error), (false, true));
        // Marking an existing error is supported, but not a runtime OOM policy.
        let ordinary: Value = ctx.eval("new Error('out of memory')").unwrap();
        assert_eq!(ffi::mark_error(&ctx, &ordinary), (false, true));
        let null = Value::new_null(ctx.clone());
        assert_eq!(ffi::mark_error(&ctx, &null), (false, false));
        // Force the real engine OOM error-allocation fallback to null.
        let f: Function = ctx.eval("()=>{try{'x'.repeat(33554432)}catch(e){throw e}}").unwrap();
        ffi::heap_limit(&ctx, 1);
        assert!(f.call::<_, Value>(()).is_err());
        let fallback = ctx.catch();
        assert!(fallback.is_null(), "engine OOM fallback must be witnessed");
        assert_eq!(ffi::mark_error(&ctx, &fallback), (false, false));
    });
    drop(context);
    drop(rt);
    assert!(weak.try_ref().is_none());
    let fresh = probe("return 42;", Config::default());
    assert_eq!(fresh.outcome, Outcome::Value(Portable::Number(42.0)));
    retired_and_healthy(&fresh);
}
