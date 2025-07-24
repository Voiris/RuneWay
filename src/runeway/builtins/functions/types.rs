use std::any::{Any, TypeId};
use std::rc::Rc;
use crate::runeway::core::errors::RWResult;
use crate::runeway::runtime::types::{cast_to_type_id, type_obj_from_id, RNWObject, RNWObjectRef, RNWRegisteredNativeFunction, RNWType};

pub fn native_type_cast(args: &[RNWObjectRef]) -> RWResult<RNWObjectRef> {
    let obj = args.get(0).unwrap();
    let type_id = {
        let type_obj = args.get(1).unwrap();
        let type_borrow = type_obj.borrow();
        type_borrow.as_any().downcast_ref::<RNWType>().unwrap().type_id.clone()
    };
    let casted_obj = cast_to_type_id(obj, type_id)?;

    Ok(casted_obj)
}

pub fn register_native_type_cast() -> Rc<RNWRegisteredNativeFunction> {
    RNWRegisteredNativeFunction::new(
        "cast".to_owned(),
        Rc::new(native_type_cast),
        vec![TypeId::of::<dyn RNWObject>(), TypeId::of::<RNWType>()],
    )
}
