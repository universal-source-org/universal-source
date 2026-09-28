//! Non-production F1 probes. Passing tests include counterexamples, not conformance.

#[cfg(test)]
mod tests {
    use boa_engine::{
        Context, JsError, JsResult, JsValue, Source,
        builtins::promise::{OperationType, Promise, PromiseState},
        context::HostHooks,
        job::{Job, JobCallback, JobExecutor, PromiseJob},
        object::{
            JsObject,
            builtins::{JsFunction, JsPromise},
        },
    };
    use std::{
        cell::{Cell, RefCell},
        collections::{BTreeSet, VecDeque},
        rc::Rc,
    };

    const A: u64 = 1;
    const B: u64 = 2;

    #[derive(Clone, Copy)]
    enum Mode {
        Observe,
        // Deliberately violate call_job_callback's Call requirement to measure fallout.
        SkipWithUndefined,
        SkipWithError,
    }

    struct Hooks {
        current: Cell<Option<u64>>,
        revoked: RefCell<BTreeSet<u64>>,
        made: RefCell<Vec<u64>>,
        calls: RefCell<Vec<(u64, u64, bool)>>,
        rejections: Cell<usize>,
        mode: Mode,
    }

    impl HostHooks for Hooks {
        fn make_job_callback(&self, callback: JsFunction, _: &mut Context) -> JobCallback {
            let owner = self
                .current
                .get()
                .expect("test registration has a host owner");
            self.made.borrow_mut().push(owner);
            JobCallback::new(callback, owner)
        }

        fn call_job_callback(
            &self,
            job: &JobCallback,
            this: &JsValue,
            args: &[JsValue],
            ctx: &mut Context,
        ) -> JsResult<JsValue> {
            let owner = *job.host_defined().downcast_ref::<u64>().unwrap();
            let current = self.current.get().unwrap();
            let skip =
                self.revoked.borrow().contains(&owner) && !matches!(self.mode, Mode::Observe);
            self.calls.borrow_mut().push((owner, current, skip));
            if skip {
                return match self.mode {
                    Mode::SkipWithUndefined => Ok(JsValue::undefined()),
                    Mode::SkipWithError => Err(JsError::from_opaque(JsValue::from(99))),
                    Mode::Observe => unreachable!(),
                };
            }
            job.callback().call(this, args, ctx)
        }

        fn promise_rejection_tracker(
            &self,
            _: &JsObject<Promise>,
            op: OperationType,
            _: &mut Context,
        ) {
            if matches!(op, OperationType::Reject) {
                self.rejections.set(self.rejections.get() + 1);
            }
        }
    }

    struct Queue {
        hooks: Rc<Hooks>,
        // Enqueue-time attribution is intentionally tested, not assumed registration ownership.
        jobs: RefCell<VecDeque<(u64, PromiseJob)>>,
        enqueued: RefCell<Vec<u64>>,
    }

    impl Queue {
        fn step(&self, ctx: &mut Context) -> bool {
            let entry = self.jobs.borrow_mut().pop_front();
            if let Some((_, job)) = entry {
                job.call(ctx).unwrap();
                true
            } else {
                false
            }
        }

        fn discard(&self, owner: u64) {
            self.jobs.borrow_mut().retain(|(tag, _)| *tag != owner);
        }
    }

    impl JobExecutor for Queue {
        fn enqueue_job(self: Rc<Self>, job: Job, _: &mut Context) {
            let Job::PromiseJob(job) = job else {
                panic!("unexpected job type in fixed-source probe")
            };
            let owner = self.hooks.current.get().unwrap();
            self.enqueued.borrow_mut().push(owner);
            self.jobs.borrow_mut().push_back((owner, job));
        }

        fn run_jobs(self: Rc<Self>, ctx: &mut Context) -> JsResult<()> {
            for _ in 0..100 {
                if !self.step(ctx) {
                    return Ok(());
                }
            }
            panic!("fixed-source probe exceeded 100-job backstop")
        }
    }

    struct Probe {
        ctx: Context,
        hooks: Rc<Hooks>,
        queue: Rc<Queue>,
    }

