use crate::assign_rnw_type_id;
use crate::runeway::builtins::types::RNWString;
use crate::runeway::core::errors::RWResult;
use crate::runeway::runtime::types::{
    register_cast, RNWMethod, RNWObject, RNWObjectRef, RNWRegisteredNativeMethod, RNWType,
    RNWTypeId,
};
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct RNWBox {
    fields: HashMap<String, RNWObjectRef>,
}

impl RNWBox {
    pub fn new() -> RNWObjectRef {
        Rc::new(RefCell::new(Self {
            fields: HashMap::new(),
        }))
    }
    pub fn is_type_equals(other: &RNWObjectRef) -> bool {
        Self::rnw_type_id() == other.borrow().rnw_type_id()
    }

    pub fn type_name() -> &'static str {
        "Box"
    }

    assign_rnw_type_id!();
}

impl RNWObject for RNWBox {
    fn rnw_type_id(&self) -> RNWTypeId {
        Self::rnw_type_id()
    }
    fn type_name(&self) -> &'static str {
        Self::type_name()
    }
    fn display(&self) -> String {
        format!("<Box fields_count={}>", self.fields.len())
    }
    fn value(&self) -> &dyn Any {
        &self.fields
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_attr(&self, name: &str) -> Option<RNWObjectRef> {
        self.fields.get(name).cloned()
    }
    fn set_attr(&mut self, name: &str, value: RNWObjectRef) -> RWResult<()> {
        self.fields.insert(name.to_owned(), value);
        Ok(())
    }
}

fn native_type_box_new(_: RNWObjectRef, _: &[RNWObjectRef]) -> RWResult<RNWObjectRef> {
    Ok(RNWBox::new())
}

pub(super) fn register() -> Rc<RefCell<RNWType>> {
    register_cast(RNWBox::rnw_type_id(), RNWString::rnw_type_id(), |obj| {
        Ok(RNWString::new(obj.display()))
    });

    let mut type_fields = HashMap::new();

    type_fields.insert(
        "new".to_string(),
        RNWMethod::new(RNWRegisteredNativeMethod::new(
            "Box.new".to_string(),
            Rc::new(native_type_box_new),
            vec![RNWType::rnw_type_id()],
        )),
    );

    RNWType::new_with_fields(RNWBox::rnw_type_id(), RNWBox::type_name(), type_fields)
}
