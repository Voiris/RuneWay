mod errors;
mod general;

use crate::runeway::builtins::types::RNWString;
use crate::runeway::runtime::environment::{EnvRef, Environment};
use crate::runeway::runtime::libraries::register_stdlib;
use crate::runeway::stdlibs::buffered::general::{
    register_native_buffered_flush, register_native_buffered_print,
    register_native_buffered_println,
};

fn load() -> EnvRef {
    let env = Environment::new_global();

    let mut borrow = env.borrow_mut();

    borrow.define_variable("VERSION".to_owned(), RNWString::new("v0.0.1".to_owned()));
    borrow.define_variable("print".to_string(), register_native_buffered_print());
    borrow.define_variable("println".to_string(), register_native_buffered_println());
    borrow.define_variable("flush".to_string(), register_native_buffered_flush());

    drop(borrow);

    env
}

pub(super) fn register() {
    register_stdlib("buffered", load)
}
