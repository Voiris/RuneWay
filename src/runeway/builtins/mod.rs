use std::rc::Rc;
use crate::runeway::builtins::types::register_basic_types;
use crate::runeway::executor::runtime::environment::EnvRef;

pub mod types;
pub mod functions;

pub fn prelude(env: EnvRef) {
    register_basic_types();

    let mut borrow = env.borrow_mut();
    borrow.define_function(Rc::new(functions::print::register_native_print()));
    borrow.define_function(Rc::new(functions::print::register_native_println()));
}
