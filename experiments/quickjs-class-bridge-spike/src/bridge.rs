//! The entire experiment FFI boundary. No private IDs, structs or raw ownership.
use rquickjs::{Array, Ctx, Object, Value, qjs};

/// Only engine-class eligibility. Even OrdinaryObject/Array still need RFC §6
/// prototype, descriptor, key, brand/host-identity, shape and bound validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObjectClass {
    NonObject,
    OrdinaryObject,
    Array,
    OtherObject,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ForeignContext;

/// Capture once while pristine, inside Context::with. No candidate is retained.
/// The Ctx handle keeps metadata tied to its live scope; classify rejects values
/// bearing a different Ctx even when rquickjs gives both contexts the same 'js.
/// Neither raw values/pointers nor class IDs are exposed to callers.
pub struct ClassBridge<'js> {
    ctx: Ctx<'js>,
    ordinary: qjs::JSClassID,
    array: qjs::JSClassID,
}

// This one function is the only local exception to deny(unsafe_code).
#[allow(unsafe_code)]
fn class_id(value: &Value<'_>) -> qjs::JSClassID {
    // SAFETY: Value::as_raw is the supported public representation accessor.
    // The borrowed, rooted Value and its Ctx remain live throughout this call,
    // under rquickjs's context/runtime access discipline (no parallel feature).
    // Public JS_GetClassID borrows JSValueConst, accepts every value tag, takes
    // no context argument, allocates nothing, invokes no JS, and returns a scalar.
    // It neither transfers ownership nor retains the value. No pointer is read
    // by Rust and no private layout/class constant is used.
    unsafe { qjs::JS_GetClassID(value.as_raw()) }
}

impl<'js> ClassBridge<'js> {
    pub fn new(ctx: &Ctx<'js>) -> rquickjs::Result<Self> {
        // Public native allocation, not source-global Object/Array constructors.
        let object = Object::new(ctx.clone())?;
        let array = Array::new(ctx.clone())?;
        let ordinary = class_id(object.as_value());
        let array = class_id(array.as_value());
        // Fail closed on broken setup/API assumptions; never persist numeric IDs.
        assert_ne!(ordinary, qjs::JS_INVALID_CLASS_ID);
        assert_ne!(array, qjs::JS_INVALID_CLASS_ID);
        assert_ne!(ordinary, array);
        Ok(Self {
            ctx: ctx.clone(),
            ordinary,
            array,
        })
    }

    pub fn classify(&self, value: &Value<'_>) -> Result<ObjectClass, ForeignContext> {
        // Pointer identity comparison through supported accessors only. This is
        // not dereferencing a pointer or claiming Value's Ctx proves provenance
        // of an object that a host deliberately imports from another realm.
        if self.ctx.as_raw() != value.ctx().as_raw() {
            return Err(ForeignContext);
        }
        let id = class_id(value);
        Ok(if id == qjs::JS_INVALID_CLASS_ID {
            ObjectClass::NonObject
        } else if id == self.ordinary {
            ObjectClass::OrdinaryObject
        } else if id == self.array {
            ObjectClass::Array
        } else {
            ObjectClass::OtherObject
        })
    }
}
