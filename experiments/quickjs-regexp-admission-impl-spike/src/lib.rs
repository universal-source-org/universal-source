//! Non-production RegExp admission feasibility spike over pinned QuickJS-NG:
//! a host-gated RegExp facade, literal preflight, calibration and matching probes.
//! Not production code, an engine or parser selection, or a normative policy.
#![deny(unsafe_code)]

mod ffi;
pub mod preflight;

use rquickjs::{Context, Ctx, FromJs, Function, Runtime, Value};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

pub const HEAP: usize = 8 * 1024 * 1024;
pub const STACK: usize = 128 * 1024;
/// Candidate host policy for this pin only, in UTF-16 code units of the pattern.
pub const L_MAX: usize = 4096;

const INVENTORY: &str = include_str!("../../quickjs-global-surface-spike/src/inventory.js");
const PRISTINE: &str =
    include_str!("../../quickjs-global-surface-spike/src/pristine-inventory.json");
pub const GLOBALS: &str =
    include_str!("../../quickjs-global-surface-spike/src/allowed-globals.json");
const INTRINSICS: &str =
    include_str!("../../quickjs-global-surface-spike/src/allowed-intrinsics.json");
const DYNAMIC_HARDEN: &str = include_str!("../../quickjs-dynamic-code-spike/src/harden.js");
const GLOBAL_HARDEN: &str = include_str!("../../quickjs-global-surface-spike/src/harden.js");
const GATE: &str = include_str!("gate.js");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reason {
    Cancelled,
    Timeout,
    ResourceLimit,
}

/// Owner-thread trusted host state, never reachable from source. The latch alone
/// classifies a stop; exception values and text are never consulted.
#[derive(Default)]
pub struct Gate {
    pub bound: Cell<usize>,
    /// Simulated host deadline/cancellation decision, observed at the next check.
    pub pending: Cell<Option<Reason>>,
    /// Simulated clock: the deadline passes at this gate entry.
    pub expire_at_entry: Cell<Option<usize>>,
    pub latch: Cell<Option<Reason>>,
    pub entries: Cell<usize>,
    /// Source-derived patterns handed to the native compiler.
    pub admitted: Cell<usize>,
    /// Already-admitted internal sources recompiled with new flags.
    pub readmitted: Cell<usize>,
    pub rejected: Cell<usize>,
    /// Test-only isolation: a set latch no longer makes engine polls interrupt.
    pub quiet_latch: Cell<bool>,
    pub callbacks: Cell<usize>,
    /// Test-only markers written by fixtures through `note`.
    pub notes: RefCell<Vec<String>>,
}

impl Gate {
    fn latch(&self, reason: Reason) {
        if self.latch.get().is_none() {
            self.latch.set(Some(reason));
        }
    }

    /// Gate entry: false means the facade must throw its uncatchable stop.
    pub fn check(&self) -> bool {
        if self.latch.get().is_some() {
            return false;
        }
        let entry = self.entries.get() + 1;
        self.entries.set(entry);
        if self.expire_at_entry.get() == Some(entry) {
            self.pending.set(Some(Reason::Timeout));
        }
        if let Some(reason) = self.pending.take() {
            self.latch(reason);
            return false;
        }
        true
    }

    pub fn admit(&self, units: f64) -> bool {
        if !self.check() {
            return false;
        }
        if units.is_nan() || units > self.bound.get() as f64 {
            self.rejected.set(self.rejected.get() + 1);
            self.latch(Reason::ResourceLimit);
            return false;
        }
        self.admitted.set(self.admitted.get() + 1);
        true
    }

    pub fn readmit(&self) -> bool {
        if !self.check() {
            return false;
        }
        self.readmitted.set(self.readmitted.get() + 1);
        true
    }

    fn interrupt(&self) -> bool {
        self.callbacks.set(self.callbacks.get() + 1);
        if self.latch.get().is_some() {
            return !self.quiet_latch.get();
        }
        if let Some(reason) = self.pending.take() {
            self.latch(reason);
            return true;
        }
        false
    }

    pub fn compiles(&self) -> usize {
        self.admitted.get() + self.readmitted.get()
    }
}

