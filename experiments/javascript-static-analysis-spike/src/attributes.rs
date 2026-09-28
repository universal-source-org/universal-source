//! Narrow experiment-only supplemental check. Never a complete RFC validator.
use super::analysis::{inspect, parse, symbol};
use boa_ast::{ModuleItem, declaration::ExportDeclaration};
use boa_interner::Interner;
use oxc_allocator::Allocator;
use oxc_ast::ast::ModuleDeclaration;
use oxc_parser::{ParseOptions, Parser};
use oxc_span::{GetSpan, SourceType, Span};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edge {
    pub kind: &'static str,
    pub specifier: String,
    pub declaration: Span,
    pub clause: Option<Span>,
    pub entries: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Failure {
    Utf8,
    BoaSyntax,
    OxcSyntax,
    EdgeDisagreement,
    Attributes,
    OtherProfile,
}

pub fn oxc_edges(bytes: &[u8]) -> Result<Vec<Edge>, Failure> {
    let source = std::str::from_utf8(bytes).map_err(|_| Failure::Utf8)?;
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::mjs())
        .with_options(ParseOptions {
            parse_regular_expression: true,
            ..ParseOptions::default()
        })
        .parse();
    // Never trust a recovered or partial tree just because it exists.
    if parsed.fatal_error || !parsed.diagnostics.is_empty() {
        return Err(Failure::OxcSyntax);
    }
    let mut edges = Vec::new();
    for statement in &parsed.program.body {
        let Some(module) = statement.as_module_declaration() else {
            continue;
        };
        let Some(source) = module.source() else {
            continue;
        };
        let kind = match module {
            ModuleDeclaration::ImportDeclaration(_) => "import",
            ModuleDeclaration::ExportFromDeclaration(_)
            | ModuleDeclaration::ExportAllDeclaration(_) => "reexport",
            _ => unreachable!("source-bearing JS module declaration"),
        };
        let clause = module.with_clause();
        edges.push(Edge {
            kind,
            specifier: source.value.to_string(),
            declaration: module.span(),
            clause: clause.map(|c| c.span),
            entries: clause.map_or(0, |c| c.with_entries.len()),
        });
    }
    Ok(edges)
}

pub fn agree(boa: &[(&str, String)], oxc: &[Edge]) -> Result<(), Failure> {
    // A small consistency witness, not equivalence of two entire ASTs.
    if boa.len() != oxc.len()
        || boa
            .iter()
            .zip(oxc)
            .any(|((kind, specifier), edge)| *kind != edge.kind || *specifier != edge.specifier)
    {
        return Err(Failure::EdgeDisagreement);
    }
    Ok(())
}

pub fn preflight(bytes: &[u8], operations: Option<&[&str]>) -> Result<(), Failure> {
    let mut interner = Interner::default();
    let ast = parse(bytes, &mut interner).map_err(|_| Failure::BoaSyntax)?;
    let edges = oxc_edges(bytes)?;
    let boa_edges: Vec<_> = ast
        .items()
        .items()
        .iter()
        .filter_map(|item| match item {
            ModuleItem::ImportDeclaration(i) => {
                Some(("import", symbol(&interner, i.specifier().sym())))
            }
            ModuleItem::ExportDeclaration(e) => match e.as_ref() {
                ExportDeclaration::ReExport { specifier, .. } => {
                    Some(("reexport", symbol(&interner, specifier.sym())))
                }
                _ => None,
            },
            _ => None,
        })
        .collect();
    agree(&boa_edges, &edges)?;
    if edges.iter().any(|e| e.clause.is_some()) {
        return Err(Failure::Attributes);
    }
    if !inspect(&ast, &interner, "fixture.js", operations, 1000)
        .violations
        .is_empty()
    {
        return Err(Failure::OtherProfile);
    }
    Ok(())
}
