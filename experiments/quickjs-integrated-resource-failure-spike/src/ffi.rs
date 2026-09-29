//! Public-API boundaries, no engine-private access.
#![allow(unsafe_code)]
use rquickjs::{Ctx, qjs};

pub fn leak_checks(ctx: &Ctx<'_>) {
    // SAFETY: live owner-thread context supplies its own runtime. ENABLE_DUMPS
    // is set by run.py; these flags abort on VM object/atom leaks at destruction.
    unsafe {
        qjs::JS_SetDumpFlags(
            qjs::JS_GetRuntime(ctx.as_raw().as_ptr()),
            (qjs::JS_DUMP_LEAKS | qjs::JS_DUMP_ATOM_LEAKS | qjs::JS_ABORT_ON_LEAKS) as u64,
        );
    }
}
pub fn pending(ctx: &Ctx<'_>) -> bool {
    // SAFETY: read-only queue query on the live owner-thread runtime.
    unsafe { qjs::JS_IsJobPending(qjs::JS_GetRuntime(ctx.as_raw().as_ptr())) }
}
pub fn job(ctx: &Ctx<'_>) -> bool {
    // SAFETY: exactly one live context is installed. Public API sets failed_ctx
    // on failure; release its pending exception without inspecting/coercing it.
    // Never use the binding's bool-only helper to conflate failed/no-job states.
    unsafe {
        let mut failed_ctx = std::ptr::null_mut();
        let result =
            qjs::JS_ExecutePendingJob(qjs::JS_GetRuntime(ctx.as_raw().as_ptr()), &mut failed_ctx);
        if result < 0 {
            assert_eq!(failed_ctx, ctx.as_raw().as_ptr());
            qjs::JS_FreeValue(failed_ctx, qjs::JS_GetException(failed_ctx));
        }
        result > 0
    }
}

pub fn heap_limit(ctx: &Ctx<'_>, bytes: usize) {
    // SAFETY: public runtime API on the owner thread, outside engine execution.
    // A deterministic boundary fault probe; no allocator-callback reentry.
    unsafe { qjs::JS_SetMemoryLimit(qjs::JS_GetRuntime(ctx.as_raw().as_ptr()), bytes as _) }
}

#[cfg(test)]
pub fn mark_error(ctx: &Ctx<'_>, value: &rquickjs::Value<'_>) -> (bool, bool) {
    // SAFETY: tests supply a live value from this context, held throughout both
    // queries and the public flag setter. No ownership transfer or VM reentry.
    unsafe {
        let before = qjs::JS_IsUncatchableError(value.as_raw());
        qjs::JS_SetUncatchableError(ctx.as_raw().as_ptr(), value.as_raw());
        (before, qjs::JS_IsUncatchableError(value.as_raw()))
    }
}
