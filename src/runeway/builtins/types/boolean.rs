use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use once_cell::sync::Lazy;
use crate::runeway::core::ast::operators::{BinaryOperator, UnaryOperator};
use crate::runeway::executor::runtime::types::{RNWRegisteredNativeMethod, RNWObject, RNWObjectRef};
use crate::runeway::builtins::types::RNWString;

#[derive(Debug, Clone)]
pub struct RNWBoolean {
    value: bool
}

fn native_bool_to_string(this: RNWObjectRef, _: &[RNWObjectRef]) -> RNWObjectRef {
    RNWString::new((*this).borrow().value().downcast_ref::<bool>().unwrap().to_string())
}

thread_local! {
    static BOOLEAN_NATIVE_METHODS: Lazy<RefCell<HashMap<&'static str, RNWRegisteredNativeMethod>>> = Lazy::new(|| {
        let mut map = HashMap::new();

        map.insert("to_string", RNWRegisteredNativeMethod::new(
            "bool.to_string".to_string(),
            Rc::new(native_bool_to_string),
            vec![TypeId::of::<RNWBoolean>()]
        ));

        RefCell::new(map)
    });
}

impl RNWBoolean {
    pub fn new(value: bool) -> RNWObjectRef {
        Rc::new(RefCell::new(Self { value }))
    }

    pub fn type_name() -> &'static str { "bool" }

    pub fn is_type_equals(other: RNWObjectRef) -> bool {
        TypeId::of::<Self>() == other.borrow().as_any().type_id()
    }
}

impl RNWObject for RNWBoolean {
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
        BOOLEAN_NATIVE_METHODS.with(
            |methods| methods.borrow().get(name).cloned()
        )
    }

    fn binary_operation(&self, other: RNWObjectRef, binary_operator: BinaryOperator) -> Option<RNWObjectRef> {
        let result = if let Some(&other_value) = other.borrow().value().downcast_ref::<bool>() {
            match binary_operator {
                BinaryOperator::Or => {
                    RNWBoolean::new(self.value || other_value)
                }
                BinaryOperator::And => {
                    RNWBoolean::new(self.value && other_value)
                }
                BinaryOperator::Eq => {
                    RNWBoolean::new(self.value == other_value)
                }
                BinaryOperator::NotEq => {
                    RNWBoolean::new(self.value != other_value)
                }
                _ => return None
            }
        } else {
            return None
        };

        Some(result)
    }
    
    fn unary_operation(&self, unary_operator: UnaryOperator) -> Option<RNWObjectRef> {
        match unary_operator {
            UnaryOperator::Not => Some(RNWBoolean::new(!self.value)),
            _ => None,
        }
    }
}
