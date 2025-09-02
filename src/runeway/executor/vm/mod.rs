use std::cell::RefCell;
use std::rc::Rc;
use crate::runeway::builtins;
use crate::runeway::builtins::types::{RNWBoolean, RNWFloat, RNWInteger, RNWNullType, RNWString, RNWUnsignedInteger};
use crate::runeway::compiler::bytecode::interface::application::CompiledApplication;
use crate::runeway::compiler::bytecode::interface::consts::{ConstValue, ConstsTable};
use crate::runeway::compiler::bytecode::interface::function::CompiledFunction;
use crate::runeway::compiler::bytecode::interface::module::{CompiledModule, UserDefinedStatement};
use crate::runeway::compiler::bytecode::interface::opcode::Opcode;
use crate::runeway::core::errors::RWResult;
use crate::runeway::runtime::environment::{EnvRef, Environment};
use crate::runeway::runtime::types::{RNWFunction, RNWObjectRef, RNWRegisteredNativeFunction};

#[derive(Debug)]
pub struct VM {
    modules_envs: Vec<EnvRef>,
    entry: (usize, String),
    consts_table: ConstsTable,
    stack: Vec<RNWObjectRef>
}

impl VM {
    pub fn new(app: CompiledApplication) -> VMRef {
        let mut vm = Rc::new(RefCell::new(Self {
            modules_envs: Vec::new(),
            entry: (app.entry_module, app.entry_function),
            consts_table: app.consts_table,
            stack: Vec::new()
        }));

        let global_env = Self::load_builtins();

        for module in app.modules {
            let module_env = Self::load_module(&vm, &global_env, module);
            let mut borrow = vm.borrow_mut();
            borrow.modules_envs.push(module_env);
        }

        vm
    }

    pub fn run(vm: &VMRef) -> RWResult<()> {
        let function_obj = {
            let mut vm = vm.borrow_mut();

            vm.modules_envs[vm.entry.0].borrow().get_variable(vm.entry.1.clone()).unwrap().borrow().as_any().downcast_ref::<RNWFunction>().cloned().unwrap()
        };

        function_obj.function.call(&[]).map(|_| ())
    }

    fn load_builtins() -> EnvRef {
        let global_env = Environment::new_builtins_global();

        builtins::prelude(global_env.clone());

        global_env
    }

    fn load_module(vm: &VMRef, global_env: &EnvRef, module: CompiledModule) -> EnvRef {
        match module {
            CompiledModule::UserDefined(statements) => {
                let module_env = Environment::new_global(global_env.clone());

                for statement in statements {
                    match statement {
                        UserDefinedStatement::Function(name, compiled_function) => {
                            let function = Self::load_function(vm, name, compiled_function, module_env.clone());
                            let mut borrow = module_env.borrow_mut();
                            borrow.define_function(function)
                        }
                    }
                }

                module_env
            },
            _ => unimplemented!(),
        }
    }

    fn load_function(vm: &VMRef, name: String, compiled_function: CompiledFunction, env: EnvRef) -> Rc<RNWRegisteredNativeFunction> {
        let mut params = Vec::new();

        for _ in 0..compiled_function.parameters.len() {
            params.push(0)
        }

        let _vm = vm.clone();

        let function = Rc::new(move |_args: &[RNWObjectRef]| {
            Self::execute_function(
                _vm.clone(),
                compiled_function.parameters.clone(),
                compiled_function.ops.clone(),
                env.clone()
            );
            Ok(RNWNullType::new())
        });

        RNWRegisteredNativeFunction::new(name, function, params)
    }

    fn execute_function(vm: VMRef, params: Vec<String>, ops: Vec<Opcode>, env: EnvRef) {
        Self::execute_ops(
            &vm,
            ops,
            env,
        )
    }

    fn execute_ops(vm: &VMRef, ops: Vec<Opcode>, env: EnvRef) {
        for opcode in ops {
            Self::execute_opcode(vm, opcode, &env);
        }
    }

    fn execute_opcode(vm: &VMRef, opcode: Opcode, env: &EnvRef) {
        match opcode {
            Opcode::LoadConst(id) => {
                let mut borrow = vm.borrow_mut();
                let const_value: ConstValue = borrow.consts_table.get(id).cloned().unwrap();
                let object = match const_value {
                    ConstValue::Str(s) => {
                        RNWString::new(s)
                    }
                };
                borrow.stack.push(object);
            }
            Opcode::PushInt(i) => {
                let mut borrow = vm.borrow_mut();
                borrow.stack.push(RNWInteger::new(i));
            }
            Opcode::PushUnsignedInt(u) => {
                let mut borrow = vm.borrow_mut();
                borrow.stack.push(RNWUnsignedInteger::new(u));
            }
            Opcode::PushFloat(f) => {
                let mut borrow = vm.borrow_mut();
                borrow.stack.push(RNWFloat::new(f));
            }
            Opcode::PushTrue => {
                let mut borrow = vm.borrow_mut();
                borrow.stack.push(RNWBoolean::new(true));
            }
            Opcode::PushFalse => {
                let mut borrow = vm.borrow_mut();
                borrow.stack.push(RNWBoolean::new(false));
            }
            Opcode::PushNull => {
                let mut borrow = vm.borrow_mut();
                borrow.stack.push(RNWNullType::new());
            }
            Opcode::LoadFast(name) => {
                let object = {
                    let borrow = env.borrow();
                    borrow.get_variable(name).unwrap()
                };
                let mut borrow = vm.borrow_mut();
                borrow.stack.push(object);
            }
            Opcode::DefineFast(name) => {
                let value = {
                    let mut borrow = vm.borrow_mut();
                    borrow.stack.pop().unwrap()
                };
                let mut borrow = env.borrow_mut();
                borrow.define_variable(name, value);
            }
            Opcode::Call(args_count) => {
                let (function, args) = {
                    let mut borrow = vm.borrow_mut();
                    let function = borrow.stack.pop().unwrap();

                    let mut args = Vec::with_capacity(args_count);
                    for _ in 0..args_count {
                        args.push(borrow.stack.pop().unwrap());
                    }

                    (function, args)
                };

                function.borrow().as_any().downcast_ref::<RNWFunction>().unwrap().function.call(args.as_slice()).unwrap();
            }
            _ => panic!("not implemented: {:?}", opcode)
        }
    }
}

pub type VMRef = Rc<RefCell<VM>>;
