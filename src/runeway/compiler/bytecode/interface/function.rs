use std::fmt::{Debug, Display};
use serde::{Deserialize, Serialize};
use crate::runeway::compiler::bytecode::interface::opcode::Opcode;
use crate::runeway::core::spanned::Spanned;

#[derive(Serialize, Deserialize, Debug)]
pub struct CompiledFunction {
    pub parameters: Vec<String>,
    pub ops: Vec<Opcode>
}

impl Display for CompiledFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(
            format_args!("act({}) {{{}}}",
                         self.parameters.len(),
                         self.ops.len()
            )
        )
    }
}
