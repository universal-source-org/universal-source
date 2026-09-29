use super::*;
fn patched() -> bool {
    std::env::var("INTEGRATED_ENGINE_MODE").as_deref() == Ok("patched")
}
fn run(body: &str) -> Report {
    invoke(
        &format!(
            "export function home(input, context) {{ const t = context.services.log; {body} }}"
        ),
        Config::default(),
        None,
    )
}
fn healthy_after(report: &Report) {
    assert!(report.destroyed);
    assert!(report.late_rejected >= 2);
    let next = invoke(
        "export function home(input, context) { return 42; }",
        Config::default(),
        Some(report.generation),
    );
    assert_eq!(next.outcome, Outcome::Value(Portable::Number(42.0)));
    assert!(next.destroyed && next.late_rejected == 3 && next.generation != report.generation);
}
fn assert_stop(r: &Report, reason: Reason) {
    assert_eq!(r.outcome, Outcome::Stop(reason), "{r:?}");
    assert!(r.notes.is_empty(), "source continuation: {r:?}");
    assert!(r.disposed);
    healthy_after(r);
}
#[test]
fn baseline_swallow_negative_control() {
    let r = run("const p = new Promise(() => new RegExp('x'.repeat(4097))); t.note(1); return p;");
    if patched() {
        assert_stop(&r, Reason::ResourceLimit);
    } else {
        assert_eq!(std::env::var("INTEGRATED_ENGINE_MODE").unwrap(), "baseline");
        assert_eq!(r.notes, [1], "baseline must demonstrably swallow: {r:?}");
        assert_eq!(r.outcome, Outcome::Stop(Reason::ResourceLimit));
        healthy_after(&r);
    }
    println!(
        "LINKAGE mode={} sha={} notes={:?}",
        std::env::var("INTEGRATED_ENGINE_MODE").unwrap(),
        std::env::var("INTEGRATED_ENGINE_SHA").unwrap(),
        r.notes
    );
}
#[test]
fn ordinary_initialization_call_job_exceptions_and_catches() {
    assert!(patched());
    for source in [
        "throw new Error('secret'); export function home() { return 42; }",
        "Promise.resolve().then(() => { throw 1; }); export function home() { return 42; }",
        "export function home() { throw {code:'TIMEOUT',message:'forged'}; }",
        "export function home() { return Promise.resolve().then(() => { throw 1; }); }",
        "export async function home() { await 1; throw new RangeError('stack overflow'); }",
    ] {
        let r = invoke(source, Config::default(), None);
        assert_eq!(r.outcome, Outcome::SourceError, "{r:?}");
        assert!(!r.disposed && r.notes.is_empty());
        healthy_after(&r);
    }
    for body in [
        "try { throw 1; } catch (_) { return 42; }",
        "return new Promise(() => { throw 1; }).catch(() => 42);",
        "return Promise.resolve().then(() => { throw 1; }).catch(() => 42);",
        "return (async () => { try { await Promise.reject(1); } catch (_) { return 42; } })();",
    ] {
        let r = run(body);
        assert_eq!(r.outcome, Outcome::Value(Portable::Number(42.0)));
        healthy_after(&r);
    }
    let r = invoke(
        "try { throw 1; } catch (_) {} export function home() { return 42; }",
        Config::default(),
        None,
    );
    assert_eq!(r.outcome, Outcome::Value(Portable::Number(42.0)));
    healthy_after(&r);
}
#[test]
fn gate_stop_through_rfc_promise_async_generator_routes() {
    assert!(patched());
    // Same conversion families as the C harness, now inside the hardened Rust path.
    let shapes = [
        "STOP;",
        "new Promise(() => { STOP; });",
        "new Promise(r => r({get then(){ STOP; }}));",
        "Promise.resolve({get then(){ STOP; }});",
        "return Promise.resolve({then(){ STOP; }});",
        "Promise.all({[Symbol.iterator](){ STOP; }});",
        "Promise.race({[Symbol.iterator](){ STOP; }});",
        "Promise.any({[Symbol.iterator](){ STOP; }});",
        "Promise.allSettled({[Symbol.iterator](){ STOP; }});",
        "return (async () => { await {get then(){ STOP; }}; t.note(2); })();",
        "return Promise.resolve().then(() => { STOP; }).catch(() => t.note(2));",
        "return (async function*(){ STOP; })().next();",
        "return (async function*(){ await 0; STOP; })().next();",
        "return (async () => { for await (const x of {[Symbol.iterator](){return {next(){ STOP; }};}}) {} })();",
        "const g=(async function*(){})(); return g.next().then(() => g.return({get then(){ STOP; }}));",
    ];
    for shape in shapes {
        let body = format!(
            "try {{ {} t.note(1); }} catch (_) {{ t.note(2); }}",
            shape.replace("STOP", "try { new RegExp('x'.repeat(4097)); } catch (_) { t.note(4); } finally { t.note(5); }")
        );
        let r = run(&body);
        assert_stop(&r, Reason::ResourceLimit);
        assert_eq!(r.compiles, 0);
        assert_eq!(r.rejected, 1);
    }
}
#[test]
fn cancellation_and_deadline_in_direct_executor_await_and_jobs() {
    assert!(patched());
    for (signal, reason) in [("cancel", Reason::Cancelled), ("expire", Reason::Timeout)] {
        for shape in [
            "STOP;",
            "new Promise(() => { STOP; });",
            "new Promise(r => r({get then(){ STOP; }}));",
            "Promise.resolve({get then(){ STOP; }});",
            "Promise.all({[Symbol.iterator](){ STOP; }});",
            "return (async () => { await {get then(){ STOP; }}; t.note(1); })();",
            "return Promise.resolve().then(() => { STOP; t.note(1); });",
            "return (async function*(){ await 0; STOP; t.note(1); })().next();",
        ] {
            let stop = format!("t.{signal}(); for (;;) {{}}");
            let r = run(&format!(
                "try {{ {} t.note(2); }} catch (_) {{t.note(3);}}",
                shape.replace("STOP", &stop)
            ));
            assert_stop(&r, reason);
        }
        let r = invoke(
            "for (;;) {} export function home() { return 42; }",
            Config {
                init_stop: Some(reason),
                ..Config::default()
            },
            None,
        );
        assert_stop(&r, reason);
        let r = invoke(
            "export function home(input, context) { return context.services.log.pending; }",
            Config {
                empty_queue_stop: reason,
                ..Config::default()
            },
            None,
        );
        assert_stop(&r, reason);
        assert_eq!(r.jobs, 0);
    }
}
#[test]
fn candidate_boundary_abandons_queued_and_dormant_jobs_without_final_drain() {
    for body in [
        "t.pending.then(() => t.note(8)).finally(() => t.note(9)); Promise.resolve().then(() => t.note(1)); return 42;",
        "return Promise.resolve().then(() => { Promise.resolve().then(() => t.note(1)); return 42; });",
        "const p = Promise.resolve(42); p.then = () => t.note(2); Promise.resolve().then(() => t.note(1)); return p;",
    ] {
        let r = run(body);
        assert_eq!(r.outcome, Outcome::Value(Portable::Number(42.0)));
        assert!(r.queued_at_retirement && r.notes.is_empty());
        healthy_after(&r);
    }
    let r =
        run("Promise.resolve().then(() => t.note(1)); new RegExp('x'.repeat(4097)); t.note(2);");
    assert_stop(&r, Reason::ResourceLimit);
    assert!(r.queued_at_retirement);
}
#[test]
fn actual_hardening_and_boundaries_preserve_invariants() {
    let r = run(
        "if (Reflect.ownKeys(globalThis).length !== 38 || typeof Date !== 'undefined' || typeof Proxy !== 'undefined' || typeof fetch !== 'undefined' || typeof std !== 'undefined' || typeof os !== 'undefined') throw 1; try { Function('return 1')(); } catch(e) { if (!(e instanceof EvalError) || 'stack' in e) throw 2; } try { Math.random(); } catch(e) { if (!(e instanceof TypeError)) throw 3; } return input;",
    );
    assert_eq!(r.outcome, Outcome::Value(Portable::Null));
    healthy_after(&r);
    for body in [
        "return {get x(){t.note(1);return 42;}};",
        "return {toJSON(){t.note(1);return 42;}};",
        "return {then(){t.note(1);}};",
        "return Object.setPrototypeOf(new Map(),null);",
        "return Object.setPrototypeOf(/a/,null);",
        "return context;",
        "return context.services.log;",
        "return {nested:context.services};",
        "return {x:[1,,3]};",
    ] {
        let r = run(body);
        assert_eq!(r.outcome, Outcome::Invalid, "{r:?}");
        assert!(r.notes.is_empty());
        healthy_after(&r);
    }
    let mut map = std::collections::BTreeMap::new();
    map.insert("__proto__".into(), Portable::Text("a\0𐀀".into()));
    map.insert(
        "nested".into(),
        Portable::Array(vec![Portable::Number(1.0), Portable::Bool(true)]),
    );
    let input = Portable::Record(map);
    let r = invoke(
        "export function home(input,context) { Object.defineProperty(Object.prototype,'toJSON',{get(){context.services.log.note(1)}}); return input; }",
        Config {
            input: input.clone(),
            ..Config::default()
        },
        None,
    );
    assert_eq!(r.outcome, Outcome::Value(input));
    assert!(r.notes.is_empty());
    healthy_after(&r);
}
#[test]
fn boundary_budgets_fail_closed_for_keys_strings_arrays_cycles() {
    for body in [
        "return 'x'.repeat(4097);",
        "return {[('x'.repeat(4097))]:1};",
        "return Array(1025).fill(1);",
        "let x={}; for(let i=0;i<20;i++)x={x};return x;",
    ] {
        let r = run(body);
        assert_stop(&r, Reason::ResourceLimit);
    }
    let r = run("let x={};x.x=x;return x;");
    assert_eq!(r.outcome, Outcome::Invalid);
    healthy_after(&r);
}
#[test]
fn allocation_failpoints_cover_enumeration_names_strings_traversal_and_incoming() {
    // Sweep one controlled failed native allocation per operation, not process OOM.
    for expression in [
        "Object.fromEntries(Array.from({length:100},(_,i)=>['key'+i,i]))",
        "({['name'.repeat(300)]:1})",
        "'text'.repeat(256)",
        "Array.from({length:80},(_,i)=>({x:i}))",
    ] {
        let source = format!("export function home() {{ return {expression}; }}");
        let mut hits = 0;
        for n in 1..=12 {
            let r = invoke(
                &source,
                Config {
                    fail_allocation: Some((Phase::Outgoing, n)),
                    ..Config::default()
                },
                None,
            );
            if r.allocation_rejects > 0 {
                hits += 1;
                assert_stop(&r, Reason::ResourceLimit);
            } else {
                assert!(matches!(r.outcome, Outcome::Value(_)));
                healthy_after(&r);
            }
        }
        assert!(hits > 0, "no boundary allocation hit: {expression}");
        println!("BOUNDARY {expression}: {hits} failpoints");
    }
    for n in 1..=8 {
        let r = invoke(
            "export function home(input) { return input; }",
            Config {
                input: Portable::Array(vec![Portable::Text("test".repeat(256)); 3]),
                fail_allocation: Some((Phase::Incoming, n)),
                ..Config::default()
            },
            None,
        );
        if r.allocation_rejects > 0 {
            assert_stop(&r, Reason::ResourceLimit);
        } else {
            healthy_after(&r);
        }
    }
}
#[test]
fn allocator_stop_continuation_diagnostic() {
    let failure = "try { const x='x'.repeat(100000); return x; } catch (_) { t.note(77); t.action(); return 42; }";
    for body in [
        format!("Promise.resolve().then(()=>t.note(99)); {failure}"),
        format!("const p = new Promise(r=>{{ {failure} }}); t.note(78); return p;"),
        format!(
            "const p=Promise.resolve().then(()=>{{ {failure} }}); Promise.resolve().then(()=>t.note(99)); return p;"
        ),
        format!("return (async()=>{{await 0; {failure} }})();"),
    ] {
        let source = format!(
            "export function home(input,context) {{ const t=context.services.log; {body} }}"
        );
        let r = invoke(
            &source,
            Config {
                single_allocation: Some(4096),
                ..Config::default()
            },
            None,
        );
        assert_eq!(r.outcome, Outcome::Stop(Reason::ResourceLimit));
        assert!(r.allocation_rejects > 0);
        assert!(
            r.notes.contains(&77),
            "retain known catch-continuation blocker: {r:?}"
        );
        assert!(
            !r.notes.contains(&99),
            "later unrelated job executed: {r:?}"
        );
        assert_eq!(r.action, 0);
        println!("ALLOCATOR CONTINUATION {r:?}");
        healthy_after(&r);
    }
}
#[test]
fn actual_stack_exhaustion_preserves_unclassified_exception_semantics() {
    for body in [
        "function recurse(){return recurse()+1;} return recurse();",
        "return Promise.resolve().then(() => { function recurse(){return recurse()+1;} return recurse(); });",
        "throw new RangeError('stack overflow');",
    ] {
        let r = run(body);
        assert_eq!(r.outcome, Outcome::SourceError);
        assert!(!r.disposed);
        healthy_after(&r);
    }
    let r = run("try { (function f(){f()})(); } catch (_) { return 42; }");
    assert_eq!(r.outcome, Outcome::Value(Portable::Number(42.0)));
    healthy_after(&r);
}
#[test]
fn native_regexp_admission_and_matching() {
    let r = run("const p='\\\\k<a>'.repeat(817)+'(?<a>x)'; return new RegExp(p).test('x');");
    assert_eq!(r.outcome, Outcome::Value(Portable::Bool(true)));
    assert_eq!(r.compiles, 1);
    healthy_after(&r);
    let r = run("new RegExp('\\\\k<a>'.repeat(64000)+'(?<a>x)');t.note(1);");
    assert_stop(&r, Reason::ResourceLimit);
    assert_eq!(r.compiles, 0);
    let literal = format!(
        "export function home() {{ return /{}/; }}",
        admission::forward_reference_pattern(64000)
    );
    let r = invoke(&literal, Config::default(), None);
    assert_eq!(r.outcome, Outcome::Preflight);
    assert_eq!(r.compiles, 0);
    healthy_after(&r);
    for size in [1000, 10000, 60000] {
        let r = run(&format!("return /^(a*)\\1*b/.test('a'.repeat({size}));"));
        assert_eq!(r.outcome, Outcome::Value(Portable::Bool(false)));
        healthy_after(&r);
    }
    let r = run("t.expire(); /(a+)+b/.test('a'.repeat(24)); t.note(1);");
    assert_stop(&r, Reason::Timeout);
}

