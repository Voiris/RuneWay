use std::any::{Any, TypeId};
use std::cell::{RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use once_cell::sync::Lazy;
use crate::runeway::builtins::types::{RNWBoolean, RNWDict, RNWNullType};
use crate::runeway::core::ast::operators::BinaryOperator;
use crate::runeway::runtime::types::{register_cast, RNWMethod, RNWObjectRef, RNWRegisteredNativeMethod, RNWType};
use crate::runeway::runtime::types::RNWObject;
use crate::runeway::builtins::types::string::RNWString;
use crate::runeway::core::errors::{RWResult, RuneWayError};

#[derive(Debug, Clone)]
pub struct RNWFloat {
    pub value: f64,
}

thread_local! {
    static FLOAT_NATIVE_FIELDS: Lazy<RefCell<HashMap<&'static str, RNWObjectRef>>> = Lazy::new(|| {
        let mut map = HashMap::new();

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

    fn field(&self, name: &str) -> Option<RNWObjectRef> {
        FLOAT_NATIVE_FIELDS.with(
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

pub(super) fn register() -> Rc<RefCell<RNWType>> {
    register_cast::<RNWFloat, RNWString>(|obj| {
        Ok(RNWString::new(obj.display()))
    });
    register_cast::<RNWFloat, RNWBoolean>(|obj| {
        Ok(RNWBoolean::new(obj.as_any().downcast_ref::<RNWFloat>().unwrap().value != 0.0))
    });

    RNWType::new::<RNWFloat>(RNWFloat::type_name())
}

