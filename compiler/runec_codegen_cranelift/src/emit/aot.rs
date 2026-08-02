use cranelift_codegen::{isa::OwnedTargetIsa, settings};
use cranelift_object::{ObjectBuilder, ObjectModule};
use runec_mir::MirModule;
use runec_source::span::Span;

use crate::diagnostics::backend;
use crate::{CodegenOptions, CodegenResult, CraneliftLowerer};

/// Emits an object file from the same Cranelift IR path used by the JIT.
pub struct AotBackend;

impl AotBackend {
    pub fn emit_object(
        mir: &MirModule<'_>,
        name: &str,
        diagnostic_span: Span,
    ) -> CodegenResult<Vec<u8>> {
        let builder = ObjectBuilder::new(
            native_isa(diagnostic_span)?,
            name,
            cranelift_module::default_libcall_names(),
        )
        .map_err(|error| backend(error, diagnostic_span))?;
        let mut module = ObjectModule::new(builder);
        CraneliftLowerer::new(CodegenOptions::aot()).compile(&mut module, mir, diagnostic_span)?;
        module
            .finish()
            .emit()
            .map_err(|error| backend(error, diagnostic_span))
    }
}

fn native_isa(diagnostic_span: Span) -> CodegenResult<OwnedTargetIsa> {
    cranelift_native::builder()
        .map_err(|error| backend(error.to_string(), diagnostic_span))?
        .finish(settings::Flags::new(settings::builder()))
        .map_err(|error| backend(error, diagnostic_span))
}

#[cfg(test)]
mod tests {
    use super::AotBackend;
    use runec_hir::ids::HirId;
    use runec_mir::{MirBlock, MirFunction, MirModule, MirTerminator, MirTy};
    use runec_source::{byte_pos::BytePos, source_map::SourceId, span::Span};

    fn span() -> Span {
        Span::new(
            BytePos::from_usize(0),
            BytePos::from_usize(1),
            SourceId::from_usize(0),
        )
    }
    #[test]
    fn emits_object_from_shared_lowering() {
        let mut module = MirModule::new();
        let mut main = MirFunction::new(HirId::from_usize(0), "main", MirTy::Unit, span(), span());
        main.entry = main.push_block(MirBlock::new(MirTerminator::Return(None)));
        let main = module.push_function(main);
        module.entry = Some(main);
        assert!(
            !AotBackend::emit_object(&module, "runeway_test", span())
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn missing_entry_diagnostic_uses_fallback_span() {
        let diagnostic_span = span();
        let error = AotBackend::emit_object(&MirModule::new(), "runeway_test", diagnostic_span)
            .expect_err("module without an entry should fail codegen");

        assert_eq!(error.labels[0].span, diagnostic_span);
    }
}