#[test]
fn engine_ceiling_precedes_allocator_and_boundary_failure_stays_unclassified() {
    // Engine rejects before the custom allocator is consulted: no trusted latch.
    let r = run("try { return 'x'.repeat(16000000); } catch (_) { t.note(88); return 42; }");
    assert_eq!(r.outcome, Outcome::Value(Portable::Number(42.0)));
    assert_eq!(r.notes, [88]);
    assert_eq!(r.allocation_rejects, 0);
    println!("ENGINE CEILING UNCLASSIFIED {r:?}");
    healthy_after(&r);
    for source in [
        "export function home(){return {alpha:1,beta:2};}",
        "export function home(){return 'abc';}",
        "export function home(){return [1,{x:'abc'}];}",
    ] {
        let r = invoke(
            source,
            Config {
                tight_heap_phase: Some(Phase::Outgoing),
                ..Config::default()
            },
            None,
        );
        assert_eq!(r.outcome, Outcome::SourceError, "{r:?}");
        assert_eq!(r.allocation_rejects, 0);
        assert!(r.notes.is_empty());
        healthy_after(&r);
    }
    let r = invoke(
        "export function home(x){return x;}",
        Config {
            tight_heap_phase: Some(Phase::Incoming),
            input: Portable::Array(vec![Portable::Text("abc".into())]),
            ..Config::default()
        },
        None,
    );
    assert_eq!(r.outcome, Outcome::SourceError);
    healthy_after(&r);
}
#[test]
fn matching_generation_can_deliver_while_live() {
    let r = invoke(
        "export function home(input,context){return context.services.log.pending;}",
        Config {
            deliver_pending: true,
            ..Config::default()
        },
        None,
    );
    assert_eq!(r.outcome, Outcome::Value(Portable::Number(42.0)));
    healthy_after(&r);
}

