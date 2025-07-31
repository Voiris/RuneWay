use crate::runeway::runtime::environment::EnvRef;
use once_cell::unsync::Lazy;
use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    static LOADED_LIBS: Lazy<RefCell<HashMap<String, EnvRef>>> = Lazy::new(|| {
        RefCell::new(HashMap::new())
    });
}

pub fn register_loaded(path: &String, lib: EnvRef) {
    LOADED_LIBS.with(|t| t.borrow_mut().insert(path.clone(), lib.clone()));
}

pub fn get_loaded(path: &String) -> Option<EnvRef> {
    LOADED_LIBS.with(|m| m.borrow().get(path).cloned())
}

pub fn is_loaded(path: &String) -> bool {
    LOADED_LIBS.with(|l| l.borrow().contains_key(path))
}
