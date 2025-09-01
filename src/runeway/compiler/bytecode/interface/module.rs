use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use crate::runeway::compiler::bytecode::interface::function::CompiledFunction;

#[derive(Serialize, Deserialize, Debug)]
pub enum UserDefinedStatement {
    // TODO: Import(String),
    // TODO: Class(String, CompiledClass),
    Function(String, CompiledFunction),
}

pub enum ModuleCompileData {
    Standard(String),
    UserDefined {
        path: String,
        filename: String,
        code: String,
    },
    AlreadyLoaded,
}


#[derive(Serialize, Deserialize, Debug)]
pub enum CompiledModule {
    Standard,
    UserDefined(Vec<UserDefinedStatement>),
}
