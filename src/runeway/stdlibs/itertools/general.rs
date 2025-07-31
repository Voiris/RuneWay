use crate::runeway::builtins::types::{RNWBoolean, RNWIterator};
use crate::runeway::core::ast::operators::BinaryOperator;
use crate::runeway::core::errors::{RWResult, RuneWayError, RuneWayErrorKind};
use crate::runeway::runtime::types::{
    cast_to, RNWFunction, RNWObject, RNWObjectRef, RNWRegisteredNativeFunction,
};
use colored::Colorize;
use std::rc::Rc;

fn get_iterator(obj: &RNWObjectRef) -> RWResult<RNWIterator> {
    let iter_obj = cast_to(obj, RNWIterator::rnw_type_id())?;

    let iter_borrow = iter_obj.borrow();

    Ok(iter_borrow
        .as_any()
        .downcast_ref::<RNWIterator>()
        .unwrap()
        .clone())
}

pub fn native_itertools_any(args: &[RNWObjectRef]) -> RWResult<RNWObjectRef> {
    for i in get_iterator(args.get(0).unwrap())? {
        let boolean_obj = cast_to(&i, RNWBoolean::rnw_type_id())?;
        let boolean_borrow = boolean_obj.borrow();
        match boolean_borrow.value().downcast_ref::<bool>() {
            Some(true) => return Ok(RNWBoolean::new(true)),
            Some(false) => (),
            None => unreachable!(),
        }
    }
    Ok(RNWBoolean::new(false))
}

pub fn native_itertools_all(args: &[RNWObjectRef]) -> RWResult<RNWObjectRef> {
    for i in get_iterator(args.get(0).unwrap())? {
        let boolean_obj = cast_to(&i, RNWBoolean::rnw_type_id())?;
        let boolean_borrow = boolean_obj.borrow();
        match boolean_borrow.value().downcast_ref::<bool>() {
            Some(true) => (),
            Some(false) => return Ok(RNWBoolean::new(false)),
            None => unreachable!(),
        }
    }
    Ok(RNWBoolean::new(true))
}

//noinspection DuplicatedCode
pub fn native_itertools_iter_equal(args: &[RNWObjectRef]) -> RWResult<RNWObjectRef> {
    let first = get_iterator(args.get(0).unwrap())?;
    let second = get_iterator(args.get(1).unwrap())?;

    for (f, s) in first.into_iter().zip(second.into_iter()) {
        match f.borrow().binary_operation(s.clone(), BinaryOperator::Eq) {
            Some(obj) => {
                let boolean_borrow = obj.borrow();
                match boolean_borrow.value().downcast_ref::<bool>() {
                    Some(false) => return Ok(RNWBoolean::new(false)),
                    Some(true) => (),
                    None => unreachable!(),
                }
            }
            None => {
                return Err(
                    RuneWayError::new(RuneWayErrorKind::error_with_code("OperationError"))
                        .with_message(format!(
                            "Not supported binary operation: `{} {} {}`",
                            f.borrow().type_name().bright_yellow(),
                            BinaryOperator::Eq.display().bright_red(),
                            s.borrow().type_name().bright_yellow()
                        )),
                );
            }
        }
    }

    Ok(RNWBoolean::new(true))
}

pub fn register_native_itertools_any() -> RNWObjectRef {
    RNWFunction::new(RNWRegisteredNativeFunction::new(
        "itertools.any".to_owned(),
        Rc::new(native_itertools_any),
        vec![0],
    ))
}

pub fn register_native_itertools_all() -> RNWObjectRef {
    RNWFunction::new(RNWRegisteredNativeFunction::new(
        "itertools.all".to_owned(),
        Rc::new(native_itertools_all),
        vec![0],
    ))
}

pub fn register_native_itertools_iter_equal() -> RNWObjectRef {
    RNWFunction::new(RNWRegisteredNativeFunction::new(
        "itertools.iter_equal".to_owned(),
        Rc::new(native_itertools_iter_equal),
        vec![0, 0],
    ))
}
