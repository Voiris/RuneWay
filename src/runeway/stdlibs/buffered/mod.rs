mod general;

use std::rc::Rc;
use crate::runeway::builtins::types::RNWString;
use crate::runeway::executor::runtime::environment::{EnvRef, Environment};
use crate::runeway::executor::runtime::libraries::register_stdlib;
use crate::runeway::stdlibs::buffered::general::{
    register_native_buffered_flush, register_native_buffered_print, register_native_buffered_println};

fn load() -> EnvRef {
    let env = Environment::new_global();

    let mut borrow = env.borrow_mut();

    borrow.define_variable("VERSION".to_owned(), RNWString::new("v0.0.1".to_owned()));
    borrow.define_function(Rc::new(register_native_buffered_print()));
    borrow.define_function(Rc::new(register_native_buffered_println()));
    borrow.define_function(Rc::new(register_native_buffered_flush()));

    drop(borrow);

    env
}

pub(super) fn register() {
    register_stdlib("buffered", load)
}
