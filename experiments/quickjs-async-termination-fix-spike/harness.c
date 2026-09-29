/*
 * Throwaway engine harness for the Universal Source project (Phase 3).
 *
 * Purpose: determine whether the Promise/async uncatchable-stop *swallowing*
 * defect is completely fixable by a bounded engine change that reuses the
 * engine's own `JS_IsUncatchableError` guard (the pattern `promise_reaction_job`
 * already carries).
 *
 * This file is compiled TWICE by build_and_run.py:
 *   - against a pristine copy of the pinned QuickJS-NG source ("baseline")
 *   - against a generated, gitignored, guard-patched copy      ("patched")
 * and the two outputs are diffed.
 *
 * A host stop is modelled two ways, both producing an *uncatchable* error, the
 * same shape the real engine produces for deadline / cancellation / resource
 * interrupts (JS_ThrowInterrupted -> JS_SetUncatchableError) and the same shape
 * the RegExp pre-admission gate raises:
 *   - stop(): native throw of an InternalError marked uncatchable.
 *   - a one-shot interrupt handler (the real engine deadline mechanism).
 *
 * NO OUTER WATCHDOG is used here; termination must be proven in-process by the
 * engine returning control to this host with the uncatchable still latched and
 * with source JS never continuing.
 */
#define _GNU_SOURCE
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "quickjs.h"

/* ---- continuation marker: proves whether source JS ran past the stop ---- */
static int g_cont;
static JSValue js_cont(JSContext *ctx, JSValueConst this_val,
                       int argc, JSValueConst *argv) {
    (void)ctx; (void)this_val; (void)argc; (void)argv;
    g_cont++;
    return JS_UNDEFINED;
}

/* ---- host stop: an uncatchable error, like a latched Timeout/Cancel/Limit ---- */
static JSValue js_stop(JSContext *ctx, JSValueConst this_val,
                       int argc, JSValueConst *argv) {
    (void)this_val; (void)argc; (void)argv;
    JS_ThrowInternalError(ctx, "HOST_STOP");
    JSValue e = JS_GetException(ctx);       /* materialise so we can mark it */
    JS_SetUncatchableError(ctx, e);          /* latch: flag lives on the object */
    return JS_Throw(ctx, e);                 /* re-throw; flag survives roundtrip */
}

/* ---- latched deadline: the real engine timeout/cancel/resource mechanism ----
 * expire() arms a pending stop (like the host latching Reason::Timeout); the very
 * next interrupt poll raises JS_ThrowInterrupted once (an uncatchable), mirroring
 * the gate's pending->latch->raise behaviour in the preserved Rust reproducer. */
static int g_pending;
static int on_interrupt(JSRuntime *rt, void *opaque) {
    (void)rt; (void)opaque;
    if (g_pending) { g_pending = 0; return 1; }
    return 0;
}
static JSValue js_expire(JSContext *ctx, JSValueConst this_val,
                         int argc, JSValueConst *argv) {
    (void)ctx; (void)this_val; (void)argc; (void)argv;
    g_pending = 1;
    return JS_UNDEFINED;
}

static void register_natives(JSContext *ctx) {
    JSValue g = JS_GetGlobalObject(ctx);
    JS_SetPropertyStr(ctx, g, "cont", JS_NewCFunction(ctx, js_cont, "cont", 0));
    JS_SetPropertyStr(ctx, g, "stop", JS_NewCFunction(ctx, js_stop, "stop", 0));
    JS_SetPropertyStr(ctx, g, "expire", JS_NewCFunction(ctx, js_expire, "expire", 0));
    JS_FreeValue(ctx, g);
}

typedef enum { CLS_ORDINARY, CLS_UNCATCHABLE, CLS_GUARD } CaseClass;

typedef struct {
    const char *name;
    CaseClass   cls;
    int         prearm;   /* arm the latch before eval (deadline in a bare loop) */
    const char *src;
} Case;

/*
 * Each uncatchable/guard case tries to make source continue past the stop via a
 * rejection handler (.catch / onRejected), a .finally, or a following
 * statement. If any of those runs, cont() fires => the stop was SWALLOWED.
 * Ordinary twins throw a normal Error and MUST reach their handler (CAUGHT).
 */
