use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use once_cell::unsync::Lazy;
use crate::runeway::builtins::types::{RNWBoolean, RNWDict, RNWFloat, RNWNullType};
use crate::runeway::core::ast::operators::BinaryOperator;
use crate::runeway::runtime::types::{register_cast, RNWMethod, RNWObject, RNWRegisteredNativeMethod, RNWType};
use crate::runeway::runtime::types::RNWObjectRef;
use crate::runeway::builtins::types::string::RNWString;
use crate::runeway::core::errors::RWResult;

#[derive(Debug, Clone)]
pub struct RNWInteger {
    pub value: i64,
}

thread_local! {
    static INTEGER_NATIVE_FIELDS: Lazy<RefCell<HashMap<&'static str, RNWObjectRef>>> = Lazy::new(|| {
        let mut map = HashMap::new();

        RefCell::new(map)
    });
}

// Integer implements
impl RNWInteger {
    pub fn new(value: i64) -> RNWObjectRef {
        Rc::new(RefCell::new(Self { value }))
    }

    pub fn is_type_equals(other: RNWObjectRef) -> bool {
        TypeId::of::<Self>() == other.borrow().as_any().type_id()
    }

    pub fn type_name() -> &'static str { "int" }
}

impl RNWObject for RNWInteger {
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
        INTEGER_NATIVE_FIELDS.with(
            |methods| methods.borrow().get(name).cloned()
        )
    }

    fn binary_operation(&self, other: RNWObjectRef, binary_operator: BinaryOperator) -> Option<RNWObjectRef> {
        if other.borrow().type_name() == "int" {
            let other_value = other.borrow().value().downcast_ref::<i64>().cloned().unwrap();

            Some(match binary_operator {
                // Arithmetic
                BinaryOperator::Add => {
                    RNWInteger::new(self.value + other_value)
                }
                BinaryOperator::Sub => {
                    RNWInteger::new(self.value - other_value)
                }
                BinaryOperator::Mul => {
                    RNWInteger::new(self.value * other_value)
                }
                BinaryOperator::Div => {
                    RNWFloat::new((self.value as f64) / (other_value as f64))
                }
                BinaryOperator::Pow => {
                    let value = (self.value as f64).powf(other_value as f64);
                    if (value % 1.0) == 0.0 {
                        RNWInteger::new(value as i64)
                    } else {
                        RNWFloat::new(value)
                    }
                }

                // Logic
                BinaryOperator::Eq => {
                    RNWBoolean::new(self.value == other_value)
                }
                BinaryOperator::NotEq => {
                    RNWBoolean::new(self.value != other_value)
                }
                BinaryOperator::Lt => {
                    RNWBoolean::new(self.value < other_value)
                }
                BinaryOperator::LtEq => {
                    RNWBoolean::new(self.value <= other_value)
                }
                BinaryOperator::Gt => {
                    RNWBoolean::new(self.value > other_value)
                }
                BinaryOperator::GtEq => {
                    RNWBoolean::new(self.value >= other_value)
                }
                _ => return None,
            })
        } else {
            None
        }
    }
}

pub(super) fn register() -> Rc<RefCell<RNWType>> {
    register_cast::<RNWInteger, RNWString>(|obj| {
        Ok(RNWString::new(obj.display()))
    });
    register_cast::<RNWInteger, RNWBoolean>(|obj| {
        Ok(RNWBoolean::new(obj.as_any().downcast_ref::<RNWInteger>().unwrap().value != 0))
    });

    RNWType::new::<RNWInteger>(RNWInteger::type_name())
}

