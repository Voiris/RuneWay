use crate::assign_rnw_type_id;
use crate::runeway::builtins::types::string::RNWString;
use crate::runeway::builtins::types::{RNWBoolean, RNWFloat};
use crate::runeway::core::ast::operators::BinaryOperator;
use crate::runeway::runtime::types::RNWObjectRef;
use crate::runeway::runtime::types::{register_cast, RNWObject, RNWType, RNWTypeId};
use once_cell::unsync::Lazy;
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct RNWInteger {
    pub value: i64,
}

thread_local! {
    static INTEGER_NATIVE_FIELDS: Lazy<RefCell<HashMap<&'static str, RNWObjectRef>>> = Lazy::new(|| {
        let map = HashMap::new();

        RefCell::new(map)
    });
}

// Integer implements
impl RNWInteger {
    pub fn new(value: i64) -> RNWObjectRef {
        Rc::new(RefCell::new(Self { value }))
    }

    pub fn is_type_equals(other: &RNWObjectRef) -> bool {
        Self::rnw_type_id() == other.borrow().rnw_type_id()
    }

    pub fn type_name() -> &'static str {
        "int"
    }

    assign_rnw_type_id!();
}

impl RNWObject for RNWInteger {
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
        INTEGER_NATIVE_FIELDS.with(|methods| methods.borrow().get(name).cloned())
    }

    fn binary_operation(
        &self,
        other: RNWObjectRef,
        binary_operator: BinaryOperator,
    ) -> Option<RNWObjectRef> {
        if let Some(other_value) = other.borrow().value().downcast_ref::<i64>().cloned() {
            Some(match binary_operator {
                // Arithmetic
                BinaryOperator::Add => RNWInteger::new(self.value.wrapping_add(other_value)),
                BinaryOperator::Sub => RNWInteger::new(self.value.wrapping_sub(other_value)),
                BinaryOperator::Mul => RNWInteger::new(self.value.wrapping_mul(other_value)),
                BinaryOperator::Div => RNWFloat::new((self.value as f64) / (other_value as f64)),
                BinaryOperator::Pow => {
                    let value = (self.value as f64).powf(other_value as f64);
                    if value.fract().abs() < f64::EPSILON {
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
    register_cast(RNWInteger::rnw_type_id(), RNWString::rnw_type_id(), |obj| {
        Ok(RNWString::new(obj.display()))
    });
    register_cast(
        RNWInteger::rnw_type_id(),
        RNWBoolean::rnw_type_id(),
        |obj| {
            Ok(RNWBoolean::new(
                obj.as_any().downcast_ref::<RNWInteger>().unwrap().value != 0,
            ))
        },
    );
    register_cast(RNWInteger::rnw_type_id(), RNWFloat::rnw_type_id(), |obj| {
        Ok(RNWFloat::new(
            obj.as_any().downcast_ref::<RNWInteger>().unwrap().value as f64,
        ))
    });

    let mut type_fields = HashMap::new();

    type_fields.insert("MIN".to_string(), RNWInteger::new(i64::MIN));
    type_fields.insert("MAX".to_string(), RNWInteger::new(i64::MAX));

    RNWType::new_with_fields(
        RNWInteger::rnw_type_id(),
        RNWInteger::type_name(),
        type_fields,
    )
}