static const Case CASES[] = {
    /* 1. Promise executor throw  (quickjs.c site: js_promise_constructor) */
    { "executor__uncatchable", CLS_UNCATCHABLE, 0,
      "new Promise(function(res,rej){ stop(); }).catch(function(e){ cont(); });" },
    { "executor__ordinary", CLS_ORDINARY, 0,
      "new Promise(function(res,rej){ throw new Error('ORD'); }).catch(function(e){ cont(); });" },

    /* 2. thenable `then` getter throw via Promise.resolve (js_promise_resolve_function_call) */
    { "resolve_thenable_getter__uncatchable", CLS_UNCATCHABLE, 0,
      "var t={}; Object.defineProperty(t,'then',{get:function(){ stop(); }});"
      "Promise.resolve(t).catch(function(e){ cont(); });" },
    { "resolve_thenable_getter__ordinary", CLS_ORDINARY, 0,
      "var t={}; Object.defineProperty(t,'then',{get:function(){ throw new Error('ORD'); }});"
      "Promise.resolve(t).catch(function(e){ cont(); });" },

    /* 3. thenable `then` call throw in the resolve-thenable job (js_promise_resolve_thenable_job) */
    { "thenable_then_call__uncatchable", CLS_UNCATCHABLE, 0,
      "var t={ then:function(res,rej){ stop(); } };"
      "Promise.resolve(t).then(function(){ cont(); }, function(e){ cont(); });" },
    { "thenable_then_call__ordinary", CLS_ORDINARY, 0,
      "var t={ then:function(res,rej){ throw new Error('ORD'); } };"
      "Promise.resolve(t).then(function(){ cont(); }, function(e){ cont(); });" },

    /* 4. await a thenable whose `then` getter throws (site 2 reached via js_async_function_resume) */
    { "await_thenable_getter__uncatchable", CLS_UNCATCHABLE, 0,
      "(async function(){ var t={}; Object.defineProperty(t,'then',{get:function(){ stop(); }});"
      " await t; })().then(function(){ cont(); }, function(e){ cont(); });" },
    { "await_thenable_getter__ordinary", CLS_ORDINARY, 0,
      "(async function(){ var t={}; Object.defineProperty(t,'then',{get:function(){ throw new Error('ORD'); }});"
      " await t; })().then(function(){ cont(); }, function(e){ cont(); });" },

    /* 5. Promise.all iterator throw (js_promise_all fail_reject) */
    { "promise_all_iter__uncatchable", CLS_UNCATCHABLE, 0,
      "Promise.all({ [Symbol.iterator](){ return { next(){ stop(); } }; } })"
      ".catch(function(e){ cont(); });" },
    { "promise_all_iter__ordinary", CLS_ORDINARY, 0,
      "Promise.all({ [Symbol.iterator](){ return { next(){ throw new Error('ORD'); } }; } })"
      ".catch(function(e){ cont(); });" },

    /* 6. Promise.race iterator throw (js_promise_race fail_reject) */
    { "promise_race_iter__uncatchable", CLS_UNCATCHABLE, 0,
      "Promise.race({ [Symbol.iterator](){ return { next(){ stop(); } }; } })"
      ".catch(function(e){ cont(); });" },
    { "promise_race_iter__ordinary", CLS_ORDINARY, 0,
      "Promise.race({ [Symbol.iterator](){ return { next(){ throw new Error('ORD'); } }; } })"
      ".catch(function(e){ cont(); });" },

    /* 7. Promise.try callback throw (js_promise_try) */
    { "promise_try__uncatchable", CLS_UNCATCHABLE, 0,
      "Promise.try(function(){ stop(); }).catch(function(e){ cont(); });" },
    { "promise_try__ordinary", CLS_ORDINARY, 0,
      "Promise.try(function(){ throw new Error('ORD'); }).catch(function(e){ cont(); });" },

    /* 8. async generator resume throw (js_async_generator_resume_next).
     *    8i: synchronous first resume  -> reached via js_async_generator_next.
     *    8ii: resume after an await     -> reached via js_async_generator_resolve_function. */
    { "async_gen_resume__uncatchable", CLS_UNCATCHABLE, 0,
      "(async function*(){ stop(); yield 1; })().next()"
      ".then(function(){ cont(); }, function(e){ cont(); });" },
    { "async_gen_resume__ordinary", CLS_ORDINARY, 0,
      "(async function*(){ throw new Error('ORD'); yield 1; })().next()"
      ".then(function(){ cont(); }, function(e){ cont(); });" },
    { "async_gen_await_resume__uncatchable", CLS_UNCATCHABLE, 0,
      "(async function*(){ await Promise.resolve(1); stop(); yield 1; })().next()"
      ".then(function(){ cont(); }, function(e){ cont(); });" },
    { "async_gen_await_resume__ordinary", CLS_ORDINARY, 0,
      "(async function*(){ await Promise.resolve(1); throw new Error('ORD'); yield 1; })().next()"
      ".then(function(){ cont(); }, function(e){ cont(); });" },

    /* 8b. async generator completed .return() normal path (js_async_generator_completed_return)
     *     Regression-only: the uncatchable branch of this site needs a corrupted
     *     %Promise% to reach and is not reproducible from restricted JS; this case
     *     proves the patched path still returns normally. */
    { "async_gen_completed_return__ordinary", CLS_ORDINARY, 0,
      "var g=(async function*(){ yield 1; })();"
      "g.return(42).then(function(v){ if(v.value===42 && v.done) cont(); });" },

    /* 9. for-await over a SYNC iterator that throws (js_async_from_sync_iterator_next) */
    { "for_await_sync_iter__uncatchable", CLS_UNCATCHABLE, 0,
      "(async function(){ for await (var x of { [Symbol.iterator](){ return { next(){ stop(); } }; } })"
      " { cont(); } })().then(function(){ cont(); }, function(e){ cont(); });" },
    { "for_await_sync_iter__ordinary", CLS_ORDINARY, 0,
      "(async function(){ for await (var x of { [Symbol.iterator](){ return { next(){ throw new Error('ORD'); } }; } })"
      " { cont(); } })().then(function(){ cont(); }, function(e){ cont(); });" },

    /* 10. GUARD REGRESSION: promise reaction job (already correct at quickjs.c:55584) */
    { "reaction_job_guard__uncatchable", CLS_GUARD, 0,
      "Promise.resolve(1).then(function(){ stop(); }).catch(function(e){ cont(); });" },
    { "reaction_job_guard__ordinary", CLS_ORDINARY, 0,
      "Promise.resolve(1).then(function(){ throw new Error('ORD'); }).catch(function(e){ cont(); });" },

    /* 11. GUARD REGRESSION: async function resume (already correct at quickjs.c:21482) */
    { "async_func_guard__uncatchable", CLS_GUARD, 0,
      "(async function(){ stop(); })().catch(function(e){ cont(); });" },
    { "async_func_guard__ordinary", CLS_ORDINARY, 0,
      "(async function(){ throw new Error('ORD'); })().catch(function(e){ cont(); });" },

    /* 12. Real engine deadline: latched interrupt inside an executor loop */
    { "interrupt_executor__uncatchable", CLS_UNCATCHABLE, 1,
      "new Promise(function(res,rej){ while(true){} }).catch(function(e){ cont(); });" },

    /* 12b-12f. Deadline (latched interrupt) variants of the exact five shapes in
     *          the preserved Rust reproducer (STOP -> `expire(); for(;;){}`).
     *          These exercise the interpreter-raised uncatchable (as opposed to
     *          the native admission-gate throw modelled by stop()). */
    { "deadline_executor__uncatchable", CLS_UNCATCHABLE, 0,
      "new Promise(function(){ expire(); for(;;){} }).catch(function(e){ cont(); });" },
    { "deadline_resolve_getter__uncatchable", CLS_UNCATCHABLE, 0,
      "new Promise(function(resolve){ resolve({ get then(){ expire(); for(;;){} } }); })"
      ".catch(function(e){ cont(); });" },
    { "deadline_promise_resolve_getter__uncatchable", CLS_UNCATCHABLE, 0,
      "Promise.resolve({ get then(){ expire(); for(;;){} } }).catch(function(e){ cont(); });" },
    { "deadline_await_getter__uncatchable", CLS_UNCATCHABLE, 0,
      "(async function(){ await { get then(){ expire(); for(;;){} } }; })()"
      ".then(function(){ cont(); }, function(e){ cont(); });" },
    { "deadline_promise_all_iter__uncatchable", CLS_UNCATCHABLE, 0,
      "Promise.all({ [Symbol.iterator](){ expire(); for(;;){} } }).catch(function(e){ cont(); });" },

    /* 13. finally MUST NOT run (a stop cannot be laundered into a settled outcome) */
    { "finally_no_success__uncatchable", CLS_UNCATCHABLE, 0,
      "new Promise(function(res,rej){ stop(); })"
      ".finally(function(){ cont(); }).then(function(){ cont(); });" },

    /* 14. RegExp admission-gate stop shape: the gate raises the identical
     *     uncatchable before compiling; here it is raised from an executor to
     *     show the same corrected propagation path carries it. */
    { "regexp_admission_gate__uncatchable", CLS_UNCATCHABLE, 0,
      "new Promise(function(res,rej){ stop(); /* pre-admission RESOURCE_LIMIT */ })"
      ".catch(function(e){ cont(); });" },
};

