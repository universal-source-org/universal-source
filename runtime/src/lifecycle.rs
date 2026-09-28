//! Synchronous, instance-local lifecycle. No worker threads or async executor.
use crate::{EffectiveGrants, LoadedSource, dispatch};
use serde_json::Value;
use std::{
    fmt,
    io::{self, Write},
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{Condvar, Mutex},
    time::{Duration, Instant},
};

/// Reference-host policy, not portable specification constants.
#[derive(Debug, Clone, Copy)]
pub struct CallLimits {
    /// Includes time waiting behind another call; nonzero, at most 60 seconds.
    pub timeout: Duration,
    /// Compact JSON bytes, inclusive; may only be lowered from the defaults.
    pub input_bytes: usize,
    pub result_bytes: usize,
    /// Active plus waiting calls; no runtime-owned thread or input queue.
    pub outstanding_calls: usize,
}
impl Default for CallLimits {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(5),
            input_bytes: 64 * 1024,
            result_bytes: 8 * 1024 * 1024 + 1024,
            outstanding_calls: 64,
        }
    }
}

/// Host setup/misuse, outside the Source API envelope (U1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleError {
    InvalidLimits,
    Disposed,
}
impl fmt::Display for LifecycleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidLimits => "invalid call limits",
            Self::Disposed => "instance is disposed",
        })
    }
}
impl std::error::Error for LifecycleError {}

/// Sticky caller cancellation request. Use a fresh token for each independent call.
/// Requesting cancellation does not synchronously interrupt a Rust computation.
#[derive(Default)]
pub struct Cancellation(Mutex<bool>);
impl Cancellation {
    pub fn cancel(&self) {
        *self.0.lock().unwrap() = true;
    }
}

#[derive(Default)]
struct State {
    disposed: bool,
    executing: bool,
    outstanding: usize,
}

/// Owns one validated snapshot and independent mutable scheduling state.
/// Share this instance with Arc to call it from multiple host threads.
pub struct Instance {
    source: LoadedSource,
    limits: CallLimits,
    state: Mutex<State>,
    changed: Condvar,
    #[cfg(test)]
    test_now: Mutex<Option<Instant>>,
}

impl Instance {
    pub fn new(source: LoadedSource, limits: CallLimits) -> Result<Self, LifecycleError> {
        let caps = CallLimits::default();
        if limits.timeout.is_zero()
            || limits.timeout > Duration::from_secs(60)
            || [
                (limits.input_bytes, caps.input_bytes),
                (limits.result_bytes, caps.result_bytes),
                (limits.outstanding_calls, caps.outstanding_calls),
            ]
            .iter()
            .any(|&(n, cap)| n == 0 || n > cap)
        {
            return Err(LifecycleError::InvalidLimits);
        }
        Ok(Self {
            source,
            limits,
            state: Mutex::new(State::default()),
            changed: Condvar::new(),
            #[cfg(test)]
            test_now: Mutex::new(None),
        })
    }

    fn now(&self) -> Instant {
        #[cfg(test)]
        if let Some(now) = *self.test_now.lock().unwrap() {
            return now;
        }
        Instant::now()
    }

    /// Stored-data inspection and grant setup only, never resource authorization.
    pub fn source(&self) -> &LoadedSource {
        &self.source
    }

    pub fn is_disposed(&self) -> bool {
        self.state.lock().unwrap().disposed
    }

    /// Permanent and idempotent. Returns only when admitted calls have quiesced.
    pub fn dispose(&self) {
        let mut state = self.state.lock().unwrap();
        state.disposed = true;
        self.changed.notify_all();
        while state.outstanding != 0 {
            state = self.changed.wait(state).unwrap();
        }
    }

    /// Exactly one synchronous outcome. Deadline starts on entry, including queue time.
    pub fn invoke(
        &self,
        operation: &str,
        input: &Value,
        grants: &EffectiveGrants,
        cancellation: &Cancellation,
    ) -> Result<Value, LifecycleError> {
        self.invoke_inner(
            operation,
            input,
            grants,
            cancellation,
            self.now() + self.limits.timeout,
            #[cfg(test)]
            &|_| {},
        )
    }

