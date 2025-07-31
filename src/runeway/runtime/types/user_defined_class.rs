use crate::runeway::core::errors::RWResult;
use crate::runeway::runtime::environment::{EnvRef, Environment};
use crate::runeway::runtime::types::{gen_rnw_type_id, RNWObject, RNWObjectRef, RNWTypeId};
use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct UserDefinedClass {
    pub rnw_type_id: RNWTypeId,
    pub name: &'static str,
    pub fields: EnvRef,
    pub is_instance: bool,
}

impl UserDefinedClass {
    pub fn new(name: &String) -> RNWObjectRef {
        Rc::new(RefCell::new(Self {
            rnw_type_id: gen_rnw_type_id(),
            name: Box::leak(name.clone().into_boxed_str()),
            fields: Environment::new_global(),
            is_instance: false,
        }))
    }

    pub fn new_instance(&self) -> RNWObjectRef {
        Rc::new(RefCell::new(UserDefinedClass {
            rnw_type_id: self.rnw_type_id,
            name: self.name,
            fields: Environment::new_enclosed(self.fields.clone()),
            is_instance: true,
        }))
    }

    pub fn is_type_equals(other: &RNWObjectRef) -> bool {
        TypeId::of::<Self>() == other.borrow().as_any().type_id()
    }
}

impl RNWObject for UserDefinedClass {
    fn rnw_type_id(&self) -> RNWTypeId {
        self.rnw_type_id
    }

    fn type_name(&self) -> &'static str {
        self.name
    }

    fn display(&self) -> String {
        format!("<{}#{}>", self.name, self.rnw_type_id)
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
        self.fields.borrow().get_variable(name)
    }

    fn set_attr(&mut self, name: &str, value: RNWObjectRef) -> RWResult<()> {
        let mut borrow = self.fields.borrow_mut();
        borrow.define_variable(name.to_string(), value);
        Ok(())
    }
}
