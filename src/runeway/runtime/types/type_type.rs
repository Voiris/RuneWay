use crate::assign_rnw_type_id;
use crate::runeway::builtins::types::{RNWBoolean, RNWString};
use crate::runeway::core::ast::operators::BinaryOperator;
use crate::runeway::core::errors::RWResult;
use crate::runeway::runtime::types::base_type::RNWTypeId;
use crate::runeway::runtime::types::{register_cast, type_obj_from_id, RNWObject, RNWObjectRef};
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct RNWType {
    pub rnw_type_id: RNWTypeId,
    pub type_name: &'static str,
    fields: Option<Rc<HashMap<String, RNWObjectRef>>>,
}

impl RNWType {
    pub fn new(rnw_type_id: RNWTypeId, type_name: &'static str) -> Rc<RefCell<RNWType>> {
        Rc::new(RefCell::new(Self {
            rnw_type_id,
            type_name,
            fields: None,
        }))
    }

    pub fn new_with_fields(
        rnw_type_id: RNWTypeId,
        type_name: &'static str,
        fields: HashMap<String, RNWObjectRef>,
    ) -> Rc<RefCell<RNWType>> {
        Rc::new(RefCell::new(Self {
            rnw_type_id,
            type_name,
            fields: Some(Rc::new(fields)),
        }))
    }

    pub fn is_type_equals(other: &RNWObjectRef) -> bool {
        Self::rnw_type_id() == other.borrow().rnw_type_id()
    }

    pub fn type_name() -> &'static str {
        "type"
    }

    assign_rnw_type_id!();
}

impl RNWObject for RNWType {
    fn rnw_type_id(&self) -> RNWTypeId {
        Self::rnw_type_id()
    }
    fn type_name(&self) -> &'static str {
        Self::type_name()
    }
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

    fn get_attr(&self, name: &str) -> Option<RNWObjectRef> {
        if let Some(fields) = self.fields.clone() {
            fields.get(name).cloned()
        } else {
            None
        }
    }

    //noinspection DuplicatedCode
    fn binary_operation(
        &self,
        other: RNWObjectRef,
        binary_operator: BinaryOperator,
    ) -> Option<RNWObjectRef> {
        let result = if let Some(other) = other.borrow().as_any().downcast_ref::<Self>() {
            match binary_operator {
                BinaryOperator::Eq => RNWBoolean::new(self.rnw_type_id == other.rnw_type_id),
                BinaryOperator::NotEq => RNWBoolean::new(self.rnw_type_id != other.rnw_type_id),
                _ => return None,
            }
        } else {
            return None;
        };

        Some(result)
    }

    fn call(&self, args: &[RNWObjectRef]) -> Option<RWResult<RNWObjectRef>> {
        let obj = args.get(0).unwrap();

        let type_obj = {
            let obj_borrow = obj.borrow();
            let obj_type_id = obj_borrow.rnw_type_id();
            type_obj_from_id(obj_type_id)
        };

        Some(Ok(type_obj))
    }
}

pub fn register_type_class() -> Rc<RefCell<RNWType>> {
    register_cast(RNWType::rnw_type_id(), RNWString::rnw_type_id(), |obj| {
        Ok(RNWString::new(obj.display()))
    });

    RNWType::new(RNWType::rnw_type_id(), RNWType::type_name())
}
