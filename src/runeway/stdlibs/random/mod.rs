use crate::register_types;
use crate::runeway::builtins::types::RNWString;
use crate::runeway::runtime::environment::{EnvRef, Environment};
use crate::runeway::runtime::libraries::register_stdlib;
use crate::runeway::stdlibs::random::types::RNWRandomNumberGenerator;

mod general;
mod types;

fn load() -> EnvRef {
    let env = Environment::new_global();

    let mut borrow = env.borrow_mut();

    register_types!(
        borrow;
        RNWRandomNumberGenerator => types::register()
    );

    borrow.define_variable("VERSION".to_owned(), RNWString::new("v0.0.1".to_owned()));
    borrow.define_variable(
        "positive".to_string(),
        general::register_native_random_positive(),
    );
    borrow.define_variable(
        "negative".to_string(),
        general::register_native_random_negative(),
    );
    borrow.define_variable(
        "rand_int".to_string(),
        general::register_native_random_random_int(),
    );
    borrow.define_variable("unit".to_string(), general::register_native_random_unit());
    borrow.define_variable(
        "rand_bool".to_string(),
        general::register_native_random_random_bool(),
    );
    borrow.define_variable(
        "rand_range".to_string(),
        general::register_native_random_random_range(),
    );

    drop(borrow);

    env
}

pub(super) fn register() {
    register_stdlib("random", load)
}