#define NCASES ((int)(sizeof(CASES) / sizeof(CASES[0])))

static const char *run_case(const Case *c) {
    g_cont = 0;
    g_pending = 0;

    JSRuntime *rt = JS_NewRuntime();
    JS_SetDumpFlags(rt, JS_DUMP_LEAKS | JS_DUMP_ATOM_LEAKS | JS_ABORT_ON_LEAKS);
    JSContext *ctx = JS_NewContext(rt);
    register_natives(ctx);
    JS_SetInterruptHandler(rt, on_interrupt, NULL);  /* inert until armed */
    if (c->prearm)
        g_pending = 1;

    int eval_unc = 0, pump_unc = 0;

    JSValue v = JS_Eval(ctx, c->src, strlen(c->src), "<case>", JS_EVAL_TYPE_GLOBAL);
    if (JS_IsException(v)) {
        JSValue e = JS_GetException(ctx);   /* host regains control; consume */
        if (JS_IsUncatchableError(e)) eval_unc = 1;
        JS_FreeValue(ctx, e);
    }
    JS_FreeValue(ctx, v);

    JSContext *pctx;
    for (int i = 0; i < 100000; i++) {
        int r = JS_ExecutePendingJob(rt, &pctx);
        if (r == 0) break;
        if (r < 0) {
            JSValue e = JS_GetException(pctx);
            if (JS_IsUncatchableError(e)) pump_unc = 1;
            JS_FreeValue(pctx, e);
        }
    }

    const char *verdict;
    if (c->cls == CLS_ORDINARY) {
        verdict = (g_cont > 0) ? "CAUGHT" : "NOTCAUGHT";
    } else {
        if (g_cont > 0)               verdict = "SWALLOWED";
        else if (eval_unc || pump_unc) verdict = "TERMINATED";
        else                          verdict = "UNRESOLVED";
    }

    printf("CASE %s => %s (cont=%d eval_unc=%d pump_unc=%d)\n",
           c->name, verdict, g_cont, eval_unc, pump_unc);
    fflush(stdout);

    JS_FreeContext(ctx);
    JS_FreeRuntime(rt);   /* leak check runs here; JS_ABORT_ON_LEAKS aborts on leak */
    return verdict;
}

