use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::runeway::compiler::bytecode::interface::function::CompiledFunction;
use crate::runeway::compiler::bytecode::interface::module::CompiledModule;
use super::consts::{ConstValue, ConstsTable};
use super::opcode::Opcode;

#[derive(Serialize, Deserialize, Debug)]
pub struct CompiledApplication {
    pub entry_module: usize,
    pub entry_function: String,
    pub consts_table: ConstsTable,
    pub modules: Vec<CompiledModule>,
}

impl CompiledApplication {
    pub fn new(entry_module: usize, entry_function: impl ToString) -> Self {
        Self {
            entry_module,
            entry_function: entry_function.to_string(),
            consts_table: ConstsTable::new(),
            modules: Vec::new(),
        }
    }

    pub fn add_const(&mut self, value: ConstValue) -> usize {
        if let Some(index) = self.consts_table.iter().position(|x| *x == value) {
            index
        } else {
            let index = self.consts_table.len();
            self.consts_table.push(value);
            index
        }
    }
}
