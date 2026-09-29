//! Non-production RegExp compile-route and literal-discovery probes.
//! Not an admission implementation, facade, parser selection or policy.
#![forbid(unsafe_code)]

use boa_ast::{
    expression::RegExpLiteral,
    scope::Scope,
    visitor::{VisitWith, Visitor},
};
use boa_interner::Interner;
use boa_parser::{Parser, Source};
use rquickjs::{Context, Module, Runtime};
use std::{cell::Cell, ops::ControlFlow, rc::Rc};

pub const HEAP: usize = 8 * 1024 * 1024;
pub const STACK: usize = 128 * 1024;

/// Owner-thread interrupt accounting, inaccessible to source.
#[derive(Default)]
pub struct Control {
    pub armed: Cell<bool>,
    pub callbacks: Cell<usize>,
}

/// A bare full-intrinsic realm with limits and a counting interrupt handler that
/// requests interruption on every armed callback. No hardening: route probes
/// observe which pristine native paths compile.
pub fn armed_realm() -> (Runtime, Context, Rc<Control>) {
    let rt = Runtime::new().unwrap();
    rt.set_memory_limit(HEAP);
    rt.set_max_stack_size(STACK);
    let control = Rc::new(Control::default());
    let state = control.clone();
    rt.set_interrupt_handler(Some(Box::new(move || {
        if !state.armed.get() {
            return false;
        }
        state.callbacks.set(state.callbacks.get() + 1);
        true
    })));
    let ctx = Context::full(&rt).unwrap();
    (rt, ctx, control)
}

/// `"\\k<a>".repeat(n) + "(?<a>x)"`: the preserved forward-reference family.
pub fn forward_reference_pattern(references: usize) -> String {
    format!("{}(?<a>x)", "\\k<a>".repeat(references))
}

/// Compile-only QuickJS module declaration of the exact bytes, with the
/// interrupt handler armed for the whole call. Returns (accepted, callbacks).
pub fn quickjs_declare(source: &str) -> (bool, usize) {
    let (rt, context, control) = armed_realm();
    let accepted = context.with(|ctx| {
        control.armed.set(true);
        let accepted = Module::declare(ctx.clone(), "fixture.js", source).is_ok();
        control.armed.set(false);
        drop(ctx.catch());
        accepted
    });
    let callbacks = control.callbacks.get();
    drop(context);
    drop(rt);
    (accepted, callbacks)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Literal {
    pub pattern: String,
    pub flags: String,
    pub utf16_len: usize,
}

/// Every RegExp literal Boa's public AST visitor reaches, in source order.
/// Boa's lexer also validates each body with `regress` before interning it.
pub fn boa_literals(bytes: &[u8]) -> Result<Vec<Literal>, String> {
    struct Collect<'a> {
        interner: &'a Interner,
        out: Vec<Literal>,
    }
    impl<'ast> Visitor<'ast> for Collect<'_> {
        type BreakTy = ();
        fn visit_reg_exp_literal(&mut self, node: &'ast RegExpLiteral) -> ControlFlow<()> {
            let pattern = self
                .interner
                .resolve_expect(node.pattern())
                .utf16()
                .to_vec();
            let flags = self.interner.resolve_expect(node.flags()).utf16().to_vec();
            self.out.push(Literal {
                pattern: String::from_utf16(&pattern).expect("fixture has Unicode scalar text"),
                flags: String::from_utf16(&flags).expect("ASCII flags"),
                utf16_len: pattern.len(),
            });
            ControlFlow::Continue(())
        }
    }
    let mut interner = Interner::default();
    let module = Parser::new(Source::from_bytes(bytes))
        .parse_module(&Scope::new_global(), &mut interner)
        .map_err(|error| error.to_string())?;
    let mut collect = Collect {
        interner: &interner,
        out: Vec::new(),
    };
    let _ = module.visit_with(&mut collect);
    Ok(collect.out)
}

/// Oxc module-mode acceptance; `validate` enables its RegExp pattern parser.
/// Any diagnostic or fatal error rejects; a recovered tree is never success.
pub fn oxc_accepts(source: &str, validate: bool) -> bool {
    let allocator = oxc_allocator::Allocator::default();
    let parsed = oxc_parser::Parser::new(&allocator, source, oxc_span::SourceType::mjs())
        .with_options(oxc_parser::ParseOptions {
            parse_regular_expression: validate,
            ..oxc_parser::ParseOptions::default()
        })
        .parse();
    !parsed.fatal_error && parsed.diagnostics.is_empty()
}

#[cfg(test)]
mod tests;
