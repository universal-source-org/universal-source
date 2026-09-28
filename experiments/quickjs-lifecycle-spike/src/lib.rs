//! Non-production Model B probes. No loader, dispatcher, service API or full converter.
//!
//! Scoped values cannot escape `Context::with` without explicit rooting:
//! ```compile_fail
//! use rquickjs::{Context, Runtime, Value};
//! let rt = Runtime::new().unwrap();
//! let c = Context::full(&rt).unwrap();
//! let value = c.with(|ctx| ctx.eval::<Value, _>("({})").unwrap());
//! drop(value);
//! ```
//! With the pinned features Runtime is not Send:
//! ```compile_fail
//! use rquickjs::Runtime;
//! fn send<T: Send>() {}
//! send::<Runtime>();
//! ```
//! Context is not Send either:
//! ```compile_fail
//! use rquickjs::Context;
//! fn send<T: Send>() {}
//! send::<Context>();
//! ```

#[cfg(test)]
mod extraction;
#[cfg(test)]
mod tests;
