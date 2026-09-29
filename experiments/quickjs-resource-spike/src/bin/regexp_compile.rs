//! Run ONLY under the README's external watchdog. No engine source is modified.
#![forbid(unsafe_code)]

use rquickjs::{Function, Value};
use std::{io::Write, rc::Rc, time::Instant};
use universal_source_quickjs_resource_spike::{Control, Reason, realm};

fn main() {
    let references: usize = std::env::args().nth(1).expect("count").parse().unwrap();
    assert!((1..=120_000).contains(&references));
    let pattern_bytes = references * 5 + 7;
    let (rt, context) = realm();
    let state = Rc::new(Control::default());
    let count = state.clone();
    rt.set_interrupt_handler(Some(Box::new(move || {
        if !count.armed.get() {
            return false;
        }
        count.callbacks.set(count.callbacks.get() + 1);
        if count.observed.get().is_none() {
            count.observed.set(Some(Reason::Timeout));
        }
        eprintln!("INTERRUPT_CALLBACK");
        true
    })));
    context.with(|ctx| {
        let compile: Function = ctx.eval(include_str!("../regexp_compile.js")).unwrap();
        eprintln!(
            "ENTER regexp compile: references={references}, bytes={}",
            pattern_bytes
        );
        std::io::stderr().flush().unwrap();
        let start = Instant::now();
        state.armed.set(true);
        let result = compile.call::<_, Value>((references as u32,));
        state.armed.set(false);
        eprintln!(
            "RETURN elapsed_ms={} callback_count={} result_ok={}",
            start.elapsed().as_millis(),
            state.callbacks.get(),
            result.is_ok()
        );
        assert_eq!(
            state.callbacks.get(),
            0,
            "engine called interrupt during compile"
        );
        assert!(result.is_ok(), "compile unexpectedly failed");
    });
    eprintln!("ENGINE_ACCOUNTED_BYTES {}", rt.memory_usage().malloc_size);
    drop(context);
    drop(rt);
}
