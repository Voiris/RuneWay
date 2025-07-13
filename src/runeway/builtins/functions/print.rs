use std::any::TypeId;
use std::rc::Rc;
use crate::runeway::builtins::types::{RNWNullType, RNWString};
use crate::runeway::executor::runtime::types::{RNWObject, RNWObjectRef, RNWRegisteredNativeFunction};

pub fn native_print(args: &[RNWObjectRef]) -> RNWObjectRef {
    print!("{}", args[0].borrow().value().downcast_ref::<String>().unwrap());

    RNWNullType::new()
}

pub fn native_println(args: &[RNWObjectRef]) -> RNWObjectRef {
    println!("{}", args[0].borrow().value().downcast_ref::<String>().unwrap());

    RNWNullType::new()
}

pub fn register_native_print() -> RNWRegisteredNativeFunction {
    RNWRegisteredNativeFunction::new(
        "print".to_owned(),
        Rc::new(native_print),
        vec![TypeId::of::<RNWString>()],
    )
}

pub fn register_native_println() -> RNWRegisteredNativeFunction {
    RNWRegisteredNativeFunction::new(
        "println".to_owned(),
        Rc::new(native_println),
        vec![TypeId::of::<RNWString>()],
    )
}
