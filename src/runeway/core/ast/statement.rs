use super::expression::SpannedExpr;
use crate::runeway::core::spanned::Spanned;

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Let {
        name: String,
        value: SpannedExpr,
        annotation: Option<Spanned<String>>,
    },
    LetVoid {
        name: String,
        annotation: Option<Spanned<String>>,
    },
    Assign {
        name: String,
        value: SpannedExpr,
    },
    Expr(SpannedExpr),
    Return(SpannedExpr),
    If {
        condition: SpannedExpr,
        then_branch: Vec<Box<SpannedStatement>>,
        else_branch: Option<Vec<Box<SpannedStatement>>>,
    },
    While {
        condition: SpannedExpr,
        body: Vec<Box<SpannedStatement>>,
    },
    For {
        variable: String,
        iterable: SpannedExpr,
        body: Vec<Box<SpannedStatement>>,
    },
    Break,
    Continue,
    Act {
        name: String,
        parameters: Vec<AnnotatedParameter>,
        return_annotation: Option<Spanned<String>>,
        body: Vec<Box<SpannedStatement>>,
    },
    Import {
        path: String,
        item: ImportItem,
    },
    Assert(SpannedExpr),
    Class {
        name: String,
        body: Vec<Box<SpannedStatement>>,
    },
}

impl Statement {
    pub fn is(&self, other: &Self) -> bool {
        if std::mem::discriminant(self) == std::mem::discriminant(other) {
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AnnotatedParameter {
    pub name: String,
    pub annotation: Option<Spanned<String>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImportItem {
    All,
    Selective(Vec<ImportSymbol>),
    Alias(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImportSymbol {
    pub original: String,
    pub alias: Option<String>,
}

pub type SpannedStatement = Spanned<Statement>;
