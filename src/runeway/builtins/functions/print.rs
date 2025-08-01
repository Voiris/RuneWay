use crate::runeway::builtins::types::{RNWNullType, RNWString};
use crate::runeway::core::errors::RWResult;
use crate::runeway::runtime::types::{
    cast_to, RNWObject, RNWObjectRef, RNWRegisteredNativeFunction,
};
use std::rc::Rc;

pub fn cast_args_to_string(args: &[RNWObjectRef]) -> RWResult<Vec<String>> {
    let mut string_args = Vec::new();
    for arg in args {
        let casted_arg = cast_to(&arg, RNWString::rnw_type_id())?;
        string_args.push(
            casted_arg
                .borrow()
                .value()
                .downcast_ref::<String>()
                .unwrap()
                .clone(),
        );
    }
    Ok(string_args)
}

pub fn native_print(args: &[RNWObjectRef]) -> RWResult<RNWObjectRef> {
    print!("{}", cast_args_to_string(args)?.join(" "));

    Ok(RNWNullType::new())
}

pub fn native_println(args: &[RNWObjectRef]) -> RWResult<RNWObjectRef> {
    println!("{}", cast_args_to_string(args)?.join(" "));

    Ok(RNWNullType::new())
}

pub fn register_native_print() -> Rc<RNWRegisteredNativeFunction> {
    RNWRegisteredNativeFunction::new_unlimited("print".to_owned(), Rc::new(native_print), vec![0])
}

pub fn register_native_println() -> Rc<RNWRegisteredNativeFunction> {
    RNWRegisteredNativeFunction::new_unlimited(
        "println".to_owned(),
        Rc::new(native_println),
        vec![0],
    )
}
