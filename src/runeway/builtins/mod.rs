use crate::runeway::builtins::types::register_basic_types;
use crate::runeway::runtime::environment::EnvRef;

pub mod functions;
pub mod types;

pub fn prelude(env: EnvRef) {
    let mut borrow = env.borrow_mut();

    register_basic_types(&mut borrow);

    borrow.define_function(functions::print::register_native_print());
    borrow.define_function(functions::print::register_native_println());
    borrow.define_function(functions::other::register_native_type_cast());
    borrow.define_function(functions::other::register_native_object_id());
    borrow.define_function(functions::other::register_native_is_instance());
}