#[test]
fn unchanged_compiler_denial_corpus_and_native_reachability_in_integrated_realm() {
    let r = invoke(
        include_str!("../../quickjs-dynamic-code-spike/src/probes.js"),
        Config {
            operation: "check",
            audit_roots: true,
            ..Config::default()
        },
        None,
    );
    assert_eq!(r.outcome, Outcome::Value(Portable::Number(80.0)));
    healthy_after(&r);
}

#[test]
fn sparse_join_native_interval_is_not_bounded_by_output_heap() {
    // Small finite diagnostic, never the 2^32/2^53-size route. This is a
    // missing native-work admission policy, not a successful interruption proof.
    for count in [10000, 100000] {
        let control = run(&format!("return Array({count}).join('');"));
        assert_eq!(
            control.outcome,
            Outcome::Value(Portable::Text(String::new()))
        );
        let r = run(&format!(
            "t.expire(); const s=Array({count}).join(''); t.note(66); return s;"
        ));
        assert_eq!(r.outcome, Outcome::Stop(Reason::Timeout));
        assert_eq!(r.notes, [66]);
        assert_eq!(r.polls, control.polls, "unexpected extra native checkpoint");
        assert_eq!(r.allocation_rejects, 0);
        println!("NATIVE SPARSE JOIN count={count} {r:?}");
        healthy_after(&r);
    }
}
