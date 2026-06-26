use runec_ast::statement::SpannedStmt;
use runec_errors::diagnostics::Diagnostic;

pub struct ParseResult<'src, 'diag> {
    pub stmts: Vec<SpannedStmt<'src>>,
    pub diags: Vec<Diagnostic<'diag>>,
}

impl<'src, 'diag> ParseResult<'src, 'diag> {
    pub fn new() -> Self {
        Self {
            stmts: Vec::new(),
            diags: Vec::new(),
        }
    }
}

impl Default for ParseResult<'_, '_> {
    fn default() -> Self {
        Self::new()
    }
}
