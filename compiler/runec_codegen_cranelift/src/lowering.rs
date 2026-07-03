use std::collections::HashSet;

use runec_abi::{RuntimeFunctionId, runtime_function};
use runec_builtins::TypeBits;
use runec_mir::{MirCallee, MirFunction, MirFunctionId, MirModule, MirRvalue, MirTy};

use crate::error::{CodegenError, CodegenErrorKind, CodegenResult};
use crate::signature::{AbiType, FunctionSignature};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum EmitMode {
    Jit,
    Aot,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct CodegenOptions {
    pub mode: EmitMode,
}

impl CodegenOptions {
    pub const fn jit() -> Self {
        Self {
            mode: EmitMode::Jit,
        }
    }

    pub const fn aot() -> Self {
        Self {
            mode: EmitMode::Aot,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodegenArtifact {
    pub mode: EmitMode,
    pub entry: Option<MirFunctionId>,
    pub functions: Vec<LoweredFunction>,
    pub runtime_functions: Vec<LoweredRuntimeFunction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweredFunction {
    pub id: MirFunctionId,
    pub signature: FunctionSignature,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweredRuntimeFunction {
    pub id: RuntimeFunctionId,
    pub symbol: &'static str,
    pub signature: FunctionSignature,
}

pub struct CraneliftLowerer {
    options: CodegenOptions,
}

impl CraneliftLowerer {
    pub fn new(options: CodegenOptions) -> Self {
        Self { options }
    }

    pub fn lower_module(&mut self, module: &MirModule) -> CodegenResult<CodegenArtifact> {
        let functions = module
            .functions
            .iter()
            .enumerate()
            .map(|(idx, function)| {
                Ok(LoweredFunction {
                    id: MirFunctionId::from_usize(idx),
                    signature: self.lower_signature(function)?,
                })
            })
            .collect::<CodegenResult<Vec<_>>>()?;
        let runtime_functions = self.lower_runtime_functions(module)?;

        Ok(CodegenArtifact {
            mode: self.options.mode,
            entry: module.entry,
            functions,
            runtime_functions,
        })
    }

    fn lower_runtime_functions(
        &self,
        module: &MirModule,
    ) -> CodegenResult<Vec<LoweredRuntimeFunction>> {
        let mut seen = HashSet::new();
        let mut functions = Vec::new();

        for function in &module.functions {
            for block in &function.blocks {
                for stmt in &block.stmts {
                    let runec_mir::MirStmt::Assign { rhs, .. } = stmt;
                    let MirRvalue::Call {
                        callee: MirCallee::Runtime(id),
                        ..
                    } = rhs
                    else {
                        continue;
                    };

                    if seen.insert(*id) {
                        functions.push(self.lower_runtime_function(*id)?);
                    }
                }
            }
        }

        Ok(functions)
    }

    fn lower_runtime_function(
        &self,
        id: RuntimeFunctionId,
    ) -> CodegenResult<LoweredRuntimeFunction> {
        let declaration = runtime_function(id)
            .ok_or_else(|| CodegenError::new(CodegenErrorKind::UnsupportedRuntimeFunction(id)))?;
        let returns = match declaration.ret {
            AbiType::Unit => Vec::new(),
            ty => vec![ty],
        };

        Ok(LoweredRuntimeFunction {
            id,
            symbol: declaration.symbol,
            signature: FunctionSignature::new(declaration.params, returns),
        })
    }

    fn lower_signature(&self, function: &MirFunction) -> CodegenResult<FunctionSignature> {
        let mut params = Vec::new();
        for param in &function.params {
            self.lower_type(function.locals[param.to_usize()].ty, &mut params)?;
        }

        let mut returns = Vec::new();
        self.lower_type(function.ret_ty, &mut returns)?;

        Ok(FunctionSignature::new(params, returns))
    }

    fn lower_type(&self, ty: MirTy, output: &mut Vec<AbiType>) -> CodegenResult<()> {
        match ty {
            MirTy::Unit => {}
            MirTy::Bool => output.push(AbiType::I8),
            MirTy::Int(int) => output.push(match int.bits {
                TypeBits::B8 => AbiType::I8,
                TypeBits::B16 => AbiType::I16,
                TypeBits::B32 => AbiType::I32,
                TypeBits::B64 => AbiType::I64,
                TypeBits::B128 => AbiType::I128,
            }),
            MirTy::Float(float) => output.push(match float.bits {
                TypeBits::B32 => AbiType::F32,
                TypeBits::B64 => AbiType::F64,
                _ => return Err(CodegenError::new(CodegenErrorKind::UnsupportedType(ty))),
            }),
            MirTy::Char => output.push(AbiType::I32),
            MirTy::Str | MirTy::Bytes => {
                output.push(AbiType::Pointer);
                output.push(AbiType::Usize);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use runec_abi::RUNTIME_PRINT;
    use runec_mir::{
        MirBlock, MirCallee, MirFunction, MirModule, MirOperand, MirPlace, MirRvalue, MirStmt,
        MirTerminator, MirTy,
    };

    use super::{AbiType, CodegenOptions, CraneliftLowerer, EmitMode};

    #[test]
    fn preserves_emit_mode_and_entry() {
        let mut module = MirModule::new();
        let main = module.push_function(MirFunction::new(
            runec_hir::ids::HirId::from_usize(0),
            "main",
            MirTy::Unit,
        ));
        module.entry = Some(main);

        let mut lowerer = CraneliftLowerer::new(CodegenOptions::jit());
        let artifact = lowerer
            .lower_module(&module)
            .expect("codegen skeleton should accept module");

        assert_eq!(artifact.mode, EmitMode::Jit);
        assert_eq!(artifact.entry, Some(main));
        assert_eq!(artifact.functions.len(), 1);
    }

    #[test]
    fn lowers_string_parameter_to_pointer_and_length() {
        let mut module = MirModule::new();
        let mut function = MirFunction::new(
            runec_hir::ids::HirId::from_usize(0),
            "print_message",
            MirTy::Unit,
        );
        let message = function.push_local(MirTy::Str);
        function.params = Box::new([message]);
        module.push_function(function);

        let artifact = CraneliftLowerer::new(CodegenOptions::jit())
            .lower_module(&module)
            .expect("string parameters should have a supported ABI shape");

        assert_eq!(
            artifact.functions[0].signature.params,
            [AbiType::Pointer, AbiType::Usize]
        );
        assert!(artifact.functions[0].signature.returns.is_empty());
    }

    #[test]
    fn collects_runtime_function_declarations() {
        let mut module = MirModule::new();
        let mut function =
            MirFunction::new(runec_hir::ids::HirId::from_usize(0), "main", MirTy::Unit);
        let message = function.push_local(MirTy::Str);
        let result = function.push_local(MirTy::Unit);
        let mut entry = MirBlock::new(MirTerminator::Return(None));
        entry.stmts.push(MirStmt::Assign {
            dst: MirPlace::new(result),
            rhs: MirRvalue::Call {
                callee: MirCallee::Runtime(RUNTIME_PRINT),
                args: Box::new([MirOperand::Copy(MirPlace::new(message))]),
            },
        });
        function.push_block(entry);
        module.push_function(function);

        let artifact = CraneliftLowerer::new(CodegenOptions::jit())
            .lower_module(&module)
            .expect("runtime calls should have declarations");

        assert_eq!(artifact.runtime_functions.len(), 1);
        assert_eq!(artifact.runtime_functions[0].id, RUNTIME_PRINT);
        assert_eq!(artifact.runtime_functions[0].symbol, "__runeway_print");
        assert_eq!(
            artifact.runtime_functions[0].signature.params,
            [AbiType::Pointer, AbiType::Usize]
        );
        assert!(artifact.runtime_functions[0].signature.returns.is_empty());
    }
}
