use crate::assign_rnw_type_id;
use crate::runeway::builtins::types::RNWString;
use crate::runeway::core::ast::operators::{BinaryOperator, UnaryOperator};
use crate::runeway::runtime::types::{register_cast, RNWObject, RNWObjectRef, RNWType, RNWTypeId};
use once_cell::sync::Lazy;
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct RNWBoolean {
    value: bool,
}

thread_local! {
    static BOOLEAN_NATIVE_FIELDS: Lazy<RefCell<HashMap<&'static str, RNWObjectRef>>> = Lazy::new(|| {
        let map = HashMap::new();

        RefCell::new(map)
    });
}

impl RNWBoolean {
    pub fn new(value: bool) -> RNWObjectRef {
        Rc::new(RefCell::new(Self { value }))
    }

    pub fn type_name() -> &'static str {
        "bool"
    }

    pub fn is_type_equals(other: &RNWObjectRef) -> bool {
        Self::rnw_type_id() == other.borrow().rnw_type_id()
    }

    assign_rnw_type_id!();
}

impl RNWObject for RNWBoolean {
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

    //noinspection DuplicatedCode
    fn get_attr(&self, name: &str) -> Option<RNWObjectRef> {
        BOOLEAN_NATIVE_FIELDS.with(|methods| methods.borrow().get(name).cloned())
    }

    fn binary_operation(
        &self,
        other: RNWObjectRef,
        binary_operator: BinaryOperator,
    ) -> Option<RNWObjectRef> {
        let result = if let Some(&other_value) = other.borrow().value().downcast_ref::<bool>() {
            match binary_operator {
                BinaryOperator::Or => RNWBoolean::new(self.value || other_value),
                BinaryOperator::And => RNWBoolean::new(self.value && other_value),
                BinaryOperator::Eq => RNWBoolean::new(self.value == other_value),
                BinaryOperator::NotEq => RNWBoolean::new(self.value != other_value),
                _ => return None,
            }
        } else {
            return None;
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

pub(super) fn register() -> Rc<RefCell<RNWType>> {
    register_cast(RNWBoolean::rnw_type_id(), RNWString::rnw_type_id(), |obj| {
        Ok(RNWString::new(obj.display()))
    });

    RNWType::new(RNWBoolean::rnw_type_id(), RNWBoolean::type_name())
}
