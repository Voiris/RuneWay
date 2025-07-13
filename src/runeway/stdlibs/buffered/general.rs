use std::any::TypeId;
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use once_cell::sync::OnceCell;
use crate::runeway::builtins::types::{RNWNullType, RNWString};
use crate::runeway::executor::runtime::types::{RNWObjectRef, RNWRegisteredNativeFunction};

static BUFFER: OnceCell<Arc<RwLock<String>>> = OnceCell::new();

fn get_buffer() -> Arc<RwLock<String>> {
    BUFFER.get_or_init(|| Arc::new(RwLock::new(String::new()))).clone()
}

pub fn push_buffer(s: impl AsRef<str>) {
    let lock = get_buffer();
    let mut buffer = lock.write().unwrap();
    buffer.push_str(s.as_ref());
}

pub fn clear_buffer() {
    let lock = get_buffer();
    let mut buffer = lock.write().unwrap();
    buffer.clear();
}

pub fn read_buffer() -> String {
    let lock = get_buffer();
    let buffer = lock.read().unwrap();
    buffer.clone()
}

pub fn native_buffered_print(args: &[RNWObjectRef]) -> RNWObjectRef {
    push_buffer(args[0].borrow().value().downcast_ref::<String>().unwrap());

    RNWNullType::new()
}

pub fn native_buffered_println(args: &[RNWObjectRef]) -> RNWObjectRef {
    push_buffer(format!("{}\n", args[0].borrow().value().downcast_ref::<String>().unwrap()));

    RNWNullType::new()
}

pub fn native_buffered_flush(_: &[RNWObjectRef]) -> RNWObjectRef {
    print!("{}", read_buffer());
    clear_buffer();

    RNWNullType::new()
}

// Future: buffered.show() -> string

pub fn register_native_buffered_print() -> RNWRegisteredNativeFunction {
    RNWRegisteredNativeFunction::new(
        "print".to_owned(),
        Rc::new(native_buffered_print),
        vec![TypeId::of::<RNWString>()],
    )
}

pub fn register_native_buffered_println() -> RNWRegisteredNativeFunction {
    RNWRegisteredNativeFunction::new(
        "println".to_owned(),
        Rc::new(native_buffered_println),
        vec![TypeId::of::<RNWString>()],
    )
}

pub fn register_native_buffered_flush() -> RNWRegisteredNativeFunction {
    RNWRegisteredNativeFunction::new(
        "flush".to_owned(),
        Rc::new(native_buffered_flush),
        vec![],
    )
}
