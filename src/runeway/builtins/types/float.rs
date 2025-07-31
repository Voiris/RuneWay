use crate::assign_rnw_type_id;
use crate::runeway::builtins::types::string::RNWString;
use crate::runeway::builtins::types::{RNWBoolean, RNWInteger};
use crate::runeway::core::ast::operators::BinaryOperator;
use crate::runeway::runtime::types::RNWObject;
use crate::runeway::runtime::types::{register_cast, RNWObjectRef, RNWType, RNWTypeId};
use once_cell::sync::Lazy;
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct RNWFloat {
    pub value: f64,
}

thread_local! {
    static FLOAT_NATIVE_FIELDS: Lazy<RefCell<HashMap<&'static str, RNWObjectRef>>> = Lazy::new(|| {
        let map = HashMap::new();

        RefCell::new(map)
    });
}

// Integer implements
impl RNWFloat {
    pub fn new(value: f64) -> RNWObjectRef {
        Rc::new(RefCell::new(Self { value }))
    }

    pub fn is_type_equals(other: &RNWObjectRef) -> bool {
        Self::rnw_type_id() == other.borrow().rnw_type_id()
    }

    pub fn type_name() -> &'static str {
        "float"
    }

    assign_rnw_type_id!();
}

impl RNWObject for RNWFloat {
    fn rnw_type_id(&self) -> RNWTypeId {
        Self::rnw_type_id()
    }
    fn type_name(&self) -> &'static str {
        Self::type_name()
    }
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

    fn get_attr(&self, name: &str) -> Option<RNWObjectRef> {
        FLOAT_NATIVE_FIELDS.with(|methods| methods.borrow().get(name).cloned())
    }

    fn binary_operation(
        &self,
        other: RNWObjectRef,
        binary_operator: BinaryOperator,
    ) -> Option<RNWObjectRef> {
        if let Some(other_value) = other.borrow().value().downcast_ref::<f64>().cloned() {
            Some(match binary_operator {
                // Arithmetic
                BinaryOperator::Add => RNWFloat::new(self.value + other_value),
                BinaryOperator::Sub => RNWFloat::new(self.value - other_value),
                BinaryOperator::Mul => RNWFloat::new(self.value * other_value),
                BinaryOperator::Div => RNWFloat::new((self.value) / (other_value)),
                BinaryOperator::Pow => {
                    let value = self.value.powf(other_value);
                    if (value % 1.0) == 0.0 {
                        RNWInteger::new(value as i64)
                    } else {
                        RNWFloat::new(value)
                    }
                }

                // Logic
                BinaryOperator::Eq => RNWBoolean::new(self.value == other_value),
                BinaryOperator::NotEq => RNWBoolean::new(self.value != other_value),
                BinaryOperator::Lt => RNWBoolean::new(self.value < other_value),
                BinaryOperator::LtEq => RNWBoolean::new(self.value <= other_value),
                BinaryOperator::Gt => RNWBoolean::new(self.value > other_value),
                BinaryOperator::GtEq => RNWBoolean::new(self.value >= other_value),
                _ => return None,
            })
        } else {
            None
        }
    }
}

pub(super) fn register() -> Rc<RefCell<RNWType>> {
    register_cast(RNWFloat::rnw_type_id(), RNWString::rnw_type_id(), |obj| {
        Ok(RNWString::new(obj.display()))
    });
    register_cast(RNWFloat::rnw_type_id(), RNWBoolean::rnw_type_id(), |obj| {
        Ok(RNWBoolean::new(
            obj.as_any().downcast_ref::<RNWFloat>().unwrap().value != 0.0,
        ))
    });

    let mut type_fields = HashMap::new();

    type_fields.insert("NaN".to_string(), RNWFloat::new(f64::NAN));
    type_fields.insert("inf".to_string(), RNWFloat::new(f64::INFINITY));
    type_fields.insert("MIN".to_string(), RNWFloat::new(f64::MIN));
    type_fields.insert("MAX".to_string(), RNWFloat::new(f64::MAX));

    RNWType::new_with_fields(RNWFloat::rnw_type_id(), RNWFloat::type_name(), type_fields)
}
