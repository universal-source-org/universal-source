use crate::inspect::{self, Ordinary};
use class_bridge::ClassBridge;
use rquickjs::{Array, Ctx, Object, Value, object::Property};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Portable {
    Null,
    Bool(bool),
    Number(f64),
    Text(String),
    Array(Vec<Portable>),
    Record(BTreeMap<String, Portable>),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Failure {
    Invalid,
    ResourceLimit,
    Engine,
}
// Invalid maps to INVALID_RESULT outgoing, INVALID_ARGUMENT incoming. Engine is
// an embedding failure (never a copied engine Error). ResourceLimit maps unchanged.
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub depth: usize,
    pub nodes: usize,
    pub piece_bytes: usize,
    pub text_bytes: usize,
    pub expanded_bytes: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            depth: 16,
            nodes: 1024,
            piece_bytes: 4096,
            text_bytes: 16384,
            expanded_bytes: 32768,
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Usage {
    pub nodes: usize,
    pub text_bytes: usize,
    pub expanded_bytes: usize,
}
impl Usage {
    fn node(&mut self, depth: usize, l: Limits) -> Result<(), Failure> {
        if depth > l.depth {
            return Err(Failure::ResourceLimit);
        }
        Self::add(&mut self.nodes, 1, l.nodes)?;
        // Portable size model: each node 8 bytes, plus text and record-key bytes.
        Self::add(&mut self.expanded_bytes, 8, l.expanded_bytes)
    }
    fn text(&mut self, s: &str, l: Limits) -> Result<(), Failure> {
        if s.len() > l.piece_bytes {
            return Err(Failure::ResourceLimit);
        }
        Self::add(&mut self.text_bytes, s.len(), l.text_bytes)?;
        Self::add(&mut self.expanded_bytes, s.len(), l.expanded_bytes)
    }
    fn add(n: &mut usize, amount: usize, max: usize) -> Result<(), Failure> {
        *n = n
            .checked_add(amount)
            .filter(|n| *n <= max)
            .ok_or(Failure::ResourceLimit)?;
        Ok(())
    }
}
pub struct Boundary<'js> {
    ctx: Ctx<'js>,
    gate: ClassBridge<'js>,
    object_proto: Object<'js>,
    array_proto: Object<'js>,
    excluded: Vec<Value<'js>>,
    limits: Limits,
}
impl<'js> Boundary<'js> {
    /// Host-only pristine setup before any package code or hardening.
    pub fn new(ctx: &Ctx<'js>, limits: Limits) -> Result<Self, Failure> {
        // Scaffolding upper bound prevents callers from disabling Rust recursion safety.
        if limits.depth > 64 {
            return Err(Failure::ResourceLimit);
        }
        let gate = ClassBridge::new(ctx).map_err(|_| Failure::Engine)?;
        let object_proto = Object::new(ctx.clone())
            .map_err(|_| Failure::Engine)?
            .get_prototype()
            .ok_or(Failure::Engine)?;
        let array_proto = Array::new(ctx.clone())
            .map_err(|_| Failure::Engine)?
            .as_object()
            .get_prototype()
            .ok_or(Failure::Engine)?;
        let roots: Array = ctx
            .eval(include_str!("roots.js"))
            .map_err(|_| Failure::Engine)?;
        let mut excluded = Vec::new();
        for i in 0..roots.len() {
            excluded.push(roots.get(i).map_err(|_| Failure::Engine)?);
        }
        Ok(Self {
            ctx: ctx.clone(),
            gate,
            object_proto,
            array_proto,
            excluded,
            limits,
        })
    }
    /// Register each host helper and nested helper BEFORE exposing it to source.
    /// Plain data intended for transfer is intentionally not registered.
    pub fn exclude_helper(&mut self, value: &Value<'js>) -> Result<(), Failure> {
        if self.ctx.as_raw() != value.ctx().as_raw() || !value.is_object() {
            return Err(Failure::Invalid);
        }
        if self.excluded.len() >= 8192 {
            return Err(Failure::ResourceLimit);
        }
        if !self.excluded.contains(value) {
            self.excluded.push(value.clone());
        }
        Ok(())
    }
    pub fn outgoing(&self, value: &Value<'js>) -> Result<(Portable, Usage), Failure> {
        let mut usage = Usage::default();
        let result = self.visit(value, 0, &mut Vec::new(), &mut usage)?;
        Ok((result, usage))
    }
    fn visit(
        &self,
        v: &Value<'js>,
        depth: usize,
        path: &mut Vec<Value<'js>>,
        u: &mut Usage,
    ) -> Result<Portable, Failure> {
        if self.ctx.as_raw() != v.ctx().as_raw() {
            return Err(Failure::Invalid);
        }
        u.node(depth, self.limits)?;
        if v.is_null() {
            return Ok(Portable::Null);
        }
        if let Some(b) = v.as_bool() {
            return Ok(Portable::Bool(b));
        }
        if let Some(n) = v.as_number() {
            return finite(n).map(Portable::Number);
        }
        if v.is_string() {
            let remaining = self
                .limits
                .text_bytes
                .saturating_sub(u.text_bytes)
                .min(self.limits.expanded_bytes.saturating_sub(u.expanded_bytes))
                .min(self.limits.piece_bytes);
            let s = inspect::text(v, remaining)?;
            u.text(&s, self.limits)?;
            return Ok(Portable::Text(s));
        }
        let object = Ordinary::admit(&self.gate, v)?; // Before ALL object reflection.
        if self.excluded.contains(v) || path.contains(v) {
            return Err(Failure::Invalid);
        }
        let proto = object.prototype();
        if object.array {
            if proto.as_ref() != Some(&self.array_proto) {
                return Err(Failure::Invalid);
            }
        } else if proto.is_some() && proto.as_ref() != Some(&self.object_proto) {
            return Err(Failure::Invalid);
        }
        let keys = object.keys(
            self.limits
                .nodes
                .saturating_sub(u.nodes)
                .saturating_add(usize::from(object.array)),
        )?;
        let mut fields = BTreeMap::new();
        let mut length = None;
        // Get descriptors and validate shape before visiting children. All limits
        // bound temporary key/descriptor storage as well as the eventual copy.
        for atom in keys {
            // Do NOT use Atom::to_string (NUL/CStr and unchecked UTF8 in this pin).
            let key_value = atom.to_value().map_err(|_| Failure::Engine)?;
            let key = inspect::text(
                &key_value,
                if object.array {
                    self.limits.piece_bytes.max(10)
                } else {
                    self.limits.piece_bytes
                },
            )?;
            let d = object.descriptor(&key)?;
            if d.accessor {
                return Err(Failure::Invalid);
            }
            if object.array && key == "length" {
                let n = d.value.as_number().ok_or(Failure::Invalid)?;
                if d.enumerable
                    || d.configurable
                    || n < 0.0
                    || n > u32::MAX as f64
                    || n.fract() != 0.0
                {
                    return Err(Failure::Invalid);
                }
                length = Some(n as usize);
                continue;
            }
            if !d.enumerable && !object.array {
                return Err(Failure::Invalid);
            }
            if !object.array {
                u.text(&key, self.limits)?;
            }
            fields.insert(key, d.value);
        }
        path.push(v.clone());
        let result = (|| {
            if object.array {
                let length = length.ok_or(Failure::Invalid)?;
                if length != fields.len() {
                    return Err(Failure::Invalid);
                }
                // Validate all indices before copying; canonical decimal index order.
                if (0..length).any(|i| !fields.contains_key(&i.to_string())) {
                    return Err(Failure::Invalid);
                }
                let mut out = Vec::with_capacity(length);
                for i in 0..length {
                    out.push(self.visit(
                        &fields.remove(&i.to_string()).ok_or(Failure::Invalid)?,
                        depth + 1,
                        path,
                        u,
                    )?);
                }
                Ok(Portable::Array(out))
            } else {
                let mut out = BTreeMap::new();
                for (key, value) in fields {
                    out.insert(key, self.visit(&value, depth + 1, path, u)?);
                }
                Ok(Portable::Record(out))
            }
        })();
        path.pop();
        result
    }
    pub fn incoming(&self, value: &Portable) -> Result<(Value<'js>, Usage), Failure> {
        let mut usage = Usage::default();
        self.validate_host(value, 0, &mut usage)?;
        // No engine allocation/construction until the entire host tree validates.
        Ok((self.build(value)?, usage))
    }
    fn validate_host(&self, v: &Portable, depth: usize, u: &mut Usage) -> Result<(), Failure> {
        u.node(depth, self.limits)?;
        match v {
            Portable::Number(n) => {
                finite(*n)?;
            }
            Portable::Text(s) => u.text(s, self.limits)?,
            Portable::Array(values) => {
                for v in values {
                    self.validate_host(v, depth + 1, u)?;
                }
            }
            Portable::Record(fields) => {
                for (key, v) in fields {
                    u.text(key, self.limits)?;
                    self.validate_host(v, depth + 1, u)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
    fn build(&self, v: &Portable) -> Result<Value<'js>, Failure> {
        let ctx = self.ctx.clone();
        Ok(match v {
            Portable::Null => Value::new_null(ctx),
            Portable::Bool(b) => Value::new_bool(ctx, *b),
            Portable::Number(n) => Value::new_number(ctx, finite(*n)?),
            Portable::Text(s) => rquickjs::String::from_str(ctx, s)
                .map_err(|_| Failure::Engine)?
                .into_value(),
            Portable::Record(fields) => {
                let object = Object::new_proto(ctx, Some(&self.object_proto))
                    .map_err(|_| Failure::Engine)?;
                // UTF8 lexicographic ordering on valid Rust strings equals scalar
                // order. BTreeMap iteration is independent of source builtins.
                for (key, value) in fields {
                    define(&object, key.as_str(), self.build(value)?)?;
                }
                object.into_value()
            }
            Portable::Array(values) => {
                let array = Array::new(ctx).map_err(|_| Failure::Engine)?;
                array
                    .as_object()
                    .set_prototype(Some(&self.array_proto))
                    .map_err(|_| Failure::Engine)?;
                for (i, value) in values.iter().enumerate() {
                    define(
                        array.as_object(),
                        i.to_string().as_str(),
                        self.build(value)?,
                    )?;
                }
                array.into_value()
            }
        })
    }
}
fn finite(n: f64) -> Result<f64, Failure> {
    if n.is_finite() {
        Ok(if n == 0.0 { 0.0 } else { n })
    } else {
        Err(Failure::Invalid)
    }
}
fn define<'js>(object: &Object<'js>, key: &str, value: Value<'js>) -> Result<(), Failure> {
    // Own data definition on a NEW ordinary target: no inherited setter/iterator.
    object
        .prop(
            key,
            Property::from(value).writable().enumerable().configurable(),
        )
        .map_err(|_| Failure::Engine)
}
