use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use super::consts::{ConstValue, ConstsTable};
use super::opcode::Opcode;

#[derive(Serialize, Deserialize)]
pub struct CompiledApplication {
    entry_module: String,
    entry_function: String,
    consts_table: ConstsTable,
    modules: HashMap<String, Vec<Opcode>>,
}

impl CompiledApplication {
    pub fn new(entry_module: impl ToString, entry_function: impl ToString) -> Self {
        Self {
            entry_module: entry_module.to_string(),
            entry_function: entry_function.to_string(),
            consts_table: ConstsTable::new(),
            modules: HashMap::new(),
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
