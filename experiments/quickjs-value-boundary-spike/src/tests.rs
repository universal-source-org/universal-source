use rquickjs::{Context, Ctx, Function, Object, Runtime, Type, Value, function::This};
use std::{cell::Cell, rc::Rc};

// Same bytes as the prior experiments. No alternate guard implementation.
const DYNAMIC: &str = include_str!("../../quickjs-dynamic-code-spike/src/harden.js");
const GLOBAL: &str = include_str!("../../quickjs-global-surface-spike/src/harden.js");
const INVENTORY: &str = include_str!("../../quickjs-global-surface-spike/src/inventory.js");
const PRISTINE: &str =
    include_str!("../../quickjs-global-surface-spike/src/pristine-inventory.json");
const GLOBALS: &str = include_str!("../../quickjs-global-surface-spike/src/allowed-globals.json");
const INTRINSICS: &str =
    include_str!("../../quickjs-global-surface-spike/src/allowed-intrinsics.json");

fn runtime() -> Runtime {
    let rt = Runtime::new().unwrap();
    rt.set_memory_limit(16 * 1024 * 1024);
    rt.set_max_stack_size(256 * 1024);
    rt
}

fn harden(ctx: &Ctx<'_>) {
    let actual: String = ctx.eval(INVENTORY).unwrap();
    let expected: String = ctx.eval(format!("JSON.stringify({PRISTINE})")).unwrap();
    assert_eq!(actual, expected);
    ctx.eval::<(), _>(DYNAMIC).unwrap();
    let harden: Function = ctx.eval(GLOBAL).unwrap();
    let globals: Value = ctx.eval(GLOBALS).unwrap();
    let intrinsics: Value = ctx.eval(format!("({INTRINSICS})")).unwrap();
    harden.call::<_, ()>((globals, intrinsics)).unwrap();
}

#[derive(Debug, PartialEq)]
enum Gate {
    RejectKnownNonRecord,
    // An embedding limitation, NOT RFC INVALID_RESULT. A valid record also lands here.
    BlockedOrdinaryClassProof,
}

fn record_gate(value: &Value<'_>) -> Gate {
    // Only direct, non-executing engine predicates. Never get a property/prototype,
    // enumerate, coerce, stringify or call a reflection helper on this unknown value.
    if !value.is_object()
        || value.is_proxy()
        || value.is_array()
        || value.is_function()
        || value.is_promise()
        || value.is_error()
    {
        Gate::RejectKnownNonRecord
    } else {
        Gate::BlockedOrdinaryClassProof
    }
}

fn signature(value: &Value<'_>) -> (Type, bool, bool, bool, bool, bool, bool) {
    (
        value.type_of(),
        value.is_object(),
        value.is_proxy(),
        value.is_array(),
        value.is_function(),
        value.is_promise(),
        value.is_error(),
    )
}

#[test]
fn safe_predicates_cannot_admit_record_and_exclude_prototype_mutated_map() {
    let rt = runtime();
    let c = Context::full(&rt).unwrap();
    c.with(|ctx| {
        // Capture original trusted metadata while the realm is pristine.
        let original = Object::new(ctx.clone()).unwrap().get_prototype().unwrap();
        harden(&ctx);
        for prototype in ["Object.prototype", "null"] {
            let make: Function = ctx
                .eval(format!(
                    r#"() => {{
                const record = Object.setPrototypeOf({{}}, {prototype});
                const native = Object.setPrototypeOf(new Map(), {prototype});
                return [record, native];
            }}"#
                ))
                .unwrap();
            // Fixed fixture pair, not enumeration of an unclassified source candidate.
            let pair: rquickjs::Array = make.call(()).unwrap();
            let record: Value = pair.get(0).unwrap();
            let native: Value = pair.get(1).unwrap();
            assert_eq!(signature(&record), signature(&native));
            assert_eq!(record.type_of(), Type::Object);
            assert_eq!(record_gate(&record), Gate::BlockedOrdinaryClassProof);
            assert_eq!(record_gate(&native), Gate::BlockedOrdinaryClassProof);
            // NEGATIVE CONTROL: these KNOWN fixture objects show prototype-only admission
            // wrongly accepts Map. This operation is not part of record_gate.
            let record_proto = record.as_object().unwrap().get_prototype();
            let native_proto = native.as_object().unwrap().get_prototype();
            assert_eq!(record_proto, native_proto);
            assert!(native_proto.is_none() || native_proto == Some(original.clone()));
            assert!(record.as_object().unwrap().is_empty());
            assert!(native.as_object().unwrap().is_empty());
        }
    });
    assert!(!rt.is_job_pending());
}

#[test]
fn class_created_record_is_valid_but_cannot_be_rejected_as_a_brand_workaround() {
    let rt = runtime();
    let c = Context::full(&rt).unwrap();
    c.with(|ctx| {
        harden(&ctx);
        let value: Value = ctx
            .eval(
                r#"(() => {
            class Record { #secret=17; constructor(){this.value=42;} }
            return Object.setPrototypeOf(new Record(), Object.prototype);
        })()"#,
            )
            .unwrap();
        assert_eq!(record_gate(&value), Gate::BlockedOrdinaryClassProof);
        // Known authored object: inspect only to establish the counterexample's shape.
        // RFC permits its own data field, not its private field or class behavior.
        let object = value.as_object().unwrap();
        assert_eq!(
            object
                .keys::<String>()
                .collect::<rquickjs::Result<Vec<_>>>()
                .unwrap(),
            ["value"]
        );
        assert_eq!(object.get::<_, i32>("value").unwrap(), 42);
    });
}

