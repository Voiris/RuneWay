use std::any::TypeId;
use std::cell::RefCell;
use std::rc::Rc;
use once_cell::unsync::Lazy;
use crate::runeway::runtime::types::RNWObject;
use crate::runeway::runtime::types::type_type::RNWType;

thread_local! {
    static REGISTERED_TYPES_DATA: Lazy<RefCell<Vec<Rc<RefCell<RNWType>>>>> = Lazy::new(|| {
        RefCell::new(Vec::new())
    });
}

pub fn register_type<T: 'static>(obj: Rc<RefCell<RNWType>>) -> Rc<RefCell<RNWType>> {
    REGISTERED_TYPES_DATA.with(
        |t| t.borrow_mut()
            .push(obj.clone()),
    );
    obj
}

pub fn type_name_from_id(id: &TypeId) -> &'static str {
    REGISTERED_TYPES_DATA.with(
        |t| t.borrow().iter().find(move |td| {
            &td.borrow().type_id == id
        }).cloned()
    ).map_or(
        "unknown_type",
        |rc| rc.borrow().type_name()
    )
}

pub fn type_obj_from_id(id: &TypeId) -> Rc<RefCell<RNWType>> {
    REGISTERED_TYPES_DATA.with(
        |t| t.borrow().iter().find(move |td| {
            &td.borrow().type_id == id
        }).cloned()
    ).unwrap()
}

