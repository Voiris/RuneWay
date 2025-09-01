use crate::register_types;
use crate::runeway::builtins::types::RNWString;
use crate::runeway::runtime::environment::{EnvRef, Environment};
use crate::runeway::runtime::libraries::register_stdlib;
use crate::runeway::stdlibs::dynbox::types::RNWBox;

mod types;

fn load() -> EnvRef {
    let env = Environment::new_builtins_global();

    let mut borrow = env.borrow_mut();

    register_types!(
        borrow;
        RNWBox => types::register()
    );

    borrow.define_variable("VERSION".to_owned(), RNWString::new("v0.0.1".to_owned()));

    drop(borrow);

    env
}

pub(super) fn register() {
    register_stdlib("dynbox", load)
}
