use crate::constant::MirConstant;
use crate::function::MirFunction;
use crate::ids::{MirConstantId, MirFunctionId};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct MirModule {
    pub constants: Vec<MirConstant>,
    pub functions: Vec<MirFunction>,
    pub entry: Option<MirFunctionId>,
}

impl MirModule {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_constant(&mut self, constant: MirConstant) -> MirConstantId {
        let id = MirConstantId::from_usize(self.constants.len());
        self.constants.push(constant);
        id
    }

    pub fn constant(&self, id: MirConstantId) -> &MirConstant {
        &self.constants[id.to_usize()]
    }

    pub fn push_function(&mut self, function: MirFunction) -> MirFunctionId {
        let id = MirFunctionId::from_usize(self.functions.len());
        self.functions.push(function);
        id
    }

    pub fn function(&self, id: MirFunctionId) -> &MirFunction {
        &self.functions[id.to_usize()]
    }
}