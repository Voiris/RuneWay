use serde::{Deserialize, Serialize};
use crate::runeway::compiler::bytecode::interface::consts::ConstValue;
use crate::runeway::compiler::bytecode::interface::opcode::Opcode;

mod consts;
mod opcode;
mod application;

#[derive(Serialize, Deserialize)]
pub struct CompiledPack {
    ops: Vec<Opcode>,
    consts: Vec<ConstValue>
}
