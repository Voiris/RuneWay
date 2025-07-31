use crate::assign_rnw_type_id;
use crate::runeway::builtins::types::{RNWBoolean, RNWString};
use crate::runeway::core::ast::operators::{BinaryOperator, UnaryOperator};
use crate::runeway::runtime::types::{register_cast, RNWObject, RNWObjectRef, RNWType, RNWTypeId};
use once_cell::unsync::Lazy;
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct RNWNullType;

thread_local! {
    static NULL_TYPE_NATIVE_FIELDS: Lazy<RefCell<HashMap<&'static str, RNWObjectRef>>> = Lazy::new(|| {
        let map = HashMap::new();

        RefCell::new(map)
    });
}

impl RNWNullType {
    pub fn new() -> RNWObjectRef {
        Rc::new(RefCell::new(Self {}))
    }

    pub fn is_type_equals(other: &RNWObjectRef) -> bool {
        Self::rnw_type_id() == other.borrow().rnw_type_id()
    }

    pub fn type_name() -> &'static str {
        "null_type"
    }

    assign_rnw_type_id!();
}

impl RNWObject for RNWNullType {
    fn rnw_type_id(&self) -> RNWTypeId {
        Self::rnw_type_id()
    }
    fn type_name(&self) -> &'static str {
        Self::type_name()
    }
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

    //noinspection DuplicatedCode
    fn get_attr(&self, name: &str) -> Option<RNWObjectRef> {
        NULL_TYPE_NATIVE_FIELDS.with(|m| m.borrow().get(name).cloned())
    }

    //noinspection DuplicatedCode
    fn binary_operation(
        &self,
        other: RNWObjectRef,
        binary_operator: BinaryOperator,
    ) -> Option<RNWObjectRef> {
        let result = if Self::is_type_equals(&other) {
            match binary_operator {
                BinaryOperator::Eq => RNWBoolean::new(true),
                BinaryOperator::NotEq => RNWBoolean::new(false),
                _ => return None,
            }
        } else {
            match binary_operator {
                BinaryOperator::Eq => RNWBoolean::new(false),
                BinaryOperator::NotEq => RNWBoolean::new(true),
                _ => return None,
            }
        };

        Some(result)
    }

    fn unary_operation(&self, unary_operator: UnaryOperator) -> Option<RNWObjectRef> {
        match unary_operator {
            UnaryOperator::Not => Some(RNWBoolean::new(true)),
            _ => None,
        }
    }
}

pub(super) fn register() -> Rc<RefCell<RNWType>> {
    register_cast(RNWNullType::rnw_type_id(), RNWString::rnw_type_id(), |_| {
        Ok(RNWString::new(RNWNullType::type_name()))
    });
    register_cast(
        RNWNullType::rnw_type_id(),
        RNWBoolean::rnw_type_id(),
        |_| Ok(RNWBoolean::new(false)),
    );

    RNWType::new(RNWNullType::rnw_type_id(), RNWNullType::type_name())
}
