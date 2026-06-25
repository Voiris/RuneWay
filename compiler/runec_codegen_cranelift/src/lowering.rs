use runec_abi::RuntimeFunctionId;
use runec_mir::{MirFunctionId, MirModule};

use crate::error::CodegenResult;

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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct LoweredFunction {
    pub id: MirFunctionId,
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
            .map(|(idx, _)| LoweredFunction {
                id: MirFunctionId::from_usize(idx),
            })
            .collect();

        Ok(CodegenArtifact {
            mode: self.options.mode,
            entry: module.entry,
            functions,
            runtime_functions: Vec::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use runec_mir::{MirFunction, MirModule, MirTy};

    use super::{CodegenOptions, CraneliftLowerer, EmitMode};

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
}