    impl Probe {
        fn new(mode: Mode) -> Self {
            let hooks = Rc::new(Hooks {
                current: Cell::new(None),
                revoked: RefCell::new(BTreeSet::new()),
                made: RefCell::new(Vec::new()),
                calls: RefCell::new(Vec::new()),
                rejections: Cell::new(0),
                mode,
            });
            let queue = Rc::new(Queue {
                hooks: hooks.clone(),
                jobs: RefCell::new(VecDeque::new()),
                enqueued: RefCell::new(Vec::new()),
            });
            let ctx = Context::builder()
                .host_hooks(hooks.clone())
                .job_executor(queue.clone())
                .build()
                .unwrap();
            Self { ctx, hooks, queue }
        }
        fn begin(&self, owner: u64) {
            assert!(self.hooks.current.replace(Some(owner)).is_none());
        }
        fn end(&self) {
            let owner = self.hooks.current.take().unwrap();
            self.hooks.revoked.borrow_mut().insert(owner);
            self.queue.discard(owner);
        }
        fn eval(&mut self, text: &str) -> JsValue {
            self.ctx.eval(Source::from_bytes(text)).unwrap()
        }
        fn yes(&mut self, text: &str) {
            assert_eq!(self.eval(text).as_boolean(), Some(true), "{text}");
        }
        fn state(&mut self, name: &str) -> PromiseState {
            JsPromise::from_object(self.eval(name).as_object().unwrap())
                .unwrap()
                .state()
        }
        fn pending(&mut self, name: &str) {
            assert!(matches!(self.state(name), PromiseState::Pending), "{name}");
        }
        fn undefined(&mut self, name: &str) {
            assert!(
                matches!(self.state(name), PromiseState::Fulfilled(v) if v.is_undefined()),
                "{name}"
            );
        }
        fn step(&mut self) -> bool {
            self.queue.step(&mut self.ctx)
        }
        fn drain(&mut self) {
            self.queue.clone().run_jobs(&mut self.ctx).unwrap();
        }
    }

    const RETAINED: &str = "globalThis.marker = 0; globalThis.release = undefined; globalThis.retained = new Promise(r => { release = r; });";

    #[test]
    fn supported_observation_preserves_registration_tags_but_executes_old_then_and_await() {
        let mut p = Probe::new(Mode::Observe);
        p.begin(A);
        p.eval(RETAINED);
        p.eval("retained.then(() => marker++); (async () => { await retained; marker++; })();");
        assert_eq!(&*p.hooks.made.borrow(), &[A, A, A]); // .then plus await fulfill/reject
        assert!(p.queue.jobs.borrow().is_empty());
        p.end();
        p.begin(B);
        p.eval("release(7)");
        p.drain();
        p.yes("marker === 2");
        assert_eq!(&*p.hooks.calls.borrow(), &[(A, B, false), (A, B, false)]);
        assert!(p.queue.enqueued.borrow().iter().all(|tag| *tag == B));
    }

    #[test]
    fn skip_then_blocks_body_but_fulfills_child_and_b_observes_it() {
        let mut p = Probe::new(Mode::SkipWithUndefined);
        p.begin(A);
        p.eval(RETAINED);
        p.eval("globalThis.state = 40; globalThis.child = retained.then(() => { marker++; return 42; });");
        p.pending("child");
        p.end();
        p.begin(B);
        p.eval("state += 2; globalThis.bRan = 0; child.then(v => { if (v === undefined) bRan++; }); release(7);");
        p.drain();
        p.yes("marker === 0 && state === 42 && bRan === 1");
        p.undefined("child");
        assert_eq!(&*p.hooks.calls.borrow(), &[(A, B, true), (B, B, false)]);
        assert_eq!(p.hooks.rejections.get(), 0);
        assert!(
            matches!(p.state("retained"), PromiseState::Fulfilled(v) if v.as_number() == Some(7.0))
        );
    }