    fn stop(&self, cancellation: &Cancellation, deadline: Instant) -> Result<(), &'static str> {
        let state = self.state.lock().unwrap();
        let cancelled = cancellation.0.lock().unwrap();
        stopped(&state, *cancelled, deadline, self.now())
    }

    fn invoke_inner(
        &self,
        operation: &str,
        input: &Value,
        grants: &EffectiveGrants,
        cancellation: &Cancellation,
        deadline: Instant,
        #[cfg(test)] hook: &dyn Fn(Stage),
    ) -> Result<Value, LifecycleError> {
        let mut state = self.state.lock().unwrap();
        if state.disposed {
            return Err(LifecycleError::Disposed);
        }
        if state.outstanding == self.limits.outstanding_calls {
            return Ok(dispatch::error("RESOURCE_LIMIT"));
        }
        state.outstanding += 1;
        self.changed.notify_all();
        loop {
            let cancelled = cancellation.0.lock().unwrap();
            if let Err(code) = stopped(&state, *cancelled, deadline, self.now()) {
                state.outstanding -= 1;
                self.changed.notify_all();
                return Ok(dispatch::error(code));
            }
            drop(cancelled);
            if !state.executing {
                break;
            }
            // Token polling only while queued. Execution has cooperative checkpoints.
            let delay = deadline
                .saturating_duration_since(self.now())
                .min(Duration::from_millis(10));
            state = self.changed.wait_timeout(state, delay).unwrap().0;
        }
        state.executing = true;
        drop(state);

        // No source code runs: only bounded static JSON work. Panics unwind out
        // of execution without poisoning scheduling locks; quarantine this instance.
        let result = catch_unwind(AssertUnwindSafe(|| {
            #[cfg(test)]
            hook(Stage::Entered);
            let check = || self.stop(cancellation, deadline);
            check()?;
            let result = if !dispatch::declared(&self.source, operation) {
                dispatch::error("UNSUPPORTED_OPERATION")
            } else {
                measure(input, self.limits.input_bytes, &check)?;
                self.source.invoke_checked(operation, input, grants, &check)
            };
            check()?;
            measure(&result, self.limits.result_bytes, &check)?;
            #[cfg(test)]
            hook(Stage::BeforePublish);
            Ok::<_, &'static str>(result)
        }));

        // Single terminal decision, synchronized with cancel requests/disposal.
        // A late result is dropped here; no background work survives this return.
        let mut state = self.state.lock().unwrap();
        let cancelled = cancellation.0.lock().unwrap();
        let unsafe_context = result.is_err();
        let stop = stopped(&state, *cancelled, deadline, self.now());
        if unsafe_context {
            state.disposed = true;
        }
        let outcome = match stop {
            Err(code) => {
                // Release discarded values before reporting quiescence to dispose.
                drop(result);
                dispatch::error(code)
            }
            Ok(()) => match result {
                Ok(Ok(value)) => value,
                Ok(Err(code)) => dispatch::error(code),
                Err(_) => dispatch::error("SOURCE_ERROR"),
            },
        };
        state.executing = false;
        state.outstanding -= 1;
        self.changed.notify_all();
        Ok(outcome)
    }
}
impl Drop for Instance {
    fn drop(&mut self) {
        self.dispose();
    }
}

fn stopped(
    state: &State,
    cancelled: bool,
    deadline: Instant,
    now: Instant,
) -> Result<(), &'static str> {
    if state.disposed || cancelled {
        Err("CANCELLED")
    } else if now >= deadline {
        Err("TIMEOUT")
    } else {
        Ok(())
    }
}

// Count compact JSON without allocating an encoded copy. Traversal is bounded
// before serde's recursive serializer touches caller-controlled input.
fn measure(
    value: &Value,
    limit: usize,
    check: &impl Fn() -> Result<(), &'static str>,
) -> Result<(), &'static str> {
    fn depth(
        value: &Value,
        level: usize,
        limit: usize,
        check: &impl Fn() -> Result<(), &'static str>,
    ) -> Result<(), &'static str> {
        check()?;
        if (level >= 64 && (value.is_array() || value.is_object()))
            || value.as_str().is_some_and(|s| s.len() > limit)
        {
            return Err("RESOURCE_LIMIT");
        }
        match value {
            Value::Array(a) => {
                for v in a {
                    depth(v, level + 1, limit, check)?;
                }
            }
            Value::Object(o) => {
                for (key, v) in o {
                    if key.len() > limit {
                        return Err("RESOURCE_LIMIT");
                    }
                    depth(v, level + 1, limit, check)?;
                }
            }
            _ => (),
        }
        Ok(())
    }
    depth(value, 0, limit, check)?;
    struct Counter<'a, F> {
        remaining: usize,
        check: &'a F,
        failure: Option<&'static str>,
    }
    impl<F: Fn() -> Result<(), &'static str>> Write for Counter<'_, F> {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            if let Err(code) = (self.check)() {
                self.failure = Some(code);
            } else if bytes.len() > self.remaining {
                self.failure = Some("RESOURCE_LIMIT");
            }
            if self.failure.is_some() {
                return Err(io::Error::other("call limit"));
            }
            self.remaining -= bytes.len();
            Ok(bytes.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let mut counter = Counter {
        remaining: limit,
        check,
        failure: None,
    };
    serde_json::to_writer(&mut counter, value)
        .map_err(|_| counter.failure.unwrap_or("RESOURCE_LIMIT"))
}

#[cfg(test)]
#[derive(Clone, Copy, PartialEq, Eq)]
enum Stage {
    Entered,
    BeforePublish,
}
#[cfg(test)]
#[path = "lifecycle_tests.rs"]
mod tests;
