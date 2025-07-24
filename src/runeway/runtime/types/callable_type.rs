use std::any::Any;
use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;
use crate::runeway::builtins::types::{RNWNullType, RNWString};
use crate::runeway::core::errors::RWResult;
use crate::runeway::runtime::types::{register_cast, RNWObject, RNWObjectRef, RNWRegisteredNativeFunction, RNWRegisteredNativeMethod, RNWType};

#[derive(Clone)]
pub struct RNWFunction {
    pub function: Rc<RNWRegisteredNativeFunction>
}

impl Debug for RNWFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display())
    }
}

impl RNWFunction {
    pub fn new(function: Rc<RNWRegisteredNativeFunction>) -> RNWObjectRef {
        Rc::new(RefCell::new(Self { function }))
    }

    pub fn type_name() -> &'static str {
        "function"
    }
}

impl RNWObject for RNWFunction {
    fn type_name(&self) -> &'static str { Self::type_name() }
    fn display(&self) -> String { format!("<function '{}'>", self.function.name) }
    fn value(&self) -> &dyn Any { self }
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn as_object(&self) -> &dyn RNWObject { self }
    fn call(&self, args: &[RNWObjectRef]) -> Option<RWResult<RNWObjectRef>> {
        Some(self.function.call(args))
    }
}

#[derive(Clone)]
pub struct RNWMethod {
    pub method: Rc<RNWRegisteredNativeMethod>
}

impl Debug for RNWMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display())
    }
}

impl RNWMethod {
    pub fn new(method: Rc<RNWRegisteredNativeMethod>) -> RNWObjectRef {
        Rc::new(RefCell::new(Self { method }))
    }

    pub fn type_name() -> &'static str {
        "method"
    }
}

impl RNWObject for RNWMethod {
    fn type_name(&self) -> &'static str { Self::type_name() }
    fn display(&self) -> String { format!("<method '{}'>", self.method.name) }
    fn value(&self) -> &dyn Any { self }
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn as_object(&self) -> &dyn RNWObject { self }
    fn call(&self, args: &[RNWObjectRef]) -> Option<RWResult<RNWObjectRef>> {

        Some(self.method.call(args[0].clone(), args[1..].try_into().unwrap()))
    }
}

pub fn register() -> Rc<RefCell<RNWType>> {
    register_cast::<RNWFunction, RNWString>(|obj| {
        Ok(RNWString::new(obj.display()))
    });

    RNWType::new::<RNWFunction>(RNWFunction::type_name())
}
