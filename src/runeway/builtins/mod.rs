use std::rc::Rc;
use crate::runeway::builtins::types::register_basic_types;
use crate::runeway::runtime::environment::EnvRef;

pub mod types;
pub mod functions;

pub fn prelude(env: EnvRef) {
    let mut borrow = env.borrow_mut();

    register_basic_types(&mut borrow);

    borrow.define_function(functions::print::register_native_print());
    borrow.define_function(functions::print::register_native_println());
    borrow.define_function(functions::types::register_native_type_cast());
}
