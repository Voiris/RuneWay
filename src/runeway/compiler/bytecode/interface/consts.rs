use std::fmt::Display;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq)]
pub enum ConstValue {
    Str(String),
    // To be continued...
}

impl Display for ConstValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConstValue::Str(s) => write!(f, "<ConstStr:\"{:?}\">", s),
        }
    }
}

pub type ConstsTable = Vec<ConstValue>;
