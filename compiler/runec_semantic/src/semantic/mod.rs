use runec_hir::map::HirMap;

use crate::resolving::{ResolveError, Resolver};
use crate::typeck::{TypeCheckResult, TypeChecker, TypeError, TypeInfo};

pub struct SemanticResult<'src> {
    pub info: TypeInfo<'src>,
    pub resolve_errors: Vec<ResolveError>,
    pub type_errors: Vec<TypeError>,
}

impl<'src> SemanticResult<'src> {
    pub fn has_errors(&self) -> bool {
        !self.resolve_errors.is_empty() || !self.type_errors.is_empty()
    }
}

pub struct SemanticChecker;

impl SemanticChecker {
    pub fn new() -> Self {
        Self
    }

    pub fn check<'src>(&self, hir: &mut HirMap<'src>) -> SemanticResult<'src> {
        let resolve = Resolver::new().resolve(hir);
        let TypeCheckResult { info, errors } = TypeChecker::new().check(hir);

        SemanticResult {
            info,
            resolve_errors: resolve.errors,
            type_errors: errors,
        }
    }
}

impl Default for SemanticChecker {
    fn default() -> Self {
        Self::new()
    }
}