    #[test]
    fn queued_a_jobs_can_be_dropped_before_execution_without_dropping_b_jobs() {
        let mut p = Probe::new(Mode::Observe);
        p.begin(A);
        p.eval(
            "globalThis.marker = 0; globalThis.child = Promise.resolve(7).then(() => marker++);",
        );
        assert_eq!(p.queue.jobs.borrow().len(), 1);
        p.end();
        p.begin(B);
        p.eval("globalThis.bRan = 0; Promise.resolve().then(() => bRan++);");
        p.drain();
        p.yes("marker === 0 && bRan === 1");
        p.pending("child");
        assert_eq!(&*p.hooks.calls.borrow(), &[(B, B, false)]);
    }

    #[test]
    fn enqueue_attribution_loses_future_registration_owner() {
        let mut p = Probe::new(Mode::Observe);
        p.begin(A);
        p.eval(RETAINED);
        p.eval("retained.then(() => marker++);");
        p.end();
        p.begin(B);
        p.eval("release(7)");
        assert_eq!(&*p.queue.enqueued.borrow(), &[B]);
        p.queue.discard(A);
        assert_eq!(p.queue.jobs.borrow().len(), 1);
        p.drain();
        p.yes("marker === 1");
        assert_eq!(&*p.hooks.calls.borrow(), &[(A, B, false)]);
    }

    #[test]
    fn same_retained_promise_can_hold_a_and_b_owned_reactions() {
        let mut p = Probe::new(Mode::SkipWithUndefined);
        p.begin(A);
        p.eval(RETAINED);
        p.eval("globalThis.oldChild = retained.then(() => marker += 100);");
        p.end();
        p.begin(B);
        p.eval(
            "globalThis.newChild = retained.then(v => { marker += v; return 42; }); release(7);",
        );
        assert_eq!(&*p.queue.enqueued.borrow(), &[B, B]);
        p.drain();
        p.yes("marker === 7");
        p.undefined("oldChild");
        assert!(
            matches!(p.state("newChild"), PromiseState::Fulfilled(v) if v.as_number() == Some(42.0))
        );
        assert_eq!(&*p.hooks.calls.borrow(), &[(A, B, true), (B, B, false)]);
    }

    #[test]
    fn skipped_promise_chain_settles_both_children_without_running_old_bodies() {
        let mut p = Probe::new(Mode::SkipWithUndefined);
        p.begin(A);
        p.eval(RETAINED);
        p.eval("globalThis.child = retained.then(() => { marker++; return 11; }); globalThis.tail = child.then(() => { marker++; return 12; });");
        assert_eq!(&*p.hooks.made.borrow(), &[A, A]);
        p.end();
        p.begin(B);
        p.eval("release(7)");
        p.drain();
        p.yes("marker === 0");
        p.undefined("child");
        p.undefined("tail");
        assert_eq!(&*p.hooks.calls.borrow(), &[(A, B, true), (A, B, true)]);
    }

    #[test]
    fn skipped_nested_await_leaves_inner_and_outer_pending_without_finally() {
        let mut p = Probe::new(Mode::SkipWithUndefined);
        p.begin(A);
        p.eval(RETAINED);
        p.eval("globalThis.cleanup = 0; async function inner() { try { await retained; marker++; } finally { cleanup++; } } async function outer() { await innerResult; marker++; } globalThis.innerResult = inner(); globalThis.outerResult = outer();");
        assert_eq!(&*p.hooks.made.borrow(), &[A, A, A, A]);
        p.end();
        p.begin(B);
        p.eval("globalThis.bRan = 0; innerResult.then(() => bRan++); outerResult.then(() => bRan++); Promise.resolve().then(() => bRan++); release(7);");
        p.drain();
        p.yes("marker === 0 && cleanup === 0 && bRan === 1");
        p.pending("innerResult");
        p.pending("outerResult");
        assert!(p.hooks.calls.borrow().contains(&(A, B, true)));
        assert_eq!(p.hooks.rejections.get(), 0);
    }

    #[test]
    fn terminal_promise_stops_queued_work_but_not_future_reactions() {
        let mut p = Probe::new(Mode::Observe);
        p.begin(A);
        p.eval(RETAINED);
        p.eval("globalThis.terminal = Promise.resolve().then(() => { Promise.resolve().then(() => marker += 10); retained.then(() => marker += 100); return 42; });");
        assert!(p.step());
        assert!(
            matches!(p.state("terminal"), PromiseState::Fulfilled(v) if v.as_number() == Some(42.0))
        );
        assert_eq!(p.queue.jobs.borrow().len(), 1);
        p.end();
        p.begin(B);
        p.eval("release(7)");
        p.drain();
        p.yes("marker === 100");
    }

