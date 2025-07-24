use std::any::TypeId;
use std::rc::Rc;
use crate::runeway::builtins::types::{RNWString};
use crate::runeway::core::errors::RWResult;
use crate::runeway::runtime::types::{RNWFunction, RNWObject, RNWObjectRef, RNWRegisteredNativeFunction};
use crate::runeway::utils::serde::{serde_evaluate, serde_serialize};

pub fn native_json_load(args: &[RNWObjectRef]) -> RWResult<RNWObjectRef> {
    let borrow = args[0].borrow();
    let string = borrow.value().downcast_ref::<String>().unwrap();

    let value = serde_json::from_str(&string)?;

    Ok(serde_evaluate(value))
}

//noinspection DuplicatedCode
pub fn native_json_dump(args: &[RNWObjectRef]) -> RWResult<RNWObjectRef> {
    let serialized = serde_json::to_string(&serde_serialize(
        args.get(0).unwrap().clone()
    )?)?;

    Ok(RNWString::new(serialized))
}

pub(super) fn register_native_json_load() -> RNWObjectRef {
    RNWFunction::new(RNWRegisteredNativeFunction::new(
        "json.load".to_owned(),
        Rc::new(native_json_load),
        vec![TypeId::of::<RNWString>()],
    ))
}

pub(super) fn register_native_json_dump() -> RNWObjectRef {
    RNWFunction::new(RNWRegisteredNativeFunction::new(
        "json.dump".to_owned(),
        Rc::new(native_json_dump),
        vec![TypeId::of::<dyn RNWObject>()],
    ))
}
