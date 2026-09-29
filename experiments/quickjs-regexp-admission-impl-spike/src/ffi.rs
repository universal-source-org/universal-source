//! The only unsafe code in this crate: two documented public QuickJS-NG C APIs
//! reached through rquickjs `Ctx::as_raw` / `Value::as_raw`. Pins: rquickjs-sys
//! 0.14.0 vendoring QuickJS-NG 0.16.2.
use rquickjs::{Ctx, Value, qjs};

/// Marks a trusted, setup-time Error object uncatchable. Called once per realm.
#[allow(unsafe_code)]
pub fn mark_uncatchable(ctx: &Ctx<'_>, value: &Value<'_>) {
    // SAFETY: public `JS_SetUncatchableError(JSContext *, JSValueConst)` (quickjs.c:12068)
    // borrows `value`, accepts every tag, and only sets a header bit on JS_CLASS_ERROR
    // objects. It runs no JavaScript, allocates nothing and retains nothing. `ctx` and
    // `value` are live, owned by the same runtime, and used on its owner thread.
    unsafe { qjs::JS_SetUncatchableError(ctx.as_raw().as_ptr(), value.as_raw()) }
}

/// Invisible [[RegExpMatcher]] brand check: no property access, no Proxy trap.
#[allow(unsafe_code)]
pub fn has_regexp_brand(value: &Value<'_>) -> bool {
    // SAFETY: public `JS_IsRegExp(JSValueConst)` (quickjs.c:12008) compares
    // `JS_GetClassID(value)` with JS_CLASS_REGEXP. It borrows the value, returns
    // JS_INVALID_CLASS_ID for non-objects, reads only the object header and runs
    // no JavaScript. A Proxy reports its own class, so no trap can run.
    unsafe { qjs::JS_IsRegExp(value.as_raw()) }
}
