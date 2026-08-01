use runec_errors::diagnostics::Diagnostic;
use runec_hir::map::HirMap;

use crate::resolving::Resolver;
use crate::typeck::{TypeCheckResult, TypeChecker, TypeInfo};

pub struct SemanticResult<'src> {
    pub info: TypeInfo<'src>,
    pub diags: Vec<Diagnostic<'static>>,
}

impl<'src> SemanticResult<'src> {
    pub fn has_errors(&self) -> bool {
        !self.diags.is_empty()
    }
}

pub struct SemanticChecker;

impl SemanticChecker {
    pub fn new() -> Self {
        Self
    }

    pub fn check<'src>(&self, hir: &mut HirMap<'src>) -> SemanticResult<'src> {
        let resolve = Resolver::new().resolve(hir);
        let TypeCheckResult {
            info,
            diags: mut type_diags,
        } = TypeChecker::new().check(hir);
        let mut diags = resolve.diags;
        diags.append(&mut type_diags);

        SemanticResult { info, diags }
    }
}

impl Default for SemanticChecker {
    fn default() -> Self {
        Self::new()
    }
}
