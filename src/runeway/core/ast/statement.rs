use super::expression::Expr;

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Let {
        name: String,
        value: Expr,
    },
    Assign {
        name: String,
        value: Expr,
    },
    Expr(Expr),
    Return(Expr),
    If {
        condition: Expr,
        then_branch: Vec<Box<Statement>>,
        else_branch: Option<Vec<Box<Statement>>>,
    },
    While {
        condition: Expr,
        body: Vec<Box<Statement>>,
    },
    For {
        variable: String,
        iterable: Expr,
        body: Vec<Box<Statement>>,
    },
    Break,
    Continue,
    Act {
        name: String,
        parameters: Vec<String>,
        body: Vec<Box<Statement>>,
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
    pub alias: Option<String>
}
