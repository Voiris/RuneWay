mod general;
mod errors;

use crate::runeway::builtins::types::RNWString;
use crate::runeway::runtime::environment::{EnvRef, Environment};
use crate::runeway::runtime::libraries::register_stdlib;
use crate::runeway::stdlibs::http::general::{register_native_http_get, register_native_http_post};

fn load() -> EnvRef {
    let env = Environment::new_global();

    let mut borrow = env.borrow_mut();

    borrow.define_variable("VERSION".to_owned(), RNWString::new("v0.0.1".to_owned()));
    borrow.define_variable("get_".to_string(), register_native_http_get());
    borrow.define_variable("post".to_string(), register_native_http_post());

    drop(borrow);

    env
}

pub(super) fn register() {
    register_stdlib("http", load)
}
