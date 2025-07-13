use std::any::Any;
use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;
use crate::runeway::core::ast::operators::{BinaryOperator, UnaryOperator};
use super::{RNWRegisteredNativeFunction, RNWRegisteredNativeMethod};

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

    /// param: name of method
    /// return: native or pseudo native method if it exists
    fn method(&self, _: &str) -> Option<RNWRegisteredNativeMethod> {
        None
    }

    /// param: name of function
    /// return: native or pseudo native function if it exists
    fn function(&self, _: &str) -> Option<RNWRegisteredNativeFunction> {
        None
    }

    fn binary_operation(&self, _: RNWObjectRef, _: BinaryOperator) -> Option<RNWObjectRef> {
        None
    }

    fn unary_operation(&self, _: UnaryOperator) -> Option<RNWObjectRef> {
        None
    }
}

pub type RNWObjectRef = Rc<RefCell<dyn RNWObject>>;
