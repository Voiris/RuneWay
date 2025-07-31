use crate::runeway::core::errors::{RWResult, RuneWayError, RuneWayErrorKind};
use crate::runeway::runtime::types::{type_name_from_id, RNWTypeId};
use std::panic::Location;

#[track_caller]
pub fn assert_incorrect_type(expected: RNWTypeId, type_id: RNWTypeId) -> RWResult<()> {
    if expected != type_id {
        let location = Location::caller();
        println!("{}", location);
        Err(
            RuneWayError::new(RuneWayErrorKind::type_error()).with_message(format!(
                "Incorrect value type. Expected <{}>, but <{}> were provided",
                type_name_from_id(expected),
                type_name_from_id(type_id)
            )),
        )
    } else {
        Ok(())
    }
}
