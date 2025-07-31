mod boolean;
mod dict;
mod float;
mod integer;
mod iterator;
mod list;
mod null_type;
mod string;
mod tuple;
mod u_integer;

use crate::runeway::runtime::environment::Environment;
use crate::runeway::runtime::libraries::{register_module, RNWModule};
use crate::runeway::runtime::types::{
    register_function, register_type_class, RNWFunction, RNWType,
};
pub use boolean::RNWBoolean;
pub use dict::RNWDict;
pub use float::RNWFloat;
pub use integer::RNWInteger;
pub use iterator::RNWIterator;
pub use list::RNWList;
pub use null_type::RNWNullType;
use std::cell::RefMut;
pub use string::RNWString;
pub use tuple::RNWTuple;
pub use u_integer::RNWUnsignedInteger;

#[macro_export]
macro_rules! register_types {
    ( $env:expr; $( $ty:ty => $register_fn:expr ),+ $(,)? ) => {
        $(
            let obj = crate::runeway::runtime::types::types_reg::register_type(<$ty>::rnw_type_id(), $register_fn);
            $env.define_variable(
                obj.borrow().type_name.to_owned(),
                obj.clone()
            );
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
        RNWUnsignedInteger => u_integer::register(),
    );
}