    #[test]
    fn pending_return_keeps_host_in_control_for_synthetic_cancel_and_deadline() {
        for terminal in ["cancelled", "deadline"] {
            let mut p = Probe::new(Mode::Observe);
            p.begin(A);
            p.eval("globalThis.returned = new Promise(() => {});");
            p.pending("returned");
            assert!(!p.step());
            let host_terminal = Some(terminal); // Controlled checkpoint, not a real timer/service.
            assert!(host_terminal.is_some());
            p.end();
            p.pending("returned");
            assert!(p.hooks.current.get().is_none());
        }
    }

    #[test]
    fn absent_handlers_have_no_registration_tag_and_propagate_under_b() {
        let mut p = Probe::new(Mode::SkipWithUndefined);
        p.begin(A);
        p.eval(RETAINED);
        p.eval("globalThis.child = retained.then();");
        assert!(p.hooks.made.borrow().is_empty());
        p.end();
        p.begin(B);
        p.eval("release(7)");
        p.drain();
        assert!(p.hooks.calls.borrow().is_empty());
        assert!(
            matches!(p.state("child"), PromiseState::Fulfilled(v) if v.as_number() == Some(7.0))
        );
        assert_eq!(&*p.queue.enqueued.borrow(), &[B]);
    }

    #[test]
    fn error_substitution_rejects_child_and_reports_unhandled_rejection() {
        let mut p = Probe::new(Mode::SkipWithError);
        p.begin(A);
        p.eval(RETAINED);
        p.eval("globalThis.child = retained.then(() => marker++);");
        p.end();
        p.begin(B);
        p.eval("release(7)");
        p.drain();
        p.yes("marker === 0");
        assert!(
            matches!(p.state("child"), PromiseState::Rejected(v) if v.as_number() == Some(99.0))
        );
        assert_eq!(p.hooks.rejections.get(), 1);
        p.eval("globalThis.observed = 0; child.catch(e => { observed = e; });");
        p.drain();
        p.yes("observed === 99");
    }

    #[test]
    #[should_panic(expected = "handlerResult is not an abrupt completion")]
    fn error_substitution_for_await_violates_engine_invariant() {
        // Expected counterexample. Unwind drops this probe; never reuse the context.
        let mut p = Probe::new(Mode::SkipWithError);
        p.begin(A);
        p.eval(RETAINED);
        p.eval("(async () => { await retained; marker++; })();");
        p.end();
        p.begin(B);
        p.eval("release(7)");
        p.drain();
    }

    #[test]
    fn skipped_callback_still_calls_source_species_resolver() {
        let mut p = Probe::new(Mode::SkipWithUndefined);
        p.begin(A);
        p.eval(RETAINED);
        p.eval("globalThis.resolvedBySpecies = 0; function Species(executor) { executor(() => resolvedBySpecies++, () => resolvedBySpecies++); } retained.constructor = { [Symbol.species]: Species }; globalThis.child = retained.then(() => marker++);");
        p.end();
        p.begin(B);
        p.eval("release(7)");
        p.drain();
        p.yes("marker === 0 && resolvedBySpecies === 1");
        assert_eq!(&*p.hooks.calls.borrow(), &[(A, B, true)]);
    }

    #[test]
    fn thenable_resolver_hook_tags_assimilation_at_resolution_not_resolver_creation() {
        let mut p = Probe::new(Mode::Observe);
        p.begin(A);
        p.eval(RETAINED);
        p.eval("globalThis.thenable = { then(resolve) { marker++; resolve(7); } };");
        assert!(p.hooks.made.borrow().is_empty());
        p.end();
        p.begin(B);
        p.eval("release(thenable)");
        assert_eq!(&*p.hooks.made.borrow(), &[B]);
        p.drain();
        p.yes("marker === 1");
        assert_eq!(&*p.hooks.calls.borrow(), &[(B, B, false)]);
    }
}
