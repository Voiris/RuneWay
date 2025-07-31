use crate::runeway::core::errors::{RWResult, RuneWayError, RuneWayErrorKind};
use crate::runeway::runtime::types::casting::registry::CAST_REGISTRY;
use crate::runeway::runtime::types::{type_name_from_id, RNWObjectRef, RNWTypeId};

mod registry;

pub use registry::register_cast;

pub fn cast_to(obj: &RNWObjectRef, target_type_id: RNWTypeId) -> RWResult<RNWObjectRef> {
    let obj_borrow = obj.borrow();
    let source_type_id = obj_borrow.rnw_type_id();

    if source_type_id == target_type_id {
        return Ok(obj.clone());
    }

    let reg = CAST_REGISTRY.read().unwrap();
    if let Some(func) = reg.get(&(source_type_id, target_type_id)) {
        func(obj_borrow)
    } else {
        Err(
            RuneWayError::new(RuneWayErrorKind::error_with_code("CastError")).with_message(
                format!(
                    "Cannot cast <{}> to <{}>",
                    obj_borrow.type_name(),
                    type_name_from_id(target_type_id)
                ),
            ),
        )
    }
}
