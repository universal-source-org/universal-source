use super::*;
use crate::{LoadLimits, load};
use serde_json::json;
use std::{
    path::Path,
    sync::{Arc, mpsc},
    thread,
};

fn instance(limits: CallLimits) -> Instance {
    Instance::new(
        load(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../examples/json/minimal"),
            LoadLimits::default(),
        )
        .unwrap(),
        limits,
    )
    .unwrap()
}
fn call(instance: &Instance, token: &Cancellation) -> Value {
    instance
        .invoke("home", &json!({}), &EffectiveGrants::default(), token)
        .unwrap()
}
fn code(value: &Value, expected: &str) {
    assert_eq!(value["ok"], false);
    assert_eq!(value["error"]["code"], expected);
    assert_eq!(value.as_object().unwrap().len(), 2);
    assert_eq!(value["error"].as_object().unwrap().len(), 2);
    assert!(!value["error"]["message"].as_str().unwrap().is_empty());
}
fn receive<T>(rx: &mpsc::Receiver<T>) -> T {
    rx.recv_timeout(Duration::from_secs(5))
        .expect("test synchronization watchdog")
}
fn outstanding(instance: &Instance, n: usize) {
    let until = Instant::now() + Duration::from_secs(5);
    let mut state = instance.state.lock().unwrap();
    while state.outstanding != n {
        let remaining = until.saturating_duration_since(Instant::now());
        assert!(!remaining.is_zero(), "outstanding count watchdog");
        state = instance.changed.wait_timeout(state, remaining).unwrap().0;
    }
}
fn freeze(instance: &Instance) -> Instant {
    let now = Instant::now();
    *instance.test_now.lock().unwrap() = Some(now);
    now
}
fn advance(instance: &Instance, now: Instant) {
    *instance.test_now.lock().unwrap() = Some(now);
    instance.changed.notify_all();
}

#[test]
fn ready_calls_disposal_is_permanent_and_new_load_works() {
    let source = instance(CallLimits::default());
    assert_eq!(call(&source, &Cancellation::default())["ok"], true);
    source.dispose();
    source.dispose();
    assert!(source.is_disposed());
    let outcome = source.invoke_inner(
        "home",
        &json!({}),
        &EffectiveGrants::default(),
        &Cancellation::default(),
        Instant::now() + Duration::from_secs(5),
        &|_| panic!("must not execute"),
    );
    assert_eq!(outcome, Err(LifecycleError::Disposed));
    assert_eq!(
        call(&instance(CallLimits::default()), &Cancellation::default())["ok"],
        true
    );
}

#[test]
fn same_instance_serializes_execution_explicitly() {
    let source = instance(CallLimits::default());
    let now = freeze(&source);
    let executing = Mutex::new(0);
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let release_rx = Mutex::new(release_rx);
    thread::scope(|s| {
        let first = s.spawn(|| {
            source.invoke_inner(
                "home",
                &json!({}),
                &EffectiveGrants::default(),
                &Cancellation::default(),
                now + Duration::from_secs(5),
                &|stage| {
                    if stage == Stage::Entered {
                        assert_eq!(*executing.lock().unwrap(), 0);
                        *executing.lock().unwrap() = 1;
                        entered_tx.send(()).unwrap();
                        receive(&release_rx.lock().unwrap());
                    } else {
                        *executing.lock().unwrap() = 0;
                    }
                },
            )
        });
        receive(&entered_rx);
        let second = s.spawn(|| {
            source.invoke_inner(
                "home",
                &json!({}),
                &EffectiveGrants::default(),
                &Cancellation::default(),
                now + Duration::from_secs(5),
                &|stage| {
                    if stage == Stage::Entered {
                        assert_eq!(*executing.lock().unwrap(), 0, "overlapping execution");
                    }
                },
            )
        });
        outstanding(&source, 2);
        release_tx.send(()).unwrap();
        assert_eq!(first.join().unwrap().unwrap()["ok"], true);
        assert_eq!(second.join().unwrap().unwrap()["ok"], true);
    });
}

#[test]
fn independent_instances_overlap_and_do_not_share_lifecycle_or_results() {
    let a = instance(CallLimits::default());
    let b = instance(CallLimits::default());
    let now = freeze(&a);
    let token = Cancellation::default();
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let release_rx = Mutex::new(release_rx);
    thread::scope(|s| {
        let first = s.spawn(|| {
            a.invoke_inner(
                "home",
                &json!({}),
                &EffectiveGrants::default(),
                &token,
                now + Duration::from_secs(5),
                &|stage| {
                    if stage == Stage::Entered {
                        entered_tx.send(()).unwrap();
                        receive(&release_rx.lock().unwrap());
                    }
                },
            )
        });
        receive(&entered_rx);
        let mut value = call(&b, &Cancellation::default());
        assert_eq!(value["ok"], true); // Completes while A holds its execution slot.
        value["data"]["items"][0]["title"] = json!("mutated copy");
        token.cancel();
        release_tx.send(()).unwrap();
        code(&first.join().unwrap().unwrap(), "CANCELLED");
    });
    a.dispose();
    assert_eq!(
        call(&b, &Cancellation::default())["data"]["items"][0]["title"],
        "Demo video"
    );
    assert!(!b.is_disposed());
}

