//! Deliberately incomplete AST preflight probe, never a production validator.
use boa_ast::{
    Module, ModuleItem, Span, Spanned,
    declaration::{Declaration, ExportDeclaration, LexicalDeclaration, ReExportKind},
    expression::{Await, Call, Expression, ImportCall, ImportMeta},
    function::FunctionBody,
    scope::Scope,
    statement::iteration::ForOfLoop,
    visitor::{VisitWith, Visitor},
};
use boa_interner::{Interner, Sym};
use boa_parser::{Parser, Source};
use std::ops::ControlFlow;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub module: String,
    pub category: &'static str,
    pub span: Option<Span>,
}

#[derive(Debug, Default)]
pub struct Report {
    pub operations: Vec<String>,
    pub imports: Vec<String>,
    pub violations: Vec<Diagnostic>,
    // Observations only: RFC §9 requires runtime denial, not syntax rejection.
    pub dynamic_code_sites: Vec<Span>,
    pub nested_awaits: usize,
    pub expression_visits: usize,
    pub max_expression_depth: usize,
    pub module_items: usize,
}

pub fn symbol(interner: &Interner, sym: Sym) -> String {
    // Fail rather than normalize or silently replace an unpaired surrogate.
    String::from_utf16(interner.resolve_expect(sym).utf16())
        .expect("fixture has Unicode scalar text")
}

pub fn parse(bytes: &[u8], interner: &mut Interner) -> Result<Module, boa_parser::Error> {
    Parser::new(Source::from_bytes(bytes)).parse_module(&Scope::new_global(), interner)
}

impl Report {
    fn reject(&mut self, module: &str, category: &'static str, span: Option<Span>) {
        self.violations.push(Diagnostic {
            module: module.into(),
            category,
            span,
        });
    }
}

pub fn inspect(
    ast: &Module,
    interner: &Interner,
    module: &str,
    operations: Option<&[&str]>,
    expression_limit: usize,
) -> Report {
    let mut report = Report {
        module_items: ast.items().items().len(),
        ..Report::default()
    };
    for item in ast.items().items() {
        match item {
            ModuleItem::ImportDeclaration(import) => {
                report
                    .imports
                    .push(symbol(interner, import.specifier().sym()));
                if !import.attributes().is_empty() {
                    report.reject(module, "IMPORT_ATTRIBUTES", None);
                }
            }
            ModuleItem::ExportDeclaration(export) => match export.as_ref() {
                ExportDeclaration::Declaration(Declaration::FunctionDeclaration(f)) => {
                    report.operations.push(symbol(interner, f.name().sym()));
                }
                ExportDeclaration::Declaration(Declaration::AsyncFunctionDeclaration(f)) => {
                    report.operations.push(symbol(interner, f.name().sym()));
                }
                ExportDeclaration::ReExport {
                    kind,
                    specifier,
                    attributes,
                } => {
                    report.imports.push(symbol(interner, specifier.sym()));
                    if !attributes.is_empty() {
                        report.reject(module, "IMPORT_ATTRIBUTES", None);
                    }
                    if matches!(kind, ReExportKind::Namespaced { .. }) {
                        report.reject(module, "STAR_EXPORT", None);
                    } else if operations.is_some() {
                        report.reject(module, "ENTRY_REEXPORT", None);
                    }
                }
                ExportDeclaration::List(_) => {
                    if operations.is_some() {
                        report.reject(module, "EXPORT_LIST", None);
                    }
                }
                ExportDeclaration::Declaration(_) | ExportDeclaration::VarStatement(_) => {
                    if operations.is_some() {
                        report.reject(module, "OPERATION_DECLARATION", None);
                    }
                }
                _ => report.reject(module, "DEFAULT_EXPORT", None),
            },
            _ => {}
        }
    }
    if let Some(expected) = operations {
        let mut actual = report.operations.clone();
        actual.sort();
        let mut expected: Vec<_> = expected.iter().map(|s| (*s).to_owned()).collect();
        expected.sort();
        if actual != expected {
            report.reject(module, "OPERATION_NAMES", None);
        }
    }
    let mut visitor = SyntaxVisitor {
        module,
        interner,
        report: &mut report,
        function_depth: 0,
        expression_depth: 0,
        expression_limit,
    };
    if ast.visit_with(&mut visitor).is_break() {
        report.reject(module, "EXPRESSION_LIMIT", None);
    }
    report
}

struct SyntaxVisitor<'a> {
    module: &'a str,
    interner: &'a Interner,
    report: &'a mut Report,
    function_depth: usize,
    expression_depth: usize,
    expression_limit: usize,
}

impl<'ast> Visitor<'ast> for SyntaxVisitor<'_> {
    type BreakTy = ();

    fn visit_expression(&mut self, node: &'ast Expression) -> ControlFlow<()> {
        self.report.expression_visits += 1;
        if self.report.expression_visits > self.expression_limit {
            return ControlFlow::Break(());
        }
        self.expression_depth += 1;
        self.report.max_expression_depth =
            self.report.max_expression_depth.max(self.expression_depth);
        let result = node.visit_with(self);
        self.expression_depth -= 1;
        result
    }

    fn visit_function_body(&mut self, node: &'ast FunctionBody) -> ControlFlow<()> {
        self.function_depth += 1;
        let result = node.visit_with(self);
        self.function_depth -= 1;
        result
    }

    fn visit_import_call(&mut self, node: &'ast ImportCall) -> ControlFlow<()> {
        self.report
            .reject(self.module, "DYNAMIC_IMPORT", Some(node.span()));
        node.visit_with(self)
    }

    fn visit_import_meta(&mut self, node: &'ast ImportMeta) -> ControlFlow<()> {
        self.report
            .reject(self.module, "IMPORT_META", Some(node.span()));
        node.visit_with(self)
    }

    fn visit_await(&mut self, node: &'ast Await) -> ControlFlow<()> {
        if self.function_depth == 0 {
            self.report
                .reject(self.module, "TOP_LEVEL_AWAIT", Some(node.span()));
        } else {
            self.report.nested_awaits += 1;
        }
        node.visit_with(self)
    }

    fn visit_for_of_loop(&mut self, node: &'ast ForOfLoop) -> ControlFlow<()> {
        if node.r#await() && self.function_depth == 0 {
            self.report.reject(self.module, "TOP_LEVEL_AWAIT", None);
        }
        node.visit_with(self)
    }

    fn visit_lexical_declaration(&mut self, node: &'ast LexicalDeclaration) -> ControlFlow<()> {
        if matches!(
            node,
            LexicalDeclaration::Using(_) | LexicalDeclaration::AwaitUsing(_)
        ) {
            self.report.reject(self.module, "OUTSIDE_ES2023", None);
        }
        node.visit_with(self)
    }

    fn visit_call(&mut self, node: &'ast Call) -> ControlFlow<()> {
        if let Expression::Identifier(name) = node.function()
            && matches!(
                symbol(self.interner, name.sym()).as_str(),
                "eval" | "Function"
            )
        {
            self.report.dynamic_code_sites.push(node.span());
        }
        node.visit_with(self)
    }
}

pub fn fingerprint(bytes: &[u8]) -> u64 {
    // FNV-1a is an evidence checksum, not a security boundary. Tests also compare bytes.
    bytes.iter().fold(0xcbf29ce484222325, |hash, b| {
        (hash ^ u64::from(*b)).wrapping_mul(0x100000001b3)
    })
}
