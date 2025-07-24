use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::Deref;
use std::rc::Rc;
use crate::runeway::core::ast::operators::{BinaryOperator, UnaryOperator};
use crate::runeway::core::errors::RWResult;
use crate::runeway::runtime::types::casting::{cast_to, CastTo};
use super::{cast_to_type_id, RNWRegisteredNativeFunction, RNWRegisteredNativeMethod};

pub trait RNWObject: Debug {
    fn type_name(&self) -> &'static str;
    fn display(&self) -> String;
    fn value(&self) -> &dyn Any;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn as_object(&self) -> &dyn RNWObject;

    /// param: name of field
    /// return: value of field if it exists
    fn field(&self, _: &str) -> Option<RNWObjectRef> {
        None
    }

    fn binary_operation(&self, _: RNWObjectRef, _: BinaryOperator) -> Option<RNWObjectRef> {
        None
    }

    fn unary_operation(&self, _: UnaryOperator) -> Option<RNWObjectRef> {
        None
    }

    fn call(&self, _: &[RNWObjectRef]) -> Option<RWResult<RNWObjectRef>> {
        None
    }
}

pub type RNWObjectRef = Rc<RefCell<dyn RNWObject>>;

impl CastTo for RNWObjectRef {
    fn cast_to<T: 'static>(&self) -> RWResult<RNWObjectRef> {
        cast_to::<T>(&self)
    }
    fn cast_to_type_id(&self, target_type_id: TypeId) -> RWResult<RNWObjectRef> {
        cast_to_type_id(self, target_type_id)
    }
}
