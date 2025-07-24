use std::any::{Any, TypeId};
use crate::runeway::core::errors::{RWResult, RuneWayError, RuneWayErrorKind};
use crate::runeway::runtime::types::{type_name_from_id, RNWObject, RNWObjectRef};
use crate::runeway::runtime::types::casting::registry::CAST_REGISTRY;

mod registry;

pub use registry::register_cast;
pub fn cast_to<TO: 'static>(obj: &RNWObjectRef) -> RWResult<RNWObjectRef> {
    cast_to_type_id(obj, TypeId::of::<TO>())
}

pub fn cast_to_type_id(obj: &RNWObjectRef, target_type_id: TypeId) -> RWResult<RNWObjectRef> {
    let obj_borrow = obj.borrow();
    let source_type_id = obj_borrow.as_any().type_id();

    if source_type_id == target_type_id {
        return Ok(obj.clone());
    }

    let reg = CAST_REGISTRY.read().unwrap();
    if let Some(func) = reg.get(&(source_type_id, target_type_id)) {
        func(obj_borrow)
    } else {
        Err(
            RuneWayError::new(RuneWayErrorKind::Runtime(Some("CastError".to_string())))
                .with_message(format!(
                    "Cannot cast <{}> to <{}>",
                    obj_borrow.type_name(),
                    type_name_from_id(&target_type_id)
                ))
        )
    }
}

pub(super) trait CastTo {
    fn cast_to<T: 'static>(&self) -> RWResult<RNWObjectRef>;
    fn cast_to_type_id(&self, target_type_id: TypeId) -> RWResult<RNWObjectRef>;
}

