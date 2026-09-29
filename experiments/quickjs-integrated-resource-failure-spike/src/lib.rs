//! Non-production integration only. All provider members are test-private oracles.
#![deny(unsafe_code)]
mod allocator;
mod ffi;
use admission::{Gate, Reason};
use allocator::{Accounting, Tracked};
use rquickjs::{
    Context, Ctx, Function, Module, Object, Promise, Runtime, Value,
    loader::{BuiltinLoader, BuiltinResolver},
    module::{Exports, ModuleDef},
    promise::PromiseState,
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    sync::atomic::{AtomicU64, Ordering},
};
use value_boundary::{Boundary, Failure, Limits, Portable};

const HEAP: usize = 8 * 1024 * 1024;
const STACK: usize = 256 * 1024;
static GENERATION: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Phase {
    #[default]
    Setup,
    Init,
    Incoming,
    Call,
    Job,
    Outgoing,
    Retired,
}
#[derive(Default)]
pub struct State {
    gate: Rc<Gate>,
    phase: Cell<Phase>,
    active: Cell<bool>,
    notes: RefCell<Vec<i32>>,
    polls: Cell<usize>,
    stop_after: Cell<Option<(usize, Reason)>>,
    jobs: Cell<usize>,
    action: Cell<usize>,
}
impl State {
    fn latch(&self, reason: Reason) {
        if self.gate.latch.get().is_none() {
            self.gate.latch.set(Some(reason));
        }
    }
    fn accepts(&self, actual: u64, token: u64) -> bool {
        actual == token && self.active.get() && self.gate.latch.get().is_none()
    }
    fn observe_requested_stop(&self) {
        if let Some(reason) = self.gate.pending.take() {
            self.latch(reason);
        }
    }
    fn poll(&self) -> bool {
        self.polls.set(self.polls.get() + 1);
        if let Some((n, reason)) = self.stop_after.get()
            && self.polls.get() >= n
        {
            self.latch(reason);
        }
        self.observe_requested_stop();
        self.gate.latch.get().is_some()
    }
}
#[derive(Clone, Debug)]
pub struct Config {
    pub init_stop: Option<Reason>,
    pub operation: &'static str,
    pub audit_roots: bool,
    pub fail_allocation: Option<(Phase, usize)>,
    pub single_allocation: Option<usize>,
    pub tight_heap_phase: Option<Phase>,
    /// Diagnostic override at call entry; 0 disables only the engine ceiling.
    /// The independent finite allocator budget remains enabled.
    pub call_heap_limit: Option<usize>,
    pub deliver_pending: bool,
    pub input: Portable,
    pub limits: Limits,
    pub empty_queue_stop: Reason,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            init_stop: None,
            operation: "home",
            audit_roots: false,
            fail_allocation: None,
            single_allocation: None,
            tight_heap_phase: None,
            call_heap_limit: None,
            deliver_pending: false,
            input: Portable::Null,
            limits: Limits::default(),
            empty_queue_stop: Reason::Timeout,
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub enum Outcome {
    Value(Portable),
    SourceError,
    Invalid,
    Stop(Reason),
    Preflight,
}
#[derive(Debug)]
pub struct Report {
    pub outcome: Outcome,
    pub notes: Vec<i32>,
    pub jobs: usize,
    pub polls: usize,
    pub queued_at_retirement: bool,
    pub compiles: usize,
    pub rejected: usize,
    pub allocation_rejects: usize,
    pub allocation_peak: usize,
    pub phase_allocation_calls: usize,
    pub late_rejected: usize,
    pub action: usize,
    pub destroyed: bool,
    pub generation: u64,
    pub disposed: bool,
}
struct Capture;
impl ModuleDef for Capture {
    fn evaluate<'js>(ctx: &Ctx<'js>, _: &Exports<'js>) -> rquickjs::Result<()> {
        ctx.remove_userdata::<Function<'js>>()
            .unwrap()
            .unwrap()
            .call(())
    }
}
struct Delivery<'js> {
    generation: u64,
    resolver: Option<Function<'js>>,
    state: Rc<State>,
}
impl Delivery<'_> {
    fn deliver(&self, generation: u64) -> bool {
        if !self.state.accepts(self.generation, generation) {
            return false;
        }
        self.resolver
            .as_ref()
            .is_some_and(|f| f.call::<_, ()>((42,)).is_ok())
    }
    fn close(&mut self) {
        self.state.active.set(false);
        self.resolver.take();
    }
}
fn convert_failure(f: Failure) -> Outcome {
    match f {
        Failure::Invalid => Outcome::Invalid,
        Failure::ResourceLimit => Outcome::Stop(Reason::ResourceLimit),
        Failure::Engine => Outcome::SourceError,
    }
}
fn frozen_context<'js>(
    ctx: &Ctx<'js>,
    boundary: &mut Boundary<'js>,
    state: &Rc<State>,
    pending: Promise<'js>,
) -> Object<'js> {
    let log = Object::new_proto(ctx.clone(), None).unwrap();
    let s = state.clone();
    log.set(
        "note",
        Function::new(ctx.clone(), move |n: i32| s.notes.borrow_mut().push(n)).unwrap(),
    )
    .unwrap();
    for (name, reason) in [("cancel", Reason::Cancelled), ("expire", Reason::Timeout)] {
        let s = state.clone();
        log.set(
            name,
            Function::new(ctx.clone(), move || {
                s.gate.pending.set(Some(reason));
            })
            .unwrap(),
        )
        .unwrap();
    }
    let s = state.clone();
    log.set(
        "action",
        Function::new(ctx.clone(), move || {
            if s.active.get() && s.gate.latch.get().is_none() {
                s.action.set(s.action.get() + 1);
            }
        })
        .unwrap(),
    )
    .unwrap();
    // Test-private native completion placeholder, never a public service signature.
    log.set("pending", pending).unwrap();
    let services = Object::new_proto(ctx.clone(), None).unwrap();
    services.set("log", log.clone()).unwrap();
    let context = Object::new_proto(ctx.clone(), None).unwrap();
    context.set("services", services.clone()).unwrap();
    let freeze: Function = ctx.eval("Object.freeze").unwrap();
    for helper in [&log, &services, &context] {
        boundary.exclude_helper(helper.as_value()).unwrap();
        freeze.call::<_, ()>((helper.clone(),)).unwrap();
    }
    context
}

