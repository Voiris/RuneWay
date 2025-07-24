use std::fmt::Display;

#[derive(Debug)]
pub enum IRConstValue {
    Str(String),
    // To be continued...
}

impl Display for IRConstValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IRConstValue::Str(s) => write!(f, "Str {:?}", s),
        }
    }
}
