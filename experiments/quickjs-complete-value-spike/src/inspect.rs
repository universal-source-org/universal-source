//! Safe structural inspection after the unchanged class gate. No public raw handles.
use crate::Failure;
use class_bridge::{ClassBridge, ObjectClass};
use rquickjs::{Atom, Ctx, Object, Value, object::Filter, qjs};

pub(crate) struct Ordinary<'js> {
    object: Object<'js>,
    pub(crate) array: bool,
}
impl<'js> Ordinary<'js> {
    pub(crate) fn admit(gate: &ClassBridge<'js>, value: &Value<'js>) -> Result<Self, Failure> {
        let class = gate.classify(value).map_err(|_| Failure::Invalid)?;
        if !matches!(class, ObjectClass::OrdinaryObject | ObjectClass::Array) {
            return Err(Failure::Invalid);
        }
        Ok(Self {
            object: value.as_object().ok_or(Failure::Invalid)?.clone(),
            array: class == ObjectClass::Array,
        })
    }
    pub(crate) fn prototype(&self) -> Option<Object<'js>> {
        self.object.get_prototype()
    }
    pub(crate) fn keys(&self, max: usize) -> Result<Vec<Atom<'js>>, Failure> {
        // The gate excludes proxies/native exotic callbacks before enumeration.
        // Include non-enumerables and symbols; intentionally exclude private slots.
        let keys = self
            .object
            .own_keys::<Atom>(Filter::new().string().symbol());
        if keys.len() > max {
            return Err(Failure::ResourceLimit);
        }
        keys.map(|k| k.map_err(|_| Failure::Engine)).collect()
    }
    pub(crate) fn descriptor(&self, key: &str) -> Result<Descriptor<'js>, Failure> {
        descriptor(&self.object, key)
    }
}

pub(crate) struct Descriptor<'js> {
    pub(crate) enumerable: bool,
    pub(crate) configurable: bool,
    pub(crate) accessor: bool,
    pub(crate) value: Value<'js>,
}

#[allow(unsafe_code)]
fn descriptor<'js>(object: &Object<'js>, key: &str) -> Result<Descriptor<'js>, Failure> {
    // SAFETY: private entry is reachable only through Ordinary::admit. The rooted
    // object and same-context atom live for the call. key is validated Rust UTF-8,
    // passed with its explicit byte length (including embedded NUL). No coercion.
    // Success (>0) initializes all public descriptor fields; failure does not.
    // The owned atom is always freed. Success owns three values: getter/setter
    // are freed once without calling, and value ownership moves once to Value.
    // No private representation, ID or pointer dereference is used.
    unsafe {
        let ctx = object.ctx();
        let raw = ctx.as_raw().as_ptr();
        let atom = qjs::JS_NewAtomLen(raw, key.as_ptr().cast(), key.len() as _);
        if atom == qjs::JS_ATOM_NULL {
            return Err(Failure::Engine);
        }
        let mut desc = std::mem::MaybeUninit::<qjs::JSPropertyDescriptor>::uninit();
        let found = qjs::JS_GetOwnProperty(raw, desc.as_mut_ptr(), object.as_raw(), atom);
        qjs::JS_FreeAtom(raw, atom);
        if found < 0 {
            return Err(Failure::Engine);
        }
        if found == 0 {
            return Err(Failure::Invalid);
        }
        let d = desc.assume_init();
        qjs::JS_FreeValue(raw, d.getter);
        qjs::JS_FreeValue(raw, d.setter);
        Ok(Descriptor {
            enumerable: d.flags & qjs::JS_PROP_ENUMERABLE as i32 != 0,
            configurable: d.flags & qjs::JS_PROP_CONFIGURABLE as i32 != 0,
            accessor: d.flags & qjs::JS_PROP_GETSET as i32 != 0,
            value: Value::from_raw(ctx.clone(), d.value),
        })
    }
}

struct Utf16<'js> {
    ctx: Ctx<'js>,
    ptr: *const u16,
}
impl Drop for Utf16<'_> {
    #[allow(unsafe_code)]
    fn drop(&mut self) {
        // SAFETY: ptr is a non-null owned public UTF16 buffer returned in this
        // live context. This private non-Clone guard frees it once on every exit.
        unsafe { qjs::JS_FreeCStringUTF16(self.ctx.as_raw().as_ptr(), self.ptr) }
    }
}

#[allow(unsafe_code)]
pub(crate) fn text(value: &Value<'_>, max_bytes: usize) -> Result<String, Failure> {
    // Check the actual primitive tag first: JS_ToCStringLenUTF16 otherwise coerces.
    if !value.is_string() {
        return Err(Failure::Invalid);
    }
    // SAFETY: rooted genuine primitive string and associated live Ctx. Public API
    // returns a UTF16-aligned buffer plus initialized length, or null on failure.
    // Its buffer may include lone surrogates, which Rust decodes strictly below.
    // The guard owns/free-pairs it; the slice never outlives the guard. Length is
    // checked for addressable slice size before from_raw_parts; no engine layout.
    unsafe {
        let mut len = 0;
        let ptr =
            qjs::JS_ToCStringLenUTF16(value.ctx().as_raw().as_ptr(), &mut len, value.as_raw());
        if ptr.is_null() {
            return Err(Failure::Engine);
        }
        let buffer = Utf16 {
            ctx: value.ctx().clone(),
            ptr,
        };
        let len = usize::try_from(len).map_err(|_| Failure::ResourceLimit)?;
        if len > max_bytes || len > isize::MAX as usize / 2 {
            return Err(Failure::ResourceLimit);
        }
        let units = std::slice::from_raw_parts(buffer.ptr, len);
        let mut bytes = 0usize;
        for scalar in char::decode_utf16(units.iter().copied()) {
            let scalar = scalar.map_err(|_| Failure::Invalid)?;
            bytes = bytes
                .checked_add(scalar.len_utf8())
                .ok_or(Failure::ResourceLimit)?;
            if bytes > max_bytes {
                return Err(Failure::ResourceLimit);
            }
        }
        // Bounded allocation, no replacement characters/normalization.
        String::from_utf16(units).map_err(|_| Failure::Invalid)
    }
}