#[test]
fn safe_object_get_and_property_iterator_execute_getters_negative_control() {
    let rt = runtime();
    let c = Context::full(&rt).unwrap();
    c.with(|ctx| {
        harden(&ctx);
        let count = Rc::new(Cell::new(0));
        let observed = count.clone();
        let marker = Function::new(ctx.clone(), move || observed.set(observed.get() + 1)).unwrap();
        let make: Function = ctx
            .eval("mark => ({get value(){mark(); return 42;}})")
            .unwrap();
        let candidate: Object = make.call((marker,)).unwrap();
        assert!(!candidate.is_proxy());
        assert_eq!(
            record_gate(candidate.as_value()),
            Gate::BlockedOrdinaryClassProof
        );
        assert_eq!(count.get(), 0);
        // Deliberately naive fallback: safe Rust API is not a source-quiescence guarantee.
        assert_eq!(candidate.get::<_, i32>("value").unwrap(), 42);
        assert_eq!(count.get(), 1);
        let fields = candidate
            .props::<String, i32>()
            .collect::<rquickjs::Result<Vec<_>>>()
            .unwrap();
        assert_eq!(fields, [("value".into(), 42)]);
        assert_eq!(count.get(), 2);
    });
}

#[test]
fn saved_native_brand_stringifier_runs_source_tag_getter_negative_control() {
    let rt = runtime();
    let c = Context::full(&rt).unwrap();
    c.with(|ctx| {
        let stringify: Function = ctx.eval("Object.prototype.toString").unwrap();
        harden(&ctx);
        let count = Rc::new(Cell::new(0));
        let observed = count.clone();
        let marker = Function::new(ctx.clone(), move || observed.set(observed.get() + 1)).unwrap();
        let make: Function = ctx
            .eval("mark => ({get [Symbol.toStringTag](){mark(); return 'Object';}})")
            .unwrap();
        let candidate: Value = make.call((marker,)).unwrap();
        assert_eq!(record_gate(&candidate), Gate::BlockedOrdinaryClassProof);
        assert_eq!(count.get(), 0);
        // Capturing a pristine native function does not make its algorithm non-executing.
        let tag: String = stringify.call((This(candidate),)).unwrap();
        assert_eq!(tag, "[object Object]");
        assert_eq!(count.get(), 1);
    });
}

#[test]
fn proxy_gate_runs_no_traps_but_enumeration_does_negative_control() {
    let rt = runtime();
    let c = Context::full(&rt).unwrap();
    c.with(|ctx| {
        // Test-private native constructor handle; never placed back on the global.
        let proxy: rquickjs::function::Constructor = ctx.globals().get("Proxy").unwrap();
        harden(&ctx);
        let count = Rc::new(Cell::new(0));
        let observed = count.clone();
        let marker = Function::new(ctx.clone(), move || observed.set(observed.get()+1)).unwrap();
        let make_handler: Function = ctx.eval("mark => ({ownKeys(){mark();return [];},get(){mark();},getPrototypeOf(){mark();return null;}})").unwrap();
        let handler: Value = make_handler.call((marker,)).unwrap();
        let target = Object::new(ctx.clone()).unwrap();
        let candidate: Object = proxy.construct((target,handler)).unwrap();
        assert_eq!(record_gate(candidate.as_value()), Gate::RejectKnownNonRecord);
        assert_eq!(count.get(), 0);
        // Only this explicitly naive path executes the source ownKeys trap.
        assert!(candidate.keys::<String>().collect::<rquickjs::Result<Vec<_>>>().unwrap().is_empty());
        assert_eq!(count.get(), 1);
        assert!(ctx.eval::<bool,_>("typeof Proxy === 'undefined'").unwrap());
    });
}

#[test]
fn mutated_helpers_do_not_change_direct_gate_or_recover_compilation() {
    let rt = runtime();
    let c = Context::full(&rt).unwrap();
    c.with(|ctx| {
        harden(&ctx);
        let record: Value = ctx.eval("({value:42})").unwrap();
        ctx.eval::<(),_>(r#"
            const poisoned=()=>{throw new Error('reflection called');};
            Object.getPrototypeOf=poisoned; Object.getOwnPropertyDescriptor=poisoned;
            Object.getOwnPropertyDescriptors=poisoned; Object.keys=poisoned;
            Reflect.ownKeys=poisoned; Object.prototype.toString=poisoned;
            globalThis.Object=poisoned; globalThis.Reflect=poisoned;
        "#).unwrap();
        assert_eq!(record_gate(&record), Gate::BlockedOrdinaryClassProof);
        // Explicit regression assertion; ordinary source evaluation occurs only here,
        // never inside record_gate. Prior full guard suites are rerun unchanged as well.
        assert!(ctx.eval::<bool,_>("(() => {try{eval('1');return false;}catch(e){return e instanceof EvalError && !('stack' in e);}})()").unwrap());
    });
}
