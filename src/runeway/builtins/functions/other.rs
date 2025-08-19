use crate::runeway::builtins::types::{RNWBoolean, RNWUnsignedInteger};
use crate::runeway::core::errors::RWResult;
use crate::runeway::runtime::types::{
    RNWObject, RNWObjectRef, RNWRegisteredNativeFunction, RNWType, UserDefinedClass, cast_to,
};
use std::rc::Rc;

pub fn native_type_cast(args: &[RNWObjectRef]) -> RWResult<RNWObjectRef> {
    let obj = args.get(0).unwrap();
    let type_id = {
        let type_obj = args.get(1).unwrap();
        let type_borrow = type_obj.borrow();
        type_borrow
            .as_any()
            .downcast_ref::<RNWType>()
            .unwrap()
            .rnw_type_id
    };
    let casted_obj = cast_to(obj, type_id)?;

    Ok(casted_obj)
}

pub fn native_object_id(args: &[RNWObjectRef]) -> RWResult<RNWObjectRef> {
    let obj = args.get(0).unwrap();

    let id = {
        let ptr = Rc::as_ptr(obj) as *const ();
        ptr as u64
    };

    Ok(RNWUnsignedInteger::new(id))
}

pub fn native_is_instance(args: &[RNWObjectRef]) -> RWResult<RNWObjectRef> {
    let object_rwn_type_id = {
        let obj = args.get(0).unwrap();
        let obj_borrow = obj.borrow();
        if obj_borrow.as_any().downcast_ref::<RNWType>().is_none() {
            if let Some(c) = obj_borrow.as_any().downcast_ref::<UserDefinedClass>() {
                if c.is_instance {
                    Some(c.rnw_type_id)
                } else {
                    None
                }
            } else {
                Some(obj_borrow.rnw_type_id())
            }
        } else {
            Some(obj_borrow.rnw_type_id())
        }
    };
    let class_rwn_type_id = {
        let type_obj = args.get(1).unwrap();
        let type_borrow = type_obj.borrow();
        if let Some(t) = type_borrow.as_any().downcast_ref::<RNWType>() {
            Some(t.rnw_type_id)
        } else if let Some(c) = type_borrow.as_any().downcast_ref::<UserDefinedClass>() {
            Some(c.rnw_type_id)
        } else {
            None
        }
    };

    let bool = match (object_rwn_type_id, class_rwn_type_id) {
        (Some(x), Some(y)) => { x == y || y == RNWType::rnw_type_id() },
        _ => false,
    };

    Ok(RNWBoolean::new(bool))
}

pub fn register_native_type_cast() -> Rc<RNWRegisteredNativeFunction> {
    RNWRegisteredNativeFunction::new(
        "cast".to_owned(),
        Rc::new(native_type_cast),
        vec![0, RNWType::rnw_type_id()],
    )
}

pub fn register_native_object_id() -> Rc<RNWRegisteredNativeFunction> {
    RNWRegisteredNativeFunction::new("id".to_owned(), Rc::new(native_object_id), vec![0])
}

pub fn register_native_is_instance() -> Rc<RNWRegisteredNativeFunction> {
    RNWRegisteredNativeFunction::new(
        "is_instance".to_owned(),
        Rc::new(native_is_instance),
        vec![0, 0],
    )
}
