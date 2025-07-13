use std::collections::HashMap;
use std::any::TypeId;
use std::cell::RefCell;
use once_cell::unsync::Lazy;

thread_local! {
    static TYPE_ID_TO_NAME: Lazy<RefCell<HashMap<TypeId, &'static str>>> = Lazy::new(|| {
        RefCell::new(HashMap::new())
    });
}

pub fn register_type<T: 'static>(name: &'static str) {
    TYPE_ID_TO_NAME.with(
        |t| t.borrow_mut().insert(TypeId::of::<T>(), name),
    );
}

pub fn type_name_from_id(id: &TypeId) -> &'static str {
    TYPE_ID_TO_NAME.with(
        |t| t.borrow().get(id).copied()
    ).unwrap_or("unknown_type")
}
