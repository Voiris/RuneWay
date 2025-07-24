use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use once_cell::unsync::Lazy;
use crate::runeway::builtins::types::{RNWBoolean, RNWDict, RNWNullType, RNWString};
use crate::runeway::core::ast::operators::BinaryOperator;
use crate::runeway::core::errors::{RWResult, RuneWayError};
use crate::runeway::runtime::types::{register_cast, type_obj_from_id, RNWMethod, RNWObject, RNWObjectRef, RNWRegisteredNativeMethod};

#[derive(Debug, Clone)]
pub struct RNWType {
    pub type_id: TypeId,
    pub type_name: &'static str,
}

impl RNWType {
    pub fn new<T: 'static>(type_name: &'static str) -> Rc<RefCell<RNWType>> {
        Rc::new(RefCell::new(Self {
            type_id: TypeId::of::<T>(),
            type_name
        }))
    }

    pub fn is_type_equals(other: &RNWObjectRef) -> bool {
        TypeId::of::<Self>() == other.borrow().as_any().type_id()
    }

    pub fn type_name() -> &'static str { "type" }
}

impl RNWObject for RNWType {
    fn type_name(&self) -> &'static str { Self::type_name() }
    fn display(&self) -> String {
        format!("<{} {}>", Self::type_name(), self.type_name)
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
    fn binary_operation(&self, other: RNWObjectRef, binary_operator: BinaryOperator) -> Option<RNWObjectRef> {
        let result = if let Some(other) = other.borrow().as_any().downcast_ref::<Self>() {
            match binary_operator {
                BinaryOperator::Eq => {
                    RNWBoolean::new(self.type_id == other.type_id)
                }
                BinaryOperator::NotEq => {
                    RNWBoolean::new(self.type_id != other.type_id)
                }
                _ => return None
            }
        } else {
            return None
        };

        Some(result)
    }

    fn call(&self, args: &[RNWObjectRef]) -> Option<RWResult<RNWObjectRef>> {
        let obj = args.get(0).unwrap();

        let type_obj = {
            let obj_borrow = obj.borrow();
            let obj_type_id = obj_borrow.as_any().type_id();
            type_obj_from_id(&obj_type_id)
        };

        Some(Ok(type_obj))
    }
}

pub fn register_type_class() -> Rc<RefCell<RNWType>> {
    register_cast::<RNWType, RNWString>(|obj| {
        Ok(RNWString::new(obj.display()))
    });

    RNWType::new::<RNWType>(RNWType::type_name())
}