#[test]
fn cancellation_wins_before_publish_and_discards_late_success() {
    let source = instance(CallLimits::default());
    let now = freeze(&source);
    let token = Cancellation::default();
    let (ready_tx, ready_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let release_rx = Mutex::new(release_rx);
    let (out_tx, out_rx) = mpsc::channel();
    thread::scope(|s| {
        s.spawn(|| {
            let result = source
                .invoke_inner(
                    "home",
                    &json!({}),
                    &EffectiveGrants::default(),
                    &token,
                    now + Duration::from_secs(5),
                    &|stage| {
                        if stage == Stage::BeforePublish {
                            ready_tx.send(()).unwrap();
                            receive(&release_rx.lock().unwrap());
                        }
                    },
                )
                .unwrap();
            out_tx.send(result).unwrap();
        });
        receive(&ready_rx);
        token.cancel();
        release_tx.send(()).unwrap();
        code(&receive(&out_rx), "CANCELLED");
    });
    assert!(out_rx.try_recv().is_err()); // One delivery, no detached completion.
    assert_eq!(call(&source, &Cancellation::default())["ok"], true);
    assert!(!source.is_disposed());
}

#[test]
fn completion_wins_before_later_cancellation() {
    let source = instance(CallLimits::default());
    let token = Cancellation::default();
    let result = call(&source, &token);
    token.cancel();
    assert_eq!(result["ok"], true);
    code(&call(&source, &token), "CANCELLED"); // Token is sticky; result is immutable.
    assert_eq!(call(&source, &Cancellation::default())["ok"], true);
}

#[test]
fn timeout_discards_completed_result_and_keeps_context_safe() {
    let source = instance(CallLimits::default());
    let now = freeze(&source);
    let result = source
        .invoke_inner(
            "home",
            &json!({}),
            &EffectiveGrants::default(),
            &Cancellation::default(),
            now + Duration::from_secs(1),
            &|stage| {
                if stage == Stage::BeforePublish {
                    advance(&source, now + Duration::from_secs(1));
                }
            },
        )
        .unwrap();
    code(&result, "TIMEOUT");
    assert!(!source.is_disposed());
    assert_eq!(call(&source, &Cancellation::default())["ok"], true);
}

#[test]
fn queued_cancellation_and_timeout_never_execute() {
    for cancel in [true, false] {
        let source = instance(CallLimits::default());
        let now = freeze(&source);
        let token = Cancellation::default();
        let (entered_tx, entered_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let release_rx = Mutex::new(release_rx);
        thread::scope(|s| {
            let first = s.spawn(|| {
                source.invoke_inner(
                    "home",
                    &json!({}),
                    &EffectiveGrants::default(),
                    &Cancellation::default(),
                    now + Duration::from_secs(50),
                    &|stage| {
                        if stage == Stage::Entered {
                            entered_tx.send(()).unwrap();
                            receive(&release_rx.lock().unwrap());
                        }
                    },
                )
            });
            receive(&entered_rx);
            let waiting = s.spawn(|| {
                source.invoke_inner(
                    "home",
                    &json!({}),
                    &EffectiveGrants::default(),
                    &token,
                    now + Duration::from_secs(1),
                    &|_| panic!("queued call must not execute"),
                )
            });
            outstanding(&source, 2);
            if cancel {
                token.cancel();
            } else {
                advance(&source, now + Duration::from_secs(1));
            }
            code(
                &waiting.join().unwrap().unwrap(),
                if cancel { "CANCELLED" } else { "TIMEOUT" },
            );
            release_tx.send(()).unwrap();
            assert_eq!(first.join().unwrap().unwrap()["ok"], true);
        });
    }
}

#[test]
fn actual_clock_enforces_finite_queue_deadline() {
    let source = instance(CallLimits {
        timeout: Duration::from_millis(20),
        ..CallLimits::default()
    });
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let release_rx = Mutex::new(release_rx);
    thread::scope(|s| {
        let first = s.spawn(|| {
            source.invoke_inner(
                "home",
                &json!({}),
                &EffectiveGrants::default(),
                &Cancellation::default(),
                Instant::now() + Duration::from_secs(5),
                &|stage| {
                    if stage == Stage::Entered {
                        entered_tx.send(()).unwrap();
                        receive(&release_rx.lock().unwrap());
                    }
                },
            )
        });
        receive(&entered_rx);
        code(&call(&source, &Cancellation::default()), "TIMEOUT");
        release_tx.send(()).unwrap();
        assert_eq!(first.join().unwrap().unwrap()["ok"], true);
    });
}

#[test]
fn disposal_cancels_running_and_queued_calls_and_waits_for_quiescence() {
    let source = instance(CallLimits::default());
    let now = freeze(&source);
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let release_rx = Mutex::new(release_rx);
    let (disposed_tx, disposed_rx) = mpsc::channel();
    thread::scope(|s| {
        let first = s.spawn(|| {
            source.invoke_inner(
                "home",
                &json!({}),
                &EffectiveGrants::default(),
                &Cancellation::default(),
                now + Duration::from_secs(5),
                &|stage| {
                    if stage == Stage::BeforePublish {
                        entered_tx.send(()).unwrap();
                        receive(&release_rx.lock().unwrap());
                    }
                },
            )
        });
        receive(&entered_rx);
        let waiting = s.spawn(|| call(&source, &Cancellation::default()));
        outstanding(&source, 2);
        s.spawn(|| {
            source.dispose();
            disposed_tx.send(()).unwrap();
        });
        code(&waiting.join().unwrap(), "CANCELLED");
        assert!(source.is_disposed());
        assert!(disposed_rx.try_recv().is_err()); // Active computation has not quiesced.
        release_tx.send(()).unwrap();
        code(&first.join().unwrap().unwrap(), "CANCELLED");
        receive(&disposed_rx);
    });
    assert_eq!(source.state.lock().unwrap().outstanding, 0);
    assert_eq!(
        source.invoke(
            "home",
            &json!({}),
            &EffectiveGrants::default(),
            &Cancellation::default()
        ),
        Err(LifecycleError::Disposed)
    );
}

#[test]
fn panicking_execution_disposes_unsafe_context_and_cancels_waiters() {
    let source = instance(CallLimits::default());
    let now = freeze(&source);
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let release_rx = Mutex::new(release_rx);
    thread::scope(|s| {
        let first = s.spawn(|| {
            source.invoke_inner(
                "home",
                &json!({}),
                &EffectiveGrants::default(),
                &Cancellation::default(),
                now + Duration::from_secs(5),
                &|stage| {
                    if stage == Stage::Entered {
                        entered_tx.send(()).unwrap();
                        receive(&release_rx.lock().unwrap());
                        panic!("injected internal failure");
                    }
                },
            )
        });
        receive(&entered_rx);
        let waiting = s.spawn(|| call(&source, &Cancellation::default()));
        outstanding(&source, 2);
        release_tx.send(()).unwrap();
        code(&first.join().unwrap().unwrap(), "SOURCE_ERROR");
        code(&waiting.join().unwrap(), "CANCELLED");
    });
    assert!(source.is_disposed());
}

#[test]
fn byte_limits_inclusive_depth_bounded_and_context_reusable() {
    let full = instance(CallLimits::default());
    let size = serde_json::to_vec(&call(&full, &Cancellation::default()))
        .unwrap()
        .len();
    for (limit, succeeds) in [(size, true), (size - 1, false)] {
        let source = instance(CallLimits {
            result_bytes: limit,
            ..CallLimits::default()
        });
        let result = call(&source, &Cancellation::default());
        if succeeds {
            assert_eq!(result["ok"], true);
        } else {
            code(&result, "RESOURCE_LIMIT");
        }
        assert!(!source.is_disposed());
        assert_eq!(
            source
                .invoke(
                    "search",
                    &json!({"query":"absent"}),
                    &EffectiveGrants::default(),
                    &Cancellation::default()
                )
                .unwrap()["ok"],
            true
        );
    }
    let source = instance(CallLimits {
        input_bytes: 2,
        ..CallLimits::default()
    });
    assert_eq!(call(&source, &Cancellation::default())["ok"], true); // {} is exactly two bytes.
    code(
        &source
            .invoke(
                "home",
                &json!({"extra":1}),
                &EffectiveGrants::default(),
                &Cancellation::default(),
            )
            .unwrap(),
        "RESOURCE_LIMIT",
    );
    code(
        &source
            .invoke(
                "unknown",
                &json!({"extra":1}),
                &EffectiveGrants::default(),
                &Cancellation::default(),
            )
            .unwrap(),
        "UNSUPPORTED_OPERATION",
    );
    let mut deep = Value::Null;
    for _ in 0..66 {
        deep = json!([deep]);
    }
    code(
        &full
            .invoke(
                "home",
                &deep,
                &EffectiveGrants::default(),
                &Cancellation::default(),
            )
            .unwrap(),
        "RESOURCE_LIMIT",
    );
}

#[test]
fn outstanding_call_budget_is_enforced_without_poisoning_instance() {
    let source = Arc::new(instance(CallLimits {
        outstanding_calls: 1,
        ..CallLimits::default()
    }));
    let now = freeze(&source);
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let release_rx = Mutex::new(release_rx);
    thread::scope(|s| {
        let first = s.spawn(|| {
            source.invoke_inner(
                "home",
                &json!({}),
                &EffectiveGrants::default(),
                &Cancellation::default(),
                now + Duration::from_secs(5),
                &|stage| {
                    if stage == Stage::Entered {
                        entered_tx.send(()).unwrap();
                        receive(&release_rx.lock().unwrap());
                    }
                },
            )
        });
        receive(&entered_rx);
        code(&call(&source, &Cancellation::default()), "RESOURCE_LIMIT");
        release_tx.send(()).unwrap();
        assert_eq!(first.join().unwrap().unwrap()["ok"], true);
    });
    assert_eq!(call(&source, &Cancellation::default())["ok"], true);
}

#[test]
fn limits_are_host_setup_errors_and_race_priority_is_local() {
    for limits in [
        CallLimits {
            timeout: Duration::ZERO,
            ..CallLimits::default()
        },
        CallLimits {
            timeout: Duration::from_secs(61),
            ..CallLimits::default()
        },
        CallLimits {
            input_bytes: 0,
            ..CallLimits::default()
        },
        CallLimits {
            result_bytes: usize::MAX,
            ..CallLimits::default()
        },
        CallLimits {
            outstanding_calls: 65,
            ..CallLimits::default()
        },
    ] {
        let source = load(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../examples/json/minimal"),
            LoadLimits::default(),
        )
        .unwrap();
        assert!(matches!(
            Instance::new(source, limits),
            Err(LifecycleError::InvalidLimits)
        ));
    }
    let source = instance(CallLimits::default());
    let now = freeze(&source);
    let token = Cancellation::default();
    token.cancel();
    code(
        &source
            .invoke_inner(
                "home",
                &json!({}),
                &EffectiveGrants::default(),
                &token,
                now,
                &|_| panic!("must not execute"),
            )
            .unwrap(),
        "CANCELLED",
    );
}

#[test]
fn cancellation_checkpoint_prevents_dispatch_and_suppresses_late_errors() {
    let source = instance(CallLimits::default());
    let now = freeze(&source);
    for stage_to_cancel in [Stage::Entered, Stage::BeforePublish] {
        let token = Cancellation::default();
        let result = source
            .invoke_inner(
                "detail",
                &json!({"id":"missing"}),
                &EffectiveGrants::default(),
                &token,
                now + Duration::from_secs(1),
                &|stage| {
                    if stage == stage_to_cancel {
                        token.cancel();
                    }
                    if stage_to_cancel == Stage::Entered {
                        assert!(stage != Stage::BeforePublish);
                    }
                },
            )
            .unwrap();
        code(&result, "CANCELLED");
    }
}

#[test]
fn measurement_limits_strings_keys_nesting_and_escaped_json_bytes() {
    let tiny = instance(CallLimits {
        result_bytes: 1,
        ..CallLimits::default()
    });
    let fallback = call(&tiny, &Cancellation::default());
    code(&fallback, "RESOURCE_LIMIT");
    assert!(serde_json::to_vec(&fallback).unwrap().len() < 128);
    for name in ["CANCELLED", "TIMEOUT", "SOURCE_ERROR", "RESOURCE_LIMIT"] {
        assert!(serde_json::to_vec(&dispatch::error(name)).unwrap().len() < 128);
    }
    let check = || Ok(());
    let value = json!({"quote":"\"\n你好"});
    let bytes = serde_json::to_vec(&value).unwrap().len();
    assert!(measure(&value, bytes, &check).is_ok());
    assert_eq!(measure(&value, bytes - 1, &check), Err("RESOURCE_LIMIT"));
    assert_eq!(
        measure(&json!("x".repeat(100)), 10, &check),
        Err("RESOURCE_LIMIT")
    );
    assert_eq!(
        measure(&json!({"longkey":0}), 3, &check),
        Err("RESOURCE_LIMIT")
    );
    let mut value = Value::Null;
    for _ in 0..64 {
        value = json!([value]);
    }
    assert!(measure(&value, 1000, &check).is_ok());
    value = json!([value]);
    assert_eq!(measure(&value, 1000, &check), Err("RESOURCE_LIMIT"));
    assert_eq!(measure(&json!({}), 100, &|| Err("TIMEOUT")), Err("TIMEOUT"));
}
