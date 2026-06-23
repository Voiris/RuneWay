use crate::constant::MirConstant;
use crate::function::MirFunction;
use crate::ids::{MirConstantId, MirFunctionId};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct MirModule<'src> {
    pub constants: Vec<MirConstant<'src>>,
    pub functions: Vec<MirFunction>,
    pub entry: Option<MirFunctionId>,
}

impl<'src> MirModule<'src> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_constant(&mut self, constant: MirConstant<'src>) -> MirConstantId {
        let id = MirConstantId::from_usize(self.constants.len());
        self.constants.push(constant);
        id
    }

    pub fn constant(&self, id: MirConstantId) -> &MirConstant<'src> {
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

#[cfg(test)]
mod tests {
    use runec_abi::RUNTIME_PRINT;
    use runec_hir::ids::HirId;

    use crate::block::{MirBlock, MirRvalue, MirStmt, MirTerminator};
    use crate::constant::MirConstant;
    use crate::function::{MirCallee, MirFunction};
    use crate::module::MirModule;
    use crate::operand::{MirOperand, MirPlace};
    use crate::ty::MirTy;

    #[test]
    fn stores_main_with_runtime_print_call() {
        let mut module = MirModule::new();
        let hello = module.push_constant(MirConstant::Str("Hello, World".into()));

        let mut main = MirFunction::new(HirId::from_usize(0), "main", MirTy::Unit);
        let message = main.push_local(MirTy::Str);
        let print_result = main.push_local(MirTy::Unit);

        let mut entry = MirBlock::new(MirTerminator::Return);
        entry.stmts.push(MirStmt::Assign {
            dst: MirPlace::new(message),
            rhs: MirRvalue::Use(MirOperand::Constant(hello)),
        });
        entry.stmts.push(MirStmt::Assign {
            dst: MirPlace::new(print_result),
            rhs: MirRvalue::Call {
                callee: MirCallee::Runtime(RUNTIME_PRINT),
                args: Box::new([MirOperand::Copy(MirPlace::new(message))]),
            },
        });
        main.entry = main.push_block(entry);

        let main_id = module.push_function(main);
        module.entry = Some(main_id);

        assert_eq!(module.constant(hello).ty(), MirTy::Str);
        assert_eq!(module.function(main_id).name.as_ref(), "main");
        assert_eq!(module.function(main_id).blocks[0].stmts.len(), 2);
        assert_eq!(module.entry, Some(main_id));
    }
}