/// Fixed, immutable single-module fixtures. No filesystem/package resolver claim.
/// Late token optionally belongs to a previous invocation, never a JS handle.
pub fn invoke(source: &str, config: Config, late_token: Option<u64>) -> Report {
    let generation = GENERATION.fetch_add(1, Ordering::Relaxed);
    let state = Rc::new(State::default());
    state.gate.bound.set(admission::L_MAX);
    let accounting = Rc::new(Accounting::default());
    accounting.total.set(4 * 1024 * 1024); // conservative adapter cap below engine ceiling
    let rt = Runtime::new_with_alloc(Tracked(accounting.clone(), state.clone())).unwrap();
    rt.set_memory_limit(HEAP);
    rt.set_max_stack_size(STACK);
    let weak = rt.weak();
    let s = state.clone();
    rt.set_interrupt_handler(Some(Box::new(move || s.poll())));
    // Only fixed trusted root imports private capture; package fixtures contain no imports.
    rt.set_loader(
        BuiltinResolver::default()
            .with_module("entry.js")
            .with_module("@host/capture"),
        BuiltinLoader::default(),
    );
    let context = Context::full(&rt).unwrap();
    let mut late_rejected = 0;
    let mut queued = false;
    let mut outcome = context.with(|ctx| {
        ffi::leak_checks(&ctx);
        let mut boundary = Boundary::new(&ctx, config.limits).unwrap();
        let witnesses = config.audit_roots.then(|| {
            let factory: Function = ctx.eval(include_str!("../../quickjs-global-surface-spike/src/traverse.js")).unwrap();
            let allow: Value = ctx.eval(admission::GLOBALS).unwrap();
            let globals: Function = factory.call((allow,)).unwrap();
            let natives: Value = ctx.eval("[[RegExp,'RegExp'],[RegExp.prototype.compile,'compile'],[RegExp.prototype[Symbol.split],'split'],[RegExp.prototype[Symbol.matchAll],'matchAll'],[String.prototype.match,'String.match'],[String.prototype.matchAll,'String.matchAll'],[String.prototype.search,'search']]").unwrap();
            let factory: Function = ctx.eval(include_str!("../../quickjs-regexp-admission-impl-spike/src/reach.js")).unwrap();
            let regexp: Function = factory.call((natives,)).unwrap();
            let before: String = regexp.call((false,)).unwrap();
            assert!(before.starts_with("forbidden:"));
            (globals, regexp)
        });
        let installer = admission::capture(&ctx);
        admission::harden(&ctx); // actually CALLS global hardener with policy allowlists
        admission::install(&ctx, installer, &state.gate);
        if let Some((globals, regexp)) = witnesses {
            for witness in [globals, regexp] {
                let result: String = witness.call((true,)).unwrap();
                assert!(result.starts_with("clean:"), "{result}");
                println!("INTEGRATED REACH {result}");
            }
        }
        let promise_proto: Object = ctx.eval("Promise.prototype").unwrap();
        let (pending, resolve, reject) = Promise::new(&ctx).unwrap();
        drop(reject);
        let mut delivery = Delivery {
            generation,
            resolver: Some(resolve),
            state: state.clone(),
        };
        let control = frozen_context(&ctx, &mut boundary, &state, pending);
        let candidate = (|| {
            if admission::preflight::preflight(source, admission::L_MAX).is_err() {
                return Err(Outcome::Preflight);
            }
            let entry = Module::declare(ctx.clone(), "entry.js", source)
                .map_err(|_| Outcome::SourceError)?;
            let captured: Rc<RefCell<Option<Function>>> = Rc::default();
            let saved = captured.clone();
            let m = entry.clone();
            let operation = config.operation;
            let capture = Function::new(ctx.clone(), move || -> rquickjs::Result<()> {
                *saved.borrow_mut() = Some(m.get(operation)?);
                Ok(())
            })
            .unwrap();
            ctx.store_userdata(capture).unwrap();
            Module::declare_def::<Capture, _>(ctx.clone(), "@host/capture").unwrap();
            let root = Module::declare(
                ctx.clone(),
                "@host/root",
                "import '@host/capture'; import 'entry.js';",
            )
            .map_err(|_| Outcome::SourceError)?;
            accounting.fail.set(config.fail_allocation);
            state.phase.set(Phase::Init);
            if let Some(reason) = config.init_stop {
                state.stop_after.set(Some((state.polls.get() + 2, reason)));
            }
            let (_, result) = root.eval().map_err(|_| Outcome::SourceError)?;
            result
                .result::<()>()
                .ok_or(Outcome::SourceError)?
                .map_err(|_| Outcome::SourceError)?;
            if ffi::pending(&ctx) {
                return Err(Outcome::SourceError);
            }
            let function = captured.borrow_mut().take().ok_or(Outcome::SourceError)?;
            if entry
                .get::<_, Value>(operation)
                .map_err(|_| Outcome::SourceError)?
                != *function.as_value()
            {
                return Err(Outcome::SourceError);
            }
            state.phase.set(Phase::Incoming);
            if config.tight_heap_phase == Some(Phase::Incoming) {
                ffi::heap_limit(&ctx, 1);
            }
            let (input, _) = boundary.incoming(&config.input).map_err(convert_failure)?;
            state.observe_requested_stop();
            if let Some(reason) = state.gate.latch.get() {
                return Err(Outcome::Stop(reason));
            }
            state.active.set(true);
            if let Some(old) = late_token {
                assert!(!delivery.deliver(old));
                late_rejected += 1;
            }
            state.phase.set(Phase::Call);
            if let Some(bytes) = config.call_heap_limit {
                ffi::heap_limit(&ctx, bytes);
            }
            if let Some(single) = config.single_allocation {
                accounting.single.set(single);
            }
            let value: Value = function
                .call((input, control.clone()))
                .map_err(|_| Outcome::SourceError)?;
            state.observe_requested_stop();
            if let Some(reason) = state.gate.latch.get() {
                return Err(Outcome::Stop(reason));
            }
            if let Some(promise) = value.as_promise() {
                if promise.as_object().unwrap().get_prototype().as_ref() != Some(&promise_proto) {
                    return Err(Outcome::Invalid);
                }
                while promise.state() == PromiseState::Pending {
                    state.observe_requested_stop();
            if let Some(reason) = state.gate.latch.get() {
                        return Err(Outcome::Stop(reason));
                    }
                    if !ffi::pending(&ctx) && config.deliver_pending {
                        assert!(delivery.deliver(generation));
                        continue;
                    }
                    if !ffi::pending(&ctx) || state.jobs.get() >= 64 {
                        state.latch(config.empty_queue_stop);
                        return Err(Outcome::Stop(config.empty_queue_stop));
                    }
                    state.phase.set(Phase::Job);
                    state.jobs.set(state.jobs.get() + 1);
                    if !ffi::job(&ctx) {
                        return Err(Outcome::SourceError);
                    }
                }
                promise
                    .result::<Value>()
                    .unwrap()
                    .map_err(|_| Outcome::SourceError)
            } else {
                Ok(value)
            }
        })();
        // Mandatory gate closure BEFORE conversion; no final job or resolver call.
        state.observe_requested_stop();
        delivery.close();
        assert!(!delivery.deliver(generation));
        late_rejected += 1;
        queued = ffi::pending(&ctx);
        state.phase.set(Phase::Outgoing);
        if config.tight_heap_phase == Some(Phase::Outgoing) {
            ffi::heap_limit(&ctx, 1);
        }
        let result = if let Some(reason) = state.gate.latch.get() {
            Outcome::Stop(reason)
        } else {
            match candidate {
                Ok(value) => match boundary.outgoing(&value) {
                    Ok((v, _)) => Outcome::Value(v),
                    Err(e) => convert_failure(e),
                },
                Err(e) => e,
            }
        };
        // Callback might remain on a failed link/init; release its Module root.
        ctx.remove_userdata::<Function>().unwrap();
        drop(ctx.catch()); // no formatting/properties/source hooks
        result
    });
    state.phase.set(Phase::Retired);
    state.active.set(false);
    drop(context);
    drop(rt);
    let destroyed = weak.try_ref().is_none();
    assert!(destroyed);
    assert_eq!(
        accounting.live.get(),
        0,
        "allocator blocks remain after retirement"
    );
    assert!(!state.accepts(generation, generation));
    late_rejected += 1; // same native-event gate after all JS handles are destroyed
    if let Some(reason) = state.gate.latch.get() {
        outcome = Outcome::Stop(reason);
    }
    let disposed = matches!(outcome, Outcome::Stop(_));
    Report {
        outcome,
        notes: state.notes.borrow().clone(),
        jobs: state.jobs.get(),
        polls: state.polls.get(),
        queued_at_retirement: queued,
        compiles: state.gate.compiles(),
        rejected: state.gate.rejected.get(),
        allocation_rejects: accounting.rejects.get(),
        allocation_peak: accounting.peak.get(),
        phase_allocation_calls: accounting.phase_calls.get(),
        late_rejected,
        action: state.action.get(),
        destroyed,
        generation,
        disposed,
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod failure_classification;
