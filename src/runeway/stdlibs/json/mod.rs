mod errors;
mod general;

use crate::runeway::builtins::types::RNWString;
use crate::runeway::runtime::environment::{EnvRef, Environment};
use crate::runeway::runtime::libraries::register_stdlib;
use crate::runeway::stdlibs::json::general::{
    register_native_json_dump, register_native_json_load,
};

fn load() -> EnvRef {
    let env = Environment::new_builtins_global();

    let mut borrow = env.borrow_mut();

    borrow.define_variable("VERSION".to_owned(), RNWString::new("v0.0.1".to_owned()));
    borrow.define_variable("load".to_string(), register_native_json_load());
    borrow.define_variable("dump".to_string(), register_native_json_dump());

    drop(borrow);

    env
}

pub(super) fn register() {
    register_stdlib("json", load)
}
