//! Non-production resource probes. No claim of a complete resource boundary.
#![forbid(unsafe_code)]

use rquickjs::{Context, Ctx, Runtime};
use std::{cell::Cell, rc::Rc};

pub const HEAP: usize = 8 * 1024 * 1024;
pub const STACK: usize = 128 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reason {
    Cancelled,
    Timeout,
    ResourceLimit,
}

/// Owner-thread state, inaccessible to source. Counts engine callbacks, not instructions.
#[derive(Default)]
pub struct Control {
    pub armed: Cell<bool>,
    pub callbacks: Cell<usize>,
    pub observed: Cell<Option<Reason>>,
}

pub fn realm() -> (Runtime, Context) {
    let rt = Runtime::new().unwrap();
    rt.set_memory_limit(HEAP);
    rt.set_max_stack_size(STACK);
    let ctx = Context::full(&rt).unwrap();
    ctx.with(|ctx| {
        ctx.eval::<(), _>(include_str!(
            "../../quickjs-dynamic-code-spike/src/harden.js"
        ))
        .unwrap();
        ctx.eval::<(), _>(include_str!(
            "../../quickjs-global-surface-spike/src/harden.js"
        ))
        .unwrap();
    });
    (rt, ctx)
}

pub fn install(rt: &Runtime, state: Rc<Control>, after: usize, reason: Reason) {
    rt.set_interrupt_handler(Some(Box::new(move || {
        if !state.armed.get() {
            return false;
        }
        let n = state.callbacks.get() + 1;
        state.callbacks.set(n);
        if n >= after && state.observed.get().is_none() {
            state.observed.set(Some(reason));
        }
        state.observed.get().is_some()
    })));
}

pub fn healthy() {
    let (rt, context) = realm();
    context.with(|ctx| assert_eq!(ctx.eval::<i32, _>("21 * 2").unwrap(), 42));
    drop(context);
    drop(rt);
}

pub fn clear_exception(ctx: &Ctx<'_>) {
    // Do not stringify, inspect a source error property, or reset uncatchability.
    drop(ctx.catch());
}

#[cfg(test)]
mod tests;
