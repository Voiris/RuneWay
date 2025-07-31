use crate::runeway::builtins::types::{RNWBoolean, RNWFloat, RNWInteger};
use crate::runeway::core::errors::RWResult;
use crate::runeway::runtime::types::{RNWFunction, RNWObjectRef, RNWRegisteredNativeFunction};
use std::rc::Rc;

pub fn native_random_positive(_: &[RNWObjectRef]) -> RWResult<RNWObjectRef> {
    Ok(RNWInteger::new(rand::random_range(0..=i64::MAX)))
}

pub fn native_random_negative(_: &[RNWObjectRef]) -> RWResult<RNWObjectRef> {
    Ok(RNWInteger::new(rand::random_range(i64::MIN..0)))
}

pub fn native_random_random_int(_: &[RNWObjectRef]) -> RWResult<RNWObjectRef> {
    Ok(RNWInteger::new(rand::random::<i64>()))
}

pub fn native_random_unit(_: &[RNWObjectRef]) -> RWResult<RNWObjectRef> {
    Ok(RNWFloat::new(rand::random::<f64>()))
}

pub fn native_random_random_bool(_: &[RNWObjectRef]) -> RWResult<RNWObjectRef> {
    Ok(RNWBoolean::new(rand::random::<f64>() < 0.5))
}

pub fn native_random_random_range(args: &[RNWObjectRef]) -> RWResult<RNWObjectRef> {
    let (start, end) = {
        let start_obj = args.get(0).unwrap();
        let end_obj = args.get(1).unwrap();

        let start_borrow = start_obj.borrow();
        let end_borrow = end_obj.borrow();

        let start = start_borrow.value().downcast_ref::<i64>().unwrap();
        let end = end_borrow.value().downcast_ref::<i64>().unwrap();

        (start.clone(), end.clone())
    };

    Ok(RNWInteger::new(rand::random_range(start..=end)))
}

pub fn register_native_random_positive() -> RNWObjectRef {
    RNWFunction::new(RNWRegisteredNativeFunction::new(
        "random.positive".to_owned(),
        Rc::new(native_random_positive),
        vec![],
    ))
}

pub fn register_native_random_negative() -> RNWObjectRef {
    RNWFunction::new(RNWRegisteredNativeFunction::new(
        "random.negative".to_owned(),
        Rc::new(native_random_negative),
        vec![],
    ))
}

pub fn register_native_random_random_int() -> RNWObjectRef {
    RNWFunction::new(RNWRegisteredNativeFunction::new(
        "random.rand_int".to_owned(),
        Rc::new(native_random_random_int),
        vec![],
    ))
}

pub fn register_native_random_unit() -> RNWObjectRef {
    RNWFunction::new(RNWRegisteredNativeFunction::new(
        "random.unit".to_owned(),
        Rc::new(native_random_unit),
        vec![],
    ))
}

pub fn register_native_random_random_bool() -> RNWObjectRef {
    RNWFunction::new(RNWRegisteredNativeFunction::new(
        "random.rand_bool".to_owned(),
        Rc::new(native_random_random_bool),
        vec![],
    ))
}

pub fn register_native_random_random_range() -> RNWObjectRef {
    RNWFunction::new(RNWRegisteredNativeFunction::new(
        "random.rand_range".to_owned(),
        Rc::new(native_random_random_range),
        vec![RNWInteger::rnw_type_id(), RNWInteger::rnw_type_id()],
    ))
}
