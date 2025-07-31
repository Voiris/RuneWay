use crate::runeway::runtime::types::type_type::RNWType;
use crate::runeway::runtime::types::RNWTypeId;
use once_cell::unsync::Lazy;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

thread_local! {
    static REGISTERED_TYPES_DATA: Lazy<RefCell<HashMap<RNWTypeId, Rc<RefCell<RNWType>>>>> = Lazy::new(|| {
        RefCell::new(HashMap::new())
    });
}

pub fn register_type(rnw_type_id: RNWTypeId, obj: Rc<RefCell<RNWType>>) -> Rc<RefCell<RNWType>> {
    REGISTERED_TYPES_DATA.with(|t| t.borrow_mut().insert(rnw_type_id, obj.clone()));
    obj
}

pub fn type_name_from_id(id: RNWTypeId) -> &'static str {
    REGISTERED_TYPES_DATA.with(|t| {
        t.borrow()
            .get(&id)
            .map_or("unknown_type", |rc| rc.borrow().type_name)
    })
}

pub fn type_obj_from_id(id: RNWTypeId) -> Rc<RefCell<RNWType>> {
    REGISTERED_TYPES_DATA
        .with(|t| t.borrow().get(&id).unwrap().clone())
        .clone()
}
