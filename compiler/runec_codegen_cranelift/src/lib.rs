pub mod diagnostics;
pub mod emit;
pub mod lowering;
pub mod signature;

pub use diagnostics::CodegenResult;
pub use emit::{AotBackend, JitBackend};
pub use lowering::{
    CodegenArtifact, CodegenOptions, CraneliftLowerer, EmitMode, LoweredFunction,
    LoweredRuntimeFunction,
};
pub use signature::{AbiType, FunctionSignature};

/// Native symbol used by the platform entry-point wrapper.
pub const ENTRY_SYMBOL: &str = "__runeway_main";
