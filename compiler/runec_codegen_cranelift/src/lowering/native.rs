use std::collections::HashMap;

use cranelift_codegen::ir::{AbiParam, InstBuilder, Signature, UserFuncName, Value, types};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use cranelift_module::{DataDescription, DataId, FuncId, Linkage, Module};
use runec_abi::{RuntimeFunctionDecl, RuntimeFunctionId, runtime_function};
use runec_hir::ids::HirId;
use runec_mir::{
    MirCallee, MirConstant, MirFunction, MirFunctionId, MirImmediate, MirModule, MirOperand,
    MirRvalue, MirStmt, MirTerminator, MirTy,
};

use crate::{CodegenError, CodegenErrorKind, CodegenResult};

pub struct CompiledModule {
    pub entry: MirFunctionId,
    pub entry_func: FuncId,
}

/// Backend-neutral declaration and Cranelift IR generation used by `CraneliftLowerer`.
pub(super) fn compile_module<M: Module>(
    module: &mut M,
    mir: &MirModule<'_>,
) -> CodegenResult<CompiledModule> {
    let entry = mir
        .entry
        .ok_or_else(|| CodegenError::new(CodegenErrorKind::MissingEntry))?;
    let mut functions = HashMap::<HirId, FuncId>::new();
    for function in &mir.functions {
        let id = module
            .declare_function(
                function.name,
                Linkage::Export,
                &signature_for(module, function)?,
            )
            .map_err(backend)?;
        functions.insert(function.hir_id, id);
    }

    let mut runtimes = HashMap::new();
    for function in &mir.functions {
        for block in &function.blocks {
            for stmt in &block.stmts {
                let MirStmt::Assign {
                    rhs:
                        MirRvalue::Call {
                            callee: MirCallee::Runtime(id),
                            ..
                        },
                    ..
                } = stmt
                else {
                    continue;
                };
                if runtimes.contains_key(id) {
                    continue;
                }
                let decl = runtime_function(*id).ok_or_else(|| {
                    CodegenError::new(CodegenErrorKind::UnsupportedRuntimeFunction(*id))
                })?;
                let func = module
                    .declare_function(
                        decl.symbol,
                        Linkage::Import,
                        &runtime_signature(module, decl),
                    )
                    .map_err(backend)?;
                runtimes.insert(*id, func);
            }
        }
    }

    let constants = declare_constants(module, mir)?;
    for function in &mir.functions {
        compile_function(
            module,
            function,
            functions[&function.hir_id],
            &functions,
            &runtimes,
            &constants,
        )?;
    }
    Ok(CompiledModule {
        entry,
        entry_func: functions[&mir.function(entry).hir_id],
    })
}

fn compile_function<M: Module>(
    module: &mut M,
    function: &MirFunction<'_>,
    id: FuncId,
    functions: &HashMap<HirId, FuncId>,
    runtimes: &HashMap<RuntimeFunctionId, FuncId>,
    constants: &[(DataId, usize)],
) -> CodegenResult<()> {
    let mut context = module.make_context();
    context.func.signature = signature_for(module, function)?;
    context.func.name = UserFuncName::user(0, id.as_u32());
    let mut builder_context = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut context.func, &mut builder_context);
    let entry = builder.create_block();
    builder.append_block_params_for_function_params(entry);
    builder.switch_to_block(entry);
    builder.seal_block(entry);

    let mut locals = Vec::with_capacity(function.locals.len());
    for local in &function.locals {
        let mut local_vars = Vec::new();
        for ty in clif_types(module, local.ty)? {
            local_vars.push(builder.declare_var(ty));
        }
        locals.push(local_vars);
    }
    let params = builder.block_params(entry).to_vec();
    let mut offset = 0;
    for param in function.params.iter() {
        for var in &locals[param.to_usize()] {
            builder.def_var(*var, params[offset]);
            offset += 1;
        }
    }

    let block = function
        .blocks
        .get(function.entry.to_usize())
        .ok_or_else(|| {
            CodegenError::at(
                function.span,
                CodegenErrorKind::Backend("missing MIR entry block".into()),
            )
        })?;
    for stmt in &block.stmts {
        let MirStmt::Assign { dst, rhs, .. } = stmt;
        let values = match rhs {
            MirRvalue::Use(operand) => {
                lower_operand(&mut builder, module, operand, &locals, constants)?
            }
            MirRvalue::Call { callee, args } => {
                let func_id = match callee {
                    MirCallee::Runtime(id) => *runtimes.get(id).ok_or_else(|| {
                        CodegenError::new(CodegenErrorKind::UnsupportedRuntimeFunction(*id))
                    })?,
                    MirCallee::Function(id) => *functions.get(id).ok_or_else(|| {
                        CodegenError::new(CodegenErrorKind::Backend(format!(
                            "unknown function {id:?}"
                        )))
                    })?,
                };
                let func_ref = module.declare_func_in_func(func_id, builder.func);
                let mut call_args = Vec::new();
                for arg in args.iter() {
                    call_args.extend(lower_operand(
                        &mut builder,
                        module,
                        arg,
                        &locals,
                        constants,
                    )?);
                }
                let call = builder.ins().call(func_ref, &call_args);
                builder.inst_results(call).to_vec()
            }
        };
        let vars = &locals[dst.local.to_usize()];
        if vars.len() != values.len() {
            return Err(CodegenError::new(CodegenErrorKind::Backend(
                "assignment ABI arity mismatch".into(),
            )));
        }
        for (var, value) in vars.iter().zip(values) {
            builder.def_var(*var, value);
        }
    }
    match &block.terminator {
        MirTerminator::Return(None) => {
            builder.ins().return_(&[]);
        }
        MirTerminator::Return(Some(operand)) => {
            let values = lower_operand(&mut builder, module, operand, &locals, constants)?;
            builder.ins().return_(&values);
        }
    }
    builder.finalize();
    module.define_function(id, &mut context).map_err(backend)?;
    module.clear_context(&mut context);
    Ok(())
}

