#![forbid(unsafe_code)]
//! Non-production prerequisite probe: this is NOT a complete value converter.
//!
//! The pinned public C class query cannot be called from safe Rust. This test
//! deliberately omits an unsafe block; no C function is executed.
//!
//! ```compile_fail,E0133
//! #![forbid(unsafe_code)]
//! fn class(value: &rquickjs::Value<'_>) -> rquickjs::qjs::JSClassID {
//!     rquickjs::qjs::JS_GetClassID(value.as_raw())
//! }
//! ```
//!
//! The public C descriptor reader also requires unsafe Rust. This only checks
//! the compiler diagnostic; no uninitialized descriptor is read or FFI called.
//!
//! ```compile_fail,E0133
//! #![forbid(unsafe_code)]
//! fn descriptor(ctx: &rquickjs::Ctx<'_>, value: &rquickjs::Value<'_>,
//!               atom: rquickjs::qjs::JSAtom) {
//!     let mut out = std::mem::MaybeUninit::<rquickjs::qjs::JSPropertyDescriptor>::uninit();
//!     rquickjs::qjs::JS_GetOwnProperty(ctx.as_raw().as_ptr(), out.as_mut_ptr(), value.as_raw(), atom);
//! }
//! ```
#[cfg(test)]
mod tests;
