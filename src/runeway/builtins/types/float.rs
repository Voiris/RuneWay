use std::any::{Any, TypeId};
use std::cell::{RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use once_cell::sync::Lazy;
use crate::runeway::core::ast::operators::BinaryOperator;
use crate::runeway::executor::runtime::types::{RNWObjectRef, RNWRegisteredNativeMethod};
use crate::runeway::executor::runtime::types::RNWObject;
use crate::runeway::builtins::types::string::RNWString;

#[derive(Debug, Clone)]
pub struct RNWFloat {
    pub value: f64,
}

fn native_float_to_string(this: RNWObjectRef, _: &[RNWObjectRef]) -> RNWObjectRef {
    let mut s = (*this).borrow().value().downcast_ref::<f64>().unwrap().to_string();
    if !s.contains(".") {
        s.push_str(".0");
    }

    RNWString::new(s)
}

thread_local! {
    static INTEGER_NATIVE_METHODS: Lazy<RefCell<HashMap<&'static str, RNWRegisteredNativeMethod>>> = Lazy::new(|| {
        let mut map = HashMap::new();

        map.insert("to_string", RNWRegisteredNativeMethod::new(
            "float.to_string".to_string(),
            Rc::new(native_float_to_string),
            vec![TypeId::of::<RNWFloat>()]
        ));

        RefCell::new(map)
    });
}

// Integer implements
impl RNWFloat {
    pub fn new(value: f64) -> RNWObjectRef {
        Rc::new(RefCell::new(Self { value }))
    }

    pub fn is_type_equals(other: RNWObjectRef) -> bool {
        TypeId::of::<Self>() == other.borrow().as_any().type_id()
    }

    pub fn type_name() -> &'static str { "float" }
}

impl RNWObject for RNWFloat {
    fn type_name(&self) -> &'static str { Self::type_name() }
    fn display(&self) -> String {
        self.value.to_string()
    }
    fn value(&self) -> &dyn Any {
        &self.value
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

    fn method(&self, name: &str) -> Option<RNWRegisteredNativeMethod> {
        INTEGER_NATIVE_METHODS.with(
            |methods| methods.borrow().get(name).cloned()
        )
    }

    fn binary_operation(&self, other: RNWObjectRef, binary_operator: BinaryOperator) -> Option<RNWObjectRef> {
        match (binary_operator, other.borrow().type_name()) {
            (BinaryOperator::Add, "int") => {
                Some(RNWFloat::new(self.value + (other.borrow()
                    .value().downcast_ref::<i64>().unwrap().clone() as f64)))
            }
            (BinaryOperator::Sub, "int") => {
                Some(RNWFloat::new(self.value - (other.borrow()
                    .value().downcast_ref::<i64>().unwrap().clone() as f64)))
            }
            (BinaryOperator::Add, "float") => {
                Some(RNWFloat::new(self.value + other.borrow().value().downcast_ref::<f64>().unwrap()))
            }
            (BinaryOperator::Sub, "float") => {
                Some(RNWFloat::new(self.value - other.borrow().value().downcast_ref::<f64>().unwrap()))
            }
            _ => None,
        }
    }
}