fn lower_operand<M: Module>(
    builder: &mut FunctionBuilder<'_>,
    module: &mut M,
    operand: &MirOperand,
    locals: &[Vec<Variable>],
    constants: &[(DataId, usize)],
) -> CodegenResult<Vec<Value>> {
    Ok(match operand {
        MirOperand::Copy(place) => locals[place.local.to_usize()]
            .iter()
            .map(|v| builder.use_var(*v))
            .collect(),
        MirOperand::Constant(id) => {
            let (data_id, len) = constants[id.to_usize()];
            let data = module.declare_data_in_func(data_id, builder.func);
            let pointer_ty = module.target_config().pointer_type();
            vec![
                builder.ins().global_value(pointer_ty, data),
                builder.ins().iconst(pointer_ty, len as i64),
            ]
        }
        MirOperand::Immediate(value) => match value {
            MirImmediate::Unit => vec![],
            MirImmediate::Bool(v) => vec![builder.ins().iconst(types::I8, i64::from(*v))],
            MirImmediate::Char(v) => vec![builder.ins().iconst(types::I32, *v as i64)],
            MirImmediate::Int { value, ty } => {
                vec![builder.ins().iconst(int_type(ty.bits), *value as i64)]
            }
            MirImmediate::Float { value, ty } => vec![match ty.bits {
                runec_builtins::TypeBits::B32 => builder.ins().f32const(*value as f32),
                _ => builder.ins().f64const(*value),
            }],
        },
    })
}

fn declare_constants<M: Module>(
    module: &mut M,
    mir: &MirModule<'_>,
) -> CodegenResult<Vec<(DataId, usize)>> {
    mir.constants
        .iter()
        .enumerate()
        .map(|(index, constant)| {
            let bytes: &[u8] = match constant {
                MirConstant::Str(v) => v.as_bytes(),
                MirConstant::Bytes(v) => v.as_ref(),
            };
            let id = module
                .declare_data(
                    &format!("__runeway_const_{index}"),
                    Linkage::Local,
                    false,
                    false,
                )
                .map_err(backend)?;
            let mut data = DataDescription::new();
            data.define(bytes.to_vec().into_boxed_slice());
            module.define_data(id, &data).map_err(backend)?;
            Ok((id, bytes.len()))
        })
        .collect()
}

fn signature_for<M: Module>(module: &M, function: &MirFunction<'_>) -> CodegenResult<Signature> {
    let mut signature = module.make_signature();
    for param in function.params.iter() {
        for ty in clif_types(module, function.locals[param.to_usize()].ty)? {
            signature.params.push(AbiParam::new(ty));
        }
    }
    for ty in clif_types(module, function.ret_ty)? {
        signature.returns.push(AbiParam::new(ty));
    }
    Ok(signature)
}

fn runtime_signature<M: Module>(module: &M, decl: &RuntimeFunctionDecl) -> Signature {
    let mut signature = module.make_signature();
    for ty in decl.params {
        signature.params.push(AbiParam::new(abi_type(module, *ty)));
    }
    if decl.ret != runec_abi::AbiType::Unit {
        signature
            .returns
            .push(AbiParam::new(abi_type(module, decl.ret)));
    }
    signature
}

fn clif_types<M: Module>(module: &M, ty: MirTy) -> CodegenResult<Vec<cranelift_codegen::ir::Type>> {
    Ok(match ty {
        MirTy::Unit => vec![],
        MirTy::Bool => vec![types::I8],
        MirTy::Char => vec![types::I32],
        MirTy::Int(v) => vec![int_type(v.bits)],
        MirTy::Float(v) => vec![match v.bits {
            runec_builtins::TypeBits::B32 => types::F32,
            runec_builtins::TypeBits::B64 => types::F64,
            _ => return Err(CodegenError::new(CodegenErrorKind::UnsupportedType(ty))),
        }],
        MirTy::Str | MirTy::Bytes => vec![module.target_config().pointer_type(); 2],
    })
}

fn int_type(bits: runec_builtins::TypeBits) -> cranelift_codegen::ir::Type {
    match bits {
        runec_builtins::TypeBits::B8 => types::I8,
        runec_builtins::TypeBits::B16 => types::I16,
        runec_builtins::TypeBits::B32 => types::I32,
        runec_builtins::TypeBits::B64 => types::I64,
        runec_builtins::TypeBits::B128 => types::I128,
    }
}
fn abi_type<M: Module>(module: &M, ty: runec_abi::AbiType) -> cranelift_codegen::ir::Type {
    match ty {
        runec_abi::AbiType::I8 => types::I8,
        runec_abi::AbiType::I16 => types::I16,
        runec_abi::AbiType::I32 => types::I32,
        runec_abi::AbiType::I64 => types::I64,
        runec_abi::AbiType::I128 => types::I128,
        runec_abi::AbiType::F32 => types::F32,
        runec_abi::AbiType::F64 => types::F64,
        runec_abi::AbiType::Pointer | runec_abi::AbiType::Usize => {
            module.target_config().pointer_type()
        }
        runec_abi::AbiType::Unit => types::I8,
    }
}
fn backend(error: impl std::fmt::Display) -> CodegenError {
    CodegenError::new(CodegenErrorKind::Backend(error.to_string()))
}
