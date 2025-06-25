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
    Act {
        name: String,
        parameters: Vec<String>,
        body: Vec<Box<Statement>>,
    }
}