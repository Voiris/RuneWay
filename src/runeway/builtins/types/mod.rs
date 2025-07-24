mod integer;
mod float;
mod null_type;
mod string;
mod boolean;
mod list;
mod iterator;
mod dict;
mod tuple;

use std::cell::RefMut;
pub use integer::RNWInteger;
pub use float::RNWFloat;
pub use null_type::RNWNullType;
pub use string::RNWString;
pub use boolean::RNWBoolean;
pub use list::RNWList;
pub use iterator::RNWIterator;
pub use dict::RNWDict;
pub use tuple::RNWTuple;
use crate::runeway::runtime::environment::Environment;
use crate::runeway::runtime::libraries::{register_module, RNWModule};
use crate::runeway::runtime::types::{register_function, register_type, register_type_class, RNWFunction, RNWType};

macro_rules! register_types {
    ( $env:expr; $( $ty:ty => $register_fn:expr ),+ $(,)? ) => {
        $(
            let obj = register_type::<$ty>($register_fn);
            $env.define_variable(obj.borrow().type_name.to_owned(), obj.clone());
        )+
    };
}

pub fn register_basic_types(borrow: &mut RefMut<Environment>) {
    register_types!(
        // Runtime
        borrow;
        RNWType => register_type_class(),
        RNWModule => register_module(),
        RNWFunction => register_function(),

        // Builtins
        RNWNullType => null_type::register(),
        RNWInteger => integer::register(),
        RNWString => string::register(),
        RNWBoolean => boolean::register(),
        RNWList => list::register(),
        RNWFloat => float::register(),
        RNWIterator => iterator::register(),
        RNWDict => dict::register(),
        RNWTuple => tuple::register(),
    );
}
