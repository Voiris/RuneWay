use std::cell::RefCell;
use std::collections::HashMap;
use once_cell::unsync::Lazy;
use crate::runeway::executor::runtime::environment::EnvRef;

type StdLibLoader = fn() -> EnvRef;

thread_local! {
    static STD_LIBS_REGISTRY: Lazy<RefCell<HashMap<&'static str, StdLibLoader>>> = Lazy::new(|| {
        RefCell::new(HashMap::new())
    })
}

pub fn register_stdlib(name: &'static str, loader: StdLibLoader) {
    STD_LIBS_REGISTRY.with(|reg| {
        reg.borrow_mut().insert(name, loader);
    })
}

pub fn get_stdlib(name: impl AsRef<str>) -> Option<StdLibLoader> {
    STD_LIBS_REGISTRY.with(|reg| {
        reg.borrow().get(name.as_ref()).cloned()
    })
}
