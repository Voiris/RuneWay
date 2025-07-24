use std::ops::Range;
use crate::runeway::core::spanned::Spanned;
use super::expression::{Expr, SpannedExpr};

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Let {
        name: String,
        value: SpannedExpr,
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
        parameters: Vec<String>,
        body: Vec<Box<SpannedStatement>>,
    },

    Import {
        path: String,
        item: ImportItem,
    }
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