/* fresh, independent runtime is healthy after all the stop cases */
static int health_check(void) {
    JSRuntime *rt = JS_NewRuntime();
    JS_SetDumpFlags(rt, JS_DUMP_LEAKS | JS_DUMP_ATOM_LEAKS | JS_ABORT_ON_LEAKS);
    JSContext *ctx = JS_NewContext(rt);
    register_natives(ctx);
    g_cont = 0;
    const char *src =
        "Promise.resolve(20).then(function(v){ if (v + 21 === 41) cont(); });";
    JSValue v = JS_Eval(ctx, src, strlen(src), "<health>", JS_EVAL_TYPE_GLOBAL);
    int ok = !JS_IsException(v);
    JS_FreeValue(ctx, v);
    JSContext *pctx;
    for (int i = 0; i < 1000; i++) {
        int r = JS_ExecutePendingJob(rt, &pctx);
        if (r == 0) break;
        if (r < 0) { JSValue e = JS_GetException(pctx); JS_FreeValue(pctx, e); ok = 0; }
    }
    ok = ok && (g_cont == 1);
    JS_FreeContext(ctx);
    JS_FreeRuntime(rt);
    return ok;
}

int main(void) {
    for (int i = 0; i < NCASES; i++) {
        (void)run_case(&CASES[i]);              /* prints its own CASE line */
        printf("FREED %s\n", CASES[i].name);    /* absence => leak-abort in free */
        fflush(stdout);
    }
    printf("HEALTH %s\n", health_check() ? "ok" : "fail");
    fflush(stdout);
    return 0;
}
