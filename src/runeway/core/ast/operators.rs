#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    // Arithmetic
    Add, // +
    Sub, // -
    Mul, // *
    Pow, // **
    Div, // /
    Mod, // %
    // FloorDiv, // ~/

    // Equalising
    Eq,     // ==
    NotEq,  // !=
    Lt,     // <
    LtEq,   // <=
    Gt,     // >
    GtEq,   // >=

    // Logic
    And,
    Or,
}

impl BinaryOperator {
    pub fn get_precedence(&self) -> u8 {
        match self {
            BinaryOperator::Pow => 5,
            BinaryOperator::Mod => 4,
            BinaryOperator::Mul | BinaryOperator::Div => 3,
            BinaryOperator::Add | BinaryOperator::Sub => 2,
            BinaryOperator::Eq | BinaryOperator::NotEq | BinaryOperator::Lt |
            BinaryOperator::LtEq | BinaryOperator::Gt | BinaryOperator::GtEq => 1,
            BinaryOperator::And | BinaryOperator::Or => 0,
            _ => 255,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    Neg,  // -a
    Not,  // !a (not a)
}