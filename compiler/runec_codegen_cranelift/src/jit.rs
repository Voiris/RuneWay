use cranelift_codegen::{isa::OwnedTargetIsa, settings};
use cranelift_jit::{JITBuilder, JITModule};
use runec_mir::{MirModule, MirTy};

use crate::diagnostics::{backend, error, messages};
use crate::{CodegenOptions, CodegenResult, CraneliftLowerer};

/// Finalizes shared Cranelift IR in memory and invokes its entry point.
pub struct JitBackend {
    module: JITModule,
}

impl JitBackend {
    pub fn new(
        symbols: impl IntoIterator<Item = (&'static str, *const u8)>,
    ) -> CodegenResult<Self> {
        let mut builder =
            JITBuilder::with_isa(native_isa()?, cranelift_module::default_libcall_names());
        for (name, address) in symbols {
            builder.symbol(name, address);
        }
        Ok(Self {
            module: JITModule::new(builder),
        })
    }

    pub fn run(&mut self, mir: &MirModule<'_>) -> CodegenResult<()> {
        let compiled =
            CraneliftLowerer::new(CodegenOptions::jit()).compile(&mut self.module, mir)?;
        self.module.finalize_definitions().map_err(backend)?;
        let function = mir.function(compiled.entry);
        if !function.params.is_empty() || function.ret_ty != MirTy::Unit {
            let function_id = format!("{:?}", compiled.entry);
            return Err(error(
                messages::UNSUPPORTED_FUNCTION,
                &[("function", &function_id)],
                Some(function.span),
            ));
        }
        let address = self.module.get_finalized_function(compiled.entry_func);
        // SAFETY: the entry signature is checked above and finalized by JITModule.
        let entry: unsafe extern "C" fn() = unsafe { std::mem::transmute(address) };
        unsafe { entry() };
        Ok(())
    }
}

fn native_isa() -> CodegenResult<OwnedTargetIsa> {
    cranelift_native::builder()
        .map_err(|error| backend(error.to_string()))?
        .finish(settings::Flags::new(settings::builder()))
        .map_err(backend)
}

#[cfg(test)]
mod tests {
    use super::JitBackend;
    use runec_abi::RUNTIME_PRINTLN;
    use runec_hir::ids::HirId;
    use runec_mir::{
        MirBlock, MirCallee, MirConstant, MirFunction, MirModule, MirOperand, MirPlace, MirRvalue,
        MirStmt, MirTerminator, MirTy,
    };
    use runec_source::{byte_pos::BytePos, source_map::SourceId, span::Span};
    use std::sync::atomic::{AtomicBool, Ordering};

    static CALLED: AtomicBool = AtomicBool::new(false);
    unsafe extern "C" fn test_println(ptr: *const u8, len: usize) {
        let bytes = unsafe { std::slice::from_raw_parts(ptr, len) };
        assert_eq!(bytes, b"Hello, World!");
        CALLED.store(true, Ordering::SeqCst);
    }
    fn span() -> Span {
        Span::new(
            BytePos::from_usize(0),
            BytePos::from_usize(1),
            SourceId::from_usize(0),
        )
    }
    fn hello_module() -> MirModule<'static> {
        let mut module = MirModule::new();
        let hello = module.push_constant(MirConstant::Str("Hello, World!".into()));
        let mut main = MirFunction::new(HirId::from_usize(0), "main", MirTy::Unit, span(), span());
        let result = main.push_local(None, MirTy::Unit, span());
        let mut entry = MirBlock::new(MirTerminator::Return(None));
        entry.stmts.push(MirStmt::Assign {
            dst: MirPlace::new(result),
            rhs: MirRvalue::Call {
                callee: MirCallee::Runtime(RUNTIME_PRINTLN),
                args: Box::new([MirOperand::Constant(hello)]),
            },
            span: span(),
        });
        main.entry = main.push_block(entry);
        let main = module.push_function(main);
        module.entry = Some(main);
        module
    }
    #[test]
    fn executes_shared_lowering() {
        CALLED.store(false, Ordering::SeqCst);
        let mut backend =
            JitBackend::new([("__runeway_println", test_println as *const u8)]).unwrap();
        backend.run(&hello_module()).unwrap();
        assert!(CALLED.load(Ordering::SeqCst));
    }
}
