use crate::ids::{MirConstantId, MirLocalId};
use crate::ty::{MirFloatTy, MirIntTy};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct MirPlace {
    pub local: MirLocalId,
}

impl MirPlace {
    pub fn new(local: MirLocalId) -> Self {
        Self { local }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MirOperand {
    Copy(MirPlace),
    Immediate(MirImmediate),
    Constant(MirConstantId),
}

#[derive(Debug, Clone, PartialEq)]
pub enum MirImmediate {
    Unit,
    Bool(bool),
    Int { value: u128, ty: MirIntTy },
    Float { value: f64, ty: MirFloatTy },
    Char(char),
}
