use crate::assign_rnw_type_id;
use crate::runeway::builtins::types::RNWString;
use crate::runeway::runtime::environment::EnvRef;
use crate::runeway::runtime::types::{register_cast, RNWObject, RNWObjectRef, RNWType, RNWTypeId};
use std::any::Any;
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

#[derive(Clone)]
pub struct RNWModule {
    path: String,
    env: EnvRef,
}

impl fmt::Debug for RNWModule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RNWModule")
            .field("path", &self.path)
            .field("env", &"<EnvRef>")
            .finish()
    }
}

impl RNWModule {
    pub fn new(path: String, env_ref: EnvRef) -> Rc<RefCell<RNWModule>> {
        Rc::new(RefCell::new(Self { path, env: env_ref }))
    }

    pub fn type_name() -> &'static str {
        "module"
    }

    pub fn is_type_equals(other: &RNWObjectRef) -> bool {
        Self::rnw_type_id() == other.borrow().rnw_type_id()
    }

    assign_rnw_type_id!();
}

impl RNWObject for RNWModule {
    fn rnw_type_id(&self) -> RNWTypeId {
        Self::rnw_type_id()
    }
    fn type_name(&self) -> &'static str {
        Self::type_name()
    }

    fn display(&self) -> String {
        format!("<MODULE::{}>", self.path)
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

    fn get_attr(&self, name: &str) -> Option<RNWObjectRef> {
        self.env.borrow().get_variable(name)
    }
}

pub fn register() -> Rc<RefCell<RNWType>> {
    register_cast(RNWModule::rnw_type_id(), RNWString::rnw_type_id(), |obj| {
        Ok(RNWString::new(obj.display()))
    });

    RNWType::new(RNWModule::rnw_type_id(), RNWModule::type_name())
}
