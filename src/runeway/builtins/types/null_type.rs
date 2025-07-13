use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use once_cell::unsync::Lazy;
use crate::runeway::builtins::types::RNWString;
use crate::runeway::executor::runtime::types::{RNWObject, RNWObjectRef, RNWRegisteredNativeMethod};

#[derive(Debug, Clone)]
pub struct RNWNullType;

fn native_null_to_string(_: RNWObjectRef, _: &[RNWObjectRef]) -> RNWObjectRef {
    RNWString::new(RNWNullType.display())
}

thread_local! {
    static NULL_TYPE_NATIVE_METHODS: Lazy<RefCell<HashMap<&'static str, RNWRegisteredNativeMethod>>> = Lazy::new(|| {
        let mut map = HashMap::new();

        map.insert("to_string", RNWRegisteredNativeMethod::new(
            "int.to_string".to_string(),
            Rc::new(native_null_to_string),
            vec![TypeId::of::<RNWNullType>()]
        ));

        RefCell::new(map)
    });
}

impl RNWNullType {
    pub fn new() -> RNWObjectRef {
        Rc::new(RefCell::new(Self {}))
    }

    pub fn is_type_equals(other: &RNWObjectRef) -> bool {
        std::any::TypeId::of::<Self>() == other.borrow().as_any().type_id()
    }

    pub fn type_name() -> &'static str { "NullType" }
}

impl RNWObject for RNWNullType {
    fn type_name(&self) -> &'static str { Self::type_name() }
    fn display(&self) -> String {
        "null".to_string()
    }
    fn value(&self) -> &dyn Any {
        self
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn as_object(&self) -> &dyn RNWObject {
        self
    }

    //noinspection DuplicatedCode
    fn method(&self, name: &str) -> Option<RNWRegisteredNativeMethod> {
        NULL_TYPE_NATIVE_METHODS.with(|m| {
            m.borrow().get(name).cloned()
        })
    }
}
