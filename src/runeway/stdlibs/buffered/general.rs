use crate::runeway::builtins::types::{RNWNullType, RNWString};
use crate::runeway::core::errors::RWResult;
use crate::runeway::runtime::types::{RNWFunction, RNWObjectRef, RNWRegisteredNativeFunction};
use once_cell::sync::OnceCell;
use std::io::{stdout, BufWriter, Stdout, Write};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

static BUFFER: OnceCell<Arc<Mutex<BufWriter<Stdout>>>> = OnceCell::new();

fn get_buffer() -> Arc<Mutex<BufWriter<Stdout>>> {
    BUFFER
        .get_or_init(|| Arc::new(Mutex::new(BufWriter::new(stdout()))))
        .clone()
}

pub fn native_buffered_print(args: &[RNWObjectRef]) -> RWResult<RNWObjectRef> {
    let borrow = args[0].borrow();
    let string = borrow.value().downcast_ref::<String>().unwrap();
    let buffer = get_buffer();
    let mut guard = buffer.lock().unwrap();

    guard.write_all(string.as_bytes())?;

    Ok(RNWNullType::new())
}

pub fn native_buffered_println(args: &[RNWObjectRef]) -> RWResult<RNWObjectRef> {
    let borrow = args[0].borrow();
    let string = format!("{}\n", borrow.value().downcast_ref::<String>().unwrap());
    let buffer = get_buffer();
    let mut guard = buffer.lock().unwrap();

    guard.write_all(string.as_bytes())?;

    Ok(RNWNullType::new())
}

pub fn native_buffered_flush(_: &[RNWObjectRef]) -> RWResult<RNWObjectRef> {
    let buffer = get_buffer();
    let mut guard = buffer.lock().unwrap();

    guard.flush()?;

    Ok(RNWNullType::new())
}

// Future: buffered.show() -> string

pub fn register_native_buffered_print() -> RNWObjectRef {
    RNWFunction::new(RNWRegisteredNativeFunction::new(
        "buffered.print".to_owned(),
        Rc::new(native_buffered_print),
        vec![RNWString::rnw_type_id()],
    ))
}

pub fn register_native_buffered_println() -> RNWObjectRef {
    RNWFunction::new(RNWRegisteredNativeFunction::new(
        "buffered.println".to_owned(),
        Rc::new(native_buffered_println),
        vec![RNWString::rnw_type_id()],
    ))
}

pub fn register_native_buffered_flush() -> RNWObjectRef {
    RNWFunction::new(RNWRegisteredNativeFunction::new(
        "buffered.flush".to_owned(),
        Rc::new(native_buffered_flush),
        vec![],
    ))
}
