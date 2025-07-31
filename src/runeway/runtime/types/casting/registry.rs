use crate::runeway::core::errors::RWResult;
use crate::runeway::runtime::types::{RNWObject, RNWObjectRef, RNWTypeId};
use once_cell::sync::Lazy;
use std::cell::Ref;
use std::collections::HashMap;
use std::sync::RwLock;

type CastFn = fn(Ref<dyn RNWObject>) -> RWResult<RNWObjectRef>;

pub(super) static CAST_REGISTRY: Lazy<RwLock<HashMap<(RNWTypeId, RNWTypeId), CastFn>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

pub fn register_cast(from: RNWTypeId, to: RNWTypeId, func: CastFn) {
    let mut reg = CAST_REGISTRY.write().unwrap();
    reg.insert((from, to), func);
}
