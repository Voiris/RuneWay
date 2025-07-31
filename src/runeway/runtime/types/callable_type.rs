use crate::assign_rnw_type_id;
use crate::runeway::builtins::types::RNWString;
use crate::runeway::core::errors::RWResult;
use crate::runeway::runtime::types::{
    register_cast, RNWObject, RNWObjectRef, RNWRegisteredNativeFunction, RNWRegisteredNativeMethod,
    RNWType, RNWTypeId,
};
use std::any::Any;
use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;

#[derive(Clone)]
pub struct RNWFunction {
    pub function: Rc<RNWRegisteredNativeFunction>,
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

    assign_rnw_type_id!();
}

impl RNWObject for RNWFunction {
    fn rnw_type_id(&self) -> RNWTypeId {
        Self::rnw_type_id()
    }
    fn type_name(&self) -> &'static str {
        Self::type_name()
    }
    fn display(&self) -> String {
        format!("<function '{}'>", self.function.name)
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

    fn call(&self, args: &[RNWObjectRef]) -> Option<RWResult<RNWObjectRef>> {
        Some(self.function.call(args))
    }
}

#[derive(Clone)]
pub struct RNWMethod {
    pub method: Rc<RNWRegisteredNativeMethod>,
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

    pub fn is_type_equals(other: &RNWObjectRef) -> bool {
        Self::rnw_type_id() == other.borrow().rnw_type_id()
    }

    assign_rnw_type_id!();
}

impl RNWObject for RNWMethod {
    fn rnw_type_id(&self) -> RNWTypeId {
        Self::rnw_type_id()
    }
    fn type_name(&self) -> &'static str {
        Self::type_name()
    }
    fn display(&self) -> String {
        format!("<method '{}'>", self.method.name)
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

    fn call(&self, args: &[RNWObjectRef]) -> Option<RWResult<RNWObjectRef>> {
        Some(
            self.method
                .call(args[0].clone(), args[1..].try_into().unwrap()),
        )
    }
}

pub fn register() -> Rc<RefCell<RNWType>> {
    register_cast(
        RNWFunction::rnw_type_id(),
        RNWString::rnw_type_id(),
        |obj| Ok(RNWString::new(obj.display())),
    );

    RNWType::new(RNWFunction::rnw_type_id(), RNWFunction::type_name())
}
