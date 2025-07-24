use std::any::{Any, TypeId};
use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::RwLock;
use once_cell::sync::Lazy;
use crate::runeway::core::errors::RWResult;
use crate::runeway::runtime::types::{RNWObject, RNWObjectRef};

type CastFn = fn(Ref<dyn RNWObject>) -> RWResult<RNWObjectRef>;

pub(super) static CAST_REGISTRY: Lazy<RwLock<HashMap<(TypeId, TypeId), CastFn>>> = Lazy::new(|| RwLock::new(HashMap::new()));

pub fn register_cast<FROM: 'static, TO: 'static>(func: CastFn) {
    let mut reg = CAST_REGISTRY.write().unwrap();
    reg.insert((TypeId::of::<FROM>(), TypeId::of::<TO>()), func);
}
