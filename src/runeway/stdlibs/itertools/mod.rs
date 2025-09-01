mod general;

use crate::runeway::builtins::types::RNWString;
use crate::runeway::runtime::environment::{EnvRef, Environment};
use crate::runeway::runtime::libraries::register_stdlib;

fn load() -> EnvRef {
    let env = Environment::new_builtins_global();

    let mut borrow = env.borrow_mut();

    borrow.define_variable("VERSION".to_owned(), RNWString::new("v0.0.1".to_owned()));
    borrow.define_variable("any".to_string(), general::register_native_itertools_any());
    borrow.define_variable("all".to_string(), general::register_native_itertools_all());
    borrow.define_variable(
        "iter_equal".to_string(),
        general::register_native_itertools_iter_equal(),
    );

    drop(borrow);

    env
}

pub(super) fn register() {
    register_stdlib("itertools", load)
}
