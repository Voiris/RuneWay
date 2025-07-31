use crate::runeway::builtins::types::RNWString;
use crate::runeway::core::errors::RWResult;
use crate::runeway::core::utils::serde::serde_serialize;
use crate::runeway::runtime::types::{
    RNWFunction, RNWObject, RNWObjectRef, RNWRegisteredNativeFunction,
};
use std::rc::Rc;

pub fn native_http_get(args: &[RNWObjectRef]) -> RWResult<RNWObjectRef> {
    let borrow = args[0].borrow();
    let url = borrow.value().downcast_ref::<String>().unwrap();

    let client = reqwest::blocking::Client::new();

    let response = client.get(url).send()?.text()?;

    Ok(RNWString::new(response))
}

pub fn native_http_post(args: &[RNWObjectRef]) -> RWResult<RNWObjectRef> {
    let url_borrow = args[0].borrow();
    let url = url_borrow.value().downcast_ref::<String>().unwrap();

    let data = serde_serialize(args.get(1).unwrap().clone())?;

    let client = reqwest::blocking::Client::new();

    let response = client.post(url).json(&data).send()?.text()?;

    Ok(RNWString::new(response))
}

pub(super) fn register_native_http_get() -> RNWObjectRef {
    RNWFunction::new(RNWRegisteredNativeFunction::new(
        "http.get".to_owned(),
        Rc::new(native_http_get),
        vec![RNWString::rnw_type_id()],
    ))
}

pub(super) fn register_native_http_post() -> RNWObjectRef {
    RNWFunction::new(RNWRegisteredNativeFunction::new(
        "http.post".to_owned(),
        Rc::new(native_http_post),
        vec![RNWString::rnw_type_id(), 0],
    ))
}
