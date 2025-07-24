use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use once_cell::unsync::Lazy;
use crate::runeway::builtins::types::{RNWBoolean, RNWDict, RNWList, RNWString};
use crate::runeway::core::ast::operators::{BinaryOperator, UnaryOperator};
use crate::runeway::core::errors::{RWResult, RuneWayError};
use crate::runeway::runtime::types::{register_cast, RNWMethod, RNWObject, RNWObjectRef, RNWRegisteredNativeMethod, RNWType};

#[derive(Debug, Clone)]
pub struct RNWNullType;

thread_local! {
    static NULL_TYPE_NATIVE_FIELDS: Lazy<RefCell<HashMap<&'static str, RNWObjectRef>>> = Lazy::new(|| {
        let mut map = HashMap::new();

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

    pub fn type_name() -> &'static str { "null_type" }
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
    fn field(&self, name: &str) -> Option<RNWObjectRef> {
        NULL_TYPE_NATIVE_FIELDS.with(|m| {
            m.borrow().get(name).cloned()
        })
    }

    fn unary_operation(&self, unary_operator: UnaryOperator) -> Option<RNWObjectRef> {
        match unary_operator {
            UnaryOperator::Not => Some(RNWBoolean::new(true)),
            _ => None,
        }
    }

    //noinspection DuplicatedCode
    fn binary_operation(&self, other: RNWObjectRef, binary_operator: BinaryOperator) -> Option<RNWObjectRef> {
        let result = if Self::is_type_equals(&other) {
            match binary_operator {
                BinaryOperator::Eq => {
                    RNWBoolean::new(true)
                }
                BinaryOperator::NotEq => {
                    RNWBoolean::new(false)
                }
                _ => return None
            }
        } else {
            match binary_operator {
                BinaryOperator::Eq => {
                    RNWBoolean::new(false)
                }
                BinaryOperator::NotEq => {
                    RNWBoolean::new(true)
                }
                _ => return None
            }
        };

        Some(result)
    }
}

pub(super) fn register() -> Rc<RefCell<RNWType>> {
    register_cast::<RNWNullType, RNWString>(|obj| {
        Ok(RNWString::new(obj.display()))
    });
    register_cast::<RNWNullType, RNWBoolean>(|obj| {
        Ok(RNWBoolean::new(false))
    });

    RNWType::new::<RNWNullType>(RNWNullType::type_name())
}
