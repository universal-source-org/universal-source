//! Exact-byte RegExp literal preflight before QuickJS sees a module: a byte bound,
//! then the literal bound from Oxc's raw token stream (no pattern validation),
//! then Boa's AST visitor, then fail-closed agreement. No parser is selected.
use boa_ast::{
    expression::RegExpLiteral,
    scope::Scope,
    visitor::{VisitWith, Visitor},
};
use boa_interner::Interner;
use rquickjs::{
    Ctx, Error, Module, Result,
    loader::{ImportAttributes, Loader, Resolver},
    module::Declared,
};
use std::{cell::RefCell, collections::HashMap, ops::ControlFlow, rc::Rc};

/// Defense-in-depth module byte bound for this spike.
pub const MODULE_BYTES: usize = 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Literal {
    pub body: String,
    pub flags: String,
}

impl Literal {
    pub fn units(&self) -> usize {
        self.body.encode_utf16().count()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Rejection {
    ModuleBytes(usize),
    LiteralBound { units: usize },
    OxcSyntax,
    BoaSyntax(String),
    Disagreement,
}

impl Rejection {
    /// Loading engine budget failure (load resource diagnostic), as opposed to invalid source.
    pub fn is_resource(&self) -> bool {
        matches!(self, Self::ModuleBytes(_) | Self::LiteralBound { .. })
    }
}

/// Literal tokens in source order from Oxc raw mode; any diagnostic rejects.
pub fn oxc_literals(source: &str) -> std::result::Result<Vec<Literal>, Rejection> {
    let allocator = oxc_allocator::Allocator::default();
    let parsed = oxc_parser::Parser::new(&allocator, source, oxc_span::SourceType::mjs())
        .with_options(oxc_parser::ParseOptions {
            parse_regular_expression: false,
            ..oxc_parser::ParseOptions::default()
        })
        .with_config(oxc_parser::config::TokensParserConfig)
        .parse();
    if parsed.fatal_error || !parsed.diagnostics.is_empty() {
        return Err(Rejection::OxcSyntax);
    }
    let mut out = Vec::new();
    for token in parsed.tokens.iter() {
        if token.kind() != oxc_parser::Kind::RegExp {
            continue;
        }
        let raw = &source[token.start() as usize..token.end() as usize];
        let close = raw.rfind('/').filter(|&i| i > 0 && raw.starts_with('/'));
        let Some(close) = close else {
            return Err(Rejection::Disagreement);
        };
        out.push(Literal {
            body: raw[1..close].to_owned(),
            flags: raw[close + 1..].to_owned(),
        });
    }
    Ok(out)
}

/// Every literal Boa's public visitor reaches. Boa validates each body with `regress`.
pub fn boa_literals(source: &str) -> std::result::Result<Vec<Literal>, Rejection> {
    struct Collect<'a> {
        interner: &'a Interner,
        out: Vec<Literal>,
    }
    impl<'ast> Visitor<'ast> for Collect<'_> {
        type BreakTy = ();
        fn visit_reg_exp_literal(&mut self, node: &'ast RegExpLiteral) -> ControlFlow<()> {
            let text = |sym| {
                String::from_utf16(self.interner.resolve_expect(sym).utf16())
                    .expect("UTF-8 source has no lone surrogates")
            };
            self.out.push(Literal {
                body: text(node.pattern()),
                flags: text(node.flags()),
            });
            ControlFlow::Continue(())
        }
    }
    let mut interner = Interner::default();
    let module = boa_parser::Parser::new(boa_parser::Source::from_bytes(source.as_bytes()))
        .parse_module(&Scope::new_global(), &mut interner)
        .map_err(|e| Rejection::BoaSyntax(e.to_string()))?;
    let mut collect = Collect {
        interner: &interner,
        out: Vec::new(),
    };
    let _ = module.visit_with(&mut collect);
    Ok(collect.out)
}

pub fn agree(oxc: &[Literal], boa: &[Literal]) -> std::result::Result<(), Rejection> {
    if oxc == boa {
        Ok(())
    } else {
        Err(Rejection::Disagreement)
    }
}

/// Measure before any pattern validation; Boa only sees modules whose literals fit.
pub fn preflight(source: &str, bound: usize) -> std::result::Result<Vec<Literal>, Rejection> {
    if source.len() > MODULE_BYTES {
        return Err(Rejection::ModuleBytes(source.len()));
    }
    let oxc = oxc_literals(source)?;
    if let Some(units) = oxc.iter().map(Literal::units).find(|&u| u > bound) {
        return Err(Rejection::LiteralBound { units });
    }
    let boa = boa_literals(source)?;
    agree(&oxc, &boa)?;
    Ok(oxc)
}

/// Preflights the entry, then declares it. Declaration eagerly loads the graph, so
/// a rejected dependency surfaces here as a QuickJS loader error (see `Graph::load`).
pub fn declare<'js>(
    ctx: &Ctx<'js>,
    name: &str,
    source: &str,
    bound: usize,
) -> std::result::Result<Result<Module<'js, Declared>>, Rejection> {
    preflight(source, bound)?;
    Ok(Module::declare(ctx.clone(), name, source))
}

/// In-memory static graph; every imported module is preflighted before declaration.
pub struct Graph {
    pub sources: HashMap<String, String>,
    pub bound: usize,
    pub rejection: Rc<RefCell<Option<(String, Rejection)>>>,
}

pub struct GraphResolver(pub Vec<String>);

impl Resolver for GraphResolver {
    fn resolve<'js>(
        &mut self,
        _ctx: &Ctx<'js>,
        base: &str,
        name: &str,
        _attributes: Option<ImportAttributes<'js>>,
    ) -> Result<String> {
        let name = name.trim_start_matches("./");
        if self.0.iter().any(|n| n == name) {
            Ok(name.to_owned())
        } else {
            Err(Error::new_resolving(base, name))
        }
    }
}

impl Loader for Graph {
    fn load<'js>(
        &mut self,
        ctx: &Ctx<'js>,
        name: &str,
        _attributes: Option<ImportAttributes<'js>>,
    ) -> Result<Module<'js, Declared>> {
        let source = self
            .sources
            .get(name)
            .ok_or_else(|| Error::new_loading(name))?;
        match preflight(source, self.bound) {
            Ok(_) => Module::declare(ctx.clone(), name, source.clone()),
            Err(rejection) => {
                *self.rejection.borrow_mut() = Some((name.to_owned(), rejection));
                Err(Error::new_loading_message(name, "preflight rejected"))
            }
        }
    }
}
