use std::any::Any;
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;
use crate::runeway::executor::runtime::environment::EnvRef;
use crate::runeway::executor::runtime::types::{RNWObject, RNWObjectRef, RNWRegisteredNativeFunction};

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
}

impl RNWObject for RNWModule {
    fn type_name(&self) -> &'static str {
        Self::type_name()
    }

    fn display(&self) -> String {
        format!("<MODULE::{}>", self.path)
    }
    fn value(&self) -> &dyn Any { self }
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn as_object(&self) -> &dyn RNWObject { self }

    fn field(&self, name: &str) -> Option<RNWObjectRef> {
        match self.env.borrow().get_variable(name) {
            Ok(value) => Some(value),
            Err(err) => panic!("{}", err),
        }
    }

    fn function(&self, name: &str) -> Option<RNWRegisteredNativeFunction> {
        match self.env.borrow().get_function(name) {
            Ok(func) => Some((*func).clone()),
            Err(err) => panic!("{}", err),
        }
    }
}