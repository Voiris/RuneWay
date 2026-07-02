use runec_abi::RuntimeFunctionId;
use runec_builtins::TypeBits;
use runec_mir::{MirFunction, MirFunctionId, MirModule, MirTy};

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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct LoweredRuntimeFunction {
    pub id: RuntimeFunctionId,
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

        Ok(CodegenArtifact {
            mode: self.options.mode,
            entry: module.entry,
            functions,
            runtime_functions: Vec::new(),
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
                output.push(AbiType::Pointer);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use runec_mir::{MirFunction, MirModule, MirTy};

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
            [AbiType::Pointer, AbiType::Pointer]
        );
        assert!(artifact.functions[0].signature.returns.is_empty());
    }
}
