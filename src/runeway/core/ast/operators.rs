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
    Eq,    // ==
    NotEq, // !=
    Lt,    // <
    LtEq,  // <=
    Gt,    // >
    GtEq,  // >=

    // Logic
    And,
    Or,

    // Other
    Is,
}

impl BinaryOperator {
    pub fn get_precedence(&self) -> u8 {
        match self {
            BinaryOperator::Pow => 6,
            BinaryOperator::Mod => 5,
            BinaryOperator::Mul | BinaryOperator::Div => 4,
            BinaryOperator::Add | BinaryOperator::Sub => 3,
            BinaryOperator::Eq
            | BinaryOperator::NotEq
            | BinaryOperator::Lt
            | BinaryOperator::LtEq
            | BinaryOperator::Gt
            | BinaryOperator::GtEq => 2,
            BinaryOperator::Is => 1,
            BinaryOperator::And | BinaryOperator::Or => 0
            // _ => 255,
        }
    }

    pub fn display(&self) -> &'static str {
        match self {
            BinaryOperator::Add => "+",
            BinaryOperator::Sub => "-",
            BinaryOperator::Mul => "*",
            BinaryOperator::Div => "/",
            BinaryOperator::Mod => "%",
            BinaryOperator::Pow => "**",

            BinaryOperator::Eq => "==",
            BinaryOperator::NotEq => "!=",
            BinaryOperator::Lt => "<",
            BinaryOperator::LtEq => "<=",
            BinaryOperator::Gt => ">",
            BinaryOperator::GtEq => ">=",

            BinaryOperator::And => "and",
            BinaryOperator::Or => "or",
            BinaryOperator::Is => "is",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    Neg, // -a
    Not, // !a (not a)
}

impl UnaryOperator {
    pub fn display(&self) -> &'static str {
        match self {
            UnaryOperator::Neg => "-",
            UnaryOperator::Not => "!",
        }
    }
}