/// Fields drop in order: context, then gate, then runtime.
pub struct Realm {
    pub context: Context,
    pub gate: Rc<Gate>,
    pub rt: Runtime,
}

pub fn runtime(gate: &Rc<Gate>) -> Runtime {
    let rt = Runtime::new().unwrap();
    rt.set_memory_limit(HEAP);
    rt.set_max_stack_size(STACK);
    let state = gate.clone();
    rt.set_interrupt_handler(Some(Box::new(move || state.interrupt())));
    rt
}

pub fn eval<'js, T: FromJs<'js>>(ctx: &Ctx<'js>, text: &str) -> T {
    match ctx.eval(text) {
        Ok(v) => v,
        Err(e) => panic!(
            "trusted evaluation failed: {e:?}; exception: {:?}",
            ctx.catch()
        ),
    }
}

pub fn inventory_matches(ctx: &Ctx<'_>) -> bool {
    let actual: String = eval(ctx, INVENTORY);
    let expected: String = eval(ctx, &format!("JSON.stringify({PRISTINE})"));
    actual == expected
}

/// Captures the native surface on a pristine realm; returns the installer.
pub fn capture<'js>(ctx: &Ctx<'js>) -> Function<'js> {
    assert!(inventory_matches(ctx), "unexpected pristine drift");
    eval(ctx, GATE)
}

/// Existing dynamic-code and global-surface hardening, the global step actually
/// called with its policy arguments.
pub fn harden(ctx: &Ctx<'_>) {
    eval::<()>(ctx, DYNAMIC_HARDEN);
    let harden: Function = eval(ctx, GLOBAL_HARDEN);
    let globals: Value = eval(ctx, GLOBALS);
    let intrinsics: Value = eval(ctx, &format!("({INTRINSICS})"));
    harden.call::<_, ()>((globals, intrinsics)).unwrap();
}

pub fn install<'js>(ctx: &Ctx<'js>, installer: Function<'js>, gate: &Rc<Gate>) {
    let (g1, g2, g3) = (gate.clone(), gate.clone(), gate.clone());
    let checkpoint = Function::new(ctx.clone(), move || g1.check()).unwrap();
    let admit = Function::new(ctx.clone(), move |units: f64| g2.admit(units)).unwrap();
    let readmit = Function::new(ctx.clone(), move || g3.readmit()).unwrap();
    let brand = Function::new(ctx.clone(), |v: Value<'js>| ffi::has_regexp_brand(&v)).unwrap();
    let constructor = Function::new(ctx.clone(), |v: Value<'js>| v.is_constructor()).unwrap();
    let mark = Function::new(ctx.clone(), |ctx: Ctx<'js>, v: Value<'js>| {
        ffi::mark_uncatchable(&ctx, &v)
    })
    .unwrap();
    installer
        .call::<_, ()>((checkpoint, admit, readmit, brand, constructor, mark))
        .unwrap();
}

/// Test-only marker sink for fixtures; installed after hardening.
pub fn install_note(ctx: &Ctx<'_>, gate: &Rc<Gate>) {
    let g = gate.clone();
    let note = Function::new(ctx.clone(), move |s: String| g.notes.borrow_mut().push(s)).unwrap();
    ctx.globals().set("note", note).unwrap();
}

/// Fresh limited realm: capture, hardening, facade installation, then `note`.
pub fn hardened_realm(bound: usize) -> Realm {
    let gate = Rc::new(Gate::default());
    gate.bound.set(bound);
    let rt = runtime(&gate);
    let context = Context::full(&rt).unwrap();
    context.with(|ctx| {
        let installer = capture(&ctx);
        harden(&ctx);
        install(&ctx, installer, &gate);
        install_note(&ctx, &gate);
    });
    Realm { context, gate, rt }
}

/// `"\\k<a>".repeat(n) + "(?<a>x)"`: the preserved forward-reference family.
pub fn forward_reference_pattern(references: usize) -> String {
    format!("{}(?<a>x)", "\\k<a>".repeat(references))
}

#[cfg(test)]
mod tests;
