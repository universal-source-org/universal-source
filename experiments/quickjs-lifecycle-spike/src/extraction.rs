//! Bounded shape probe, NOT a production JSON converter/envelope validator.
//! Captures trusted prototypes and opaque class IDs before source runs; uses no private constants.
use rquickjs::{Array, Atom, Ctx, Object, Value, object::Filter, qjs};
use std::collections::BTreeMap;

#[derive(Debug, PartialEq)]
pub enum Owned {
    Null,
    Bool(bool),
    Number(f64),
    Text(String),
    Array(Vec<Owned>),
    Record(BTreeMap<String, Owned>),
}

pub struct Inspector<'js> {
    record_class: qjs::JSClassID,
    array_class: qjs::JSClassID,
    record_proto: Object<'js>,
    array_proto: Object<'js>,
}

fn class(value: &Value<'_>) -> qjs::JSClassID {
    // SAFETY: borrowed live value, public non-executing class query; no ownership transfer.
    unsafe { qjs::JS_GetClassID(value.as_raw()) }
}

impl<'js> Inspector<'js> {
    pub fn new(ctx: &Ctx<'js>) -> Self {
        let record = Object::new(ctx.clone()).unwrap();
        let array = Array::new(ctx.clone()).unwrap();
        Self {
            record_class: class(record.as_value()),
            array_class: class(array.as_value()),
            record_proto: record.get_prototype().unwrap(),
            array_proto: array.as_object().get_prototype().unwrap(),
        }
    }

    pub fn copy(&self, ctx: &Ctx<'js>, v: &Value<'js>) -> Result<Owned, &'static str> {
        self.visit(ctx, v, &mut Vec::new(), &mut 128)
    }

    fn visit(
        &self,
        ctx: &Ctx<'js>,
        v: &Value<'js>,
        ancestors: &mut Vec<Value<'js>>,
        budget: &mut usize,
    ) -> Result<Owned, &'static str> {
        if *budget == 0 || ancestors.len() >= 8 {
            return Err("structural limit");
        }
        *budget -= 1;
        if v.is_null() {
            return Ok(Owned::Null);
        }
        if let Some(b) = v.as_bool() {
            return Ok(Owned::Bool(b));
        }
        if let Some(n) = v.as_number() {
            return n
                .is_finite()
                .then_some(Owned::Number(if n == 0.0 { 0.0 } else { n }))
                .ok_or("nonfinite");
        }
        if let Some(s) = v.as_string() {
            let s = s.to_string().map_err(|_| "string")?;
            if s.len() > 1024 {
                return Err("string limit");
            }
            return Ok(Owned::Text(s));
        }
        if v.is_proxy() {
            return Err("proxy");
        }
        let id = class(v);
        let array = id == self.array_class;
        if id != self.record_class && !array {
            return Err("not ordinary record/array");
        }
        let object = v.as_object().ok_or("object")?;
        let proto = object.get_prototype(); // Class gate excludes proxies/exotic native objects.
        if (array && proto.as_ref() != Some(&self.array_proto))
            || (!array && proto.is_some() && proto.as_ref() != Some(&self.record_proto))
        {
            return Err("prototype");
        }
        if ancestors.contains(v) {
            return Err("cycle");
        }
        ancestors.push(v.clone());
        let mut fields = BTreeMap::new();
        let mut length = None;
        for key in object.own_keys::<Atom>(Filter::new().string().symbol()) {
            let key = key.map_err(|_| "keys")?;
            if key.to_value().map_err(|_| "key")?.is_symbol() {
                return Err("symbol");
            }
            let name = key.to_string().map_err(|_| "key")?;
            let (flags, value) = descriptor(ctx, v, &name)?;
            if flags & qjs::JS_PROP_GETSET as i32 != 0 {
                return Err("accessor");
            }
            if array && name == "length" {
                length = value.as_int().filter(|n| *n >= 0);
                continue;
            }
            if flags & qjs::JS_PROP_ENUMERABLE as i32 == 0 {
                return Err("non-enumerable");
            }
            fields.insert(name, self.visit(ctx, &value, ancestors, budget)?);
        }
        ancestors.pop();
        if !array {
            return Ok(Owned::Record(fields));
        }
        let length = length.ok_or("length")? as usize;
        if length != fields.len() {
            return Err("hole/extra property");
        }
        let values = (0..length)
            .map(|i| fields.remove(&i.to_string()).ok_or("hole/index"))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Owned::Array(values))
    }
}

fn descriptor<'js>(
    ctx: &Ctx<'js>,
    value: &Value<'js>,
    key: &str,
) -> Result<(i32, Value<'js>), &'static str> {
    // SAFETY: same live context/value/atom; caller class-gated ordinary objects/arrays.
    // Success initializes all descriptor values. Transfer value ownership once; free getter/setter
    // exactly once, without invoking them. On failure no initialized descriptor is read.
    unsafe {
        let mut desc = std::mem::MaybeUninit::<qjs::JSPropertyDescriptor>::uninit();
        let raw = ctx.as_raw().as_ptr();
        let atom = qjs::JS_NewAtomLen(raw, key.as_ptr().cast(), key.len() as _);
        if atom == 0 {
            return Err("atom");
        }
        let found = qjs::JS_GetOwnProperty(raw, desc.as_mut_ptr(), value.as_raw(), atom);
        qjs::JS_FreeAtom(raw, atom);
        if found != 1 {
            return Err("descriptor");
        }
        let d = desc.assume_init();
        qjs::JS_FreeValue(raw, d.getter);
        qjs::JS_FreeValue(raw, d.setter);
        Ok((d.flags, Value::from_raw(ctx.clone(), d.value)))
    }
}
