use std::collections::HashMap;
use std::fs;
use std::ops::Add;
use std::path::PathBuf;
use crate::runeway::compiler::bytecode::interface::application::CompiledApplication;
use crate::runeway::compiler::bytecode::interface::consts::ConstValue;
use crate::runeway::compiler::bytecode::interface::function::CompiledFunction;
use crate::runeway::compiler::bytecode::interface::module::{CompiledModule, ModuleCompileData, UserDefinedStatement};
use crate::runeway::compiler::bytecode::interface::opcode::Opcode;
use crate::runeway::core::ast::expression::{Expr, SpannedExpr};
use crate::runeway::core::ast::statement::{SpannedStatement, Statement};
use crate::runeway::core::errors::{RWResult, RuneWayError, RuneWayErrorKind};
use crate::runeway::core::parser::parse_code;

pub mod interface;

pub struct BytecodeCompiler {
    working_dir: PathBuf,
    application: CompiledApplication,
    modules_ids: HashMap<String, usize>,
    entry_module_name: String
}

impl BytecodeCompiler {
    pub fn compile(working_dir: PathBuf, entry_module: impl ToString, entry_function: impl ToString) -> RWResult<CompiledApplication> {
        let mut compiler = BytecodeCompiler {
            working_dir,
            application: CompiledApplication::new(0, entry_function),
            modules_ids: HashMap::new(),
            entry_module_name: entry_module.to_string()
        };

        compiler._compile()?;

        Ok(compiler.application)
    }

    fn _compile(&mut self) -> RWResult<()> {
        self.application.entry_module = self.compile_module(self.entry_module_name.clone())?;

        Ok(())
    }

    fn compile_module(&mut self, module: impl ToString) -> RWResult<usize> {
        let module = module.to_string();

        let module_data = self.load_module(&module)?;

        let id = match module_data {
            ModuleCompileData::AlreadyLoaded => self.modules_ids.get(&module).unwrap().clone(),
            ModuleCompileData::Standard(name) => {
                self.insert_module(name, CompiledModule::Standard)
            },
            ModuleCompileData::UserDefined {
                path, filename, code
            } => {
                let parsed_code = parse_code(filename.clone(), code.clone()).map_err(
                    |e| e.with_source(filename, code)
                )?;

                let mut user_defined_statements = Vec::new();

                for statement in parsed_code.ast {
                    match &statement.node {
                        Statement::Act { name, parameters, body, .. } => {
                            let function = CompiledFunction {
                                parameters: parameters.iter().cloned().map(|v| v.name).collect(),
                                ops: self.compile_statements(body.iter().cloned().map(|b| *b).collect())?,
                            };
                            user_defined_statements.push(UserDefinedStatement::Function(name.clone(), function));
                        }
                        _ => unimplemented!()
                    }
                }

                self.insert_module(path.clone(), CompiledModule::UserDefined(user_defined_statements))
            }
        };

        Ok(id)
    }

    fn insert_module(&mut self, module_name: String, compiled_module: CompiledModule) -> usize {
        let id = if let Some(id) = self.modules_ids.get(&module_name) {
            id.clone()
        } else {
            let id = self.application.modules.len();
            self.modules_ids.insert(module_name, id);
            id
        };
        self.application.modules.insert(id, compiled_module);

        id
    }

    fn compile_statements(&mut self, statements: Vec<SpannedStatement>) -> RWResult<Vec<Opcode>> {
        let mut ops = Vec::new();

        for stmt in statements {
            ops.extend(self.compile_statement(stmt)?)
        }

        Ok(ops)
    }

    fn compile_statement(&mut self, statement: SpannedStatement) -> RWResult<Vec<Opcode>> {
        let mut ops = Vec::new();

        match statement.node {
            Statement::Expr(expr) => {
                ops.extend(self.compile_expr(expr)?)
            }
            _ => unimplemented!()
        }

        Ok(ops)
    }

    fn compile_expr(&mut self, expr: SpannedExpr) -> RWResult<Vec<Opcode>> {
        let mut ops = Vec::new();

        match expr.node {
            Expr::String(s) => {
                ops.push(
                    Opcode::LoadConst(
                        self.application.add_const(
                            ConstValue::Str(s)
                        )
                    )
                )
            },
            Expr::Variable(v) => {
                ops.push(Opcode::LoadFast(v))
            }
            Expr::Call { callee, arguments } => {
                let args_count = arguments.len();

                for arg_expr in arguments {
                    ops.extend(self.compile_expr(arg_expr)?)
                }

                ops.extend(self.compile_expr(*callee)?);

                ops.push(Opcode::Call(args_count))
            },
            _ => {
                panic!("not implemented: {:?}", expr.node);
            }
        }

        Ok(ops)
    }

    //noinspection DuplicatedCode
    fn load_module(&self, path: &String) -> RWResult<ModuleCompileData> {
        if let Some(name) = path.strip_prefix("std::") {
            let name = name.to_owned();
            if self.modules_ids.contains_key(&name) {
                Ok(ModuleCompileData::AlreadyLoaded)
            } else {
                Ok(ModuleCompileData::Standard(name.to_owned()))
            }
        } else {
            let path = if !path.ends_with(".rnw") {
                format!("{}.rnw", path)
            } else {
                path.to_owned()
            };
            let mut path = PathBuf::from(&path);
            if !path.is_absolute() {
                path = self.working_dir.join(path);
            }
            if !path.is_file() {
                return Err(
                    RuneWayError::new(RuneWayErrorKind::error_with_code("FileSystemError"))
                        .with_message(format!(
                            "Path is not a file or it does not exists: {}",
                            path.display()
                        )),
                );
            }
            path = path.canonicalize().map_err(|err| {
                RuneWayError::new(RuneWayErrorKind::error_with_code("FileSystemError"))
                    .with_message(format!("Cannot canonicalize path: {}", path.display()))
                    .with_note(format!("Raw Error: {}", err))
            })?;

            let str_path = path.to_str().unwrap().to_owned();

            if self.modules_ids.contains_key(&str_path) {
                Ok(ModuleCompileData::AlreadyLoaded)
            } else {
                let (filename, code) = if let Some(file_name) = path.file_name() {
                    (file_name.to_str().unwrap().to_owned(), fs::read_to_string(path.clone())?)
                } else {
                    panic!("Internal: No filename found.");
                };

                Ok(ModuleCompileData::UserDefined { path: str_path, filename, code })
            }
        }
    }
}
