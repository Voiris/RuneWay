use crate::assign_rnw_type_id;
use crate::runeway::builtins::types::{RNWBoolean, RNWInteger, RNWList, RNWString};
use crate::runeway::core::errors::{RWResult, RuneWayError, RuneWayErrorKind};
use crate::runeway::runtime::types::{
    register_cast, RNWMethod, RNWObject, RNWObjectRef, RNWRegisteredNativeMethod, RNWType,
    RNWTypeId,
};
use once_cell::unsync::Lazy;
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct RNWTuple {
    pub value: Vec<RNWObjectRef>,
}

fn native_tuple_len(this: RNWObjectRef, _: &[RNWObjectRef]) -> RWResult<RNWObjectRef> {
    let binding = this.borrow();
    let tuple = binding.as_any().downcast_ref::<RNWTuple>().unwrap();
    Ok(RNWInteger::new(tuple.value.len() as i64))
}

fn native_tuple_slice(this: RNWObjectRef, args: &[RNWObjectRef]) -> RWResult<RNWObjectRef> {
    let binding = args.get(0).unwrap().borrow();
    let index = binding.value().downcast_ref::<i64>().unwrap();

    let binding = this.borrow();
    let tuple = binding.as_any().downcast_ref::<RNWTuple>().unwrap();
    tuple.value.get(*index as usize).cloned().ok_or(
        RuneWayError::new(RuneWayErrorKind::error_with_code("IndexError"))
            .with_message("Index out of bounds"),
    )
}

// TODO: tuple_iter

thread_local! {
    static TUPLE_NATIVE_FIELDS: Lazy<RefCell<HashMap<&'static str, RNWObjectRef>>> = Lazy::new(|| {
        let mut map = HashMap::new();

        map.insert("len", RNWMethod::new(RNWRegisteredNativeMethod::new(
            "tuple.len".to_string(),
            Rc::new(native_tuple_len),
            vec![RNWTuple::rnw_type_id()]
        )));
        map.insert("slice", RNWMethod::new(RNWRegisteredNativeMethod::new(
            "tuple.slice".to_string(),
            Rc::new(native_tuple_slice),
            vec![RNWTuple::rnw_type_id(), RNWInteger::rnw_type_id()]
        )));

        RefCell::new(map)
    });
}

impl RNWTuple {
    pub fn new(vec: &Vec<RNWObjectRef>) -> RNWObjectRef {
        let mut value = Vec::with_capacity(vec.len());
        value.extend_from_slice(vec);
        Rc::new(RefCell::new(Self { value }))
    }

    pub fn type_name() -> &'static str {
        "tuple"
    }

    pub fn is_type_equals(other: &RNWObjectRef) -> bool {
        Self::rnw_type_id() == other.borrow().rnw_type_id()
    }

    assign_rnw_type_id!();
}

impl RNWObject for RNWTuple {
    fn rnw_type_id(&self) -> RNWTypeId {
        Self::rnw_type_id()
    }
    fn type_name(&self) -> &'static str {
        Self::type_name()
    }
    fn display(&self) -> String {
        format!(
            "({})",
            self.value
                .iter()
                .map(|v| v.borrow().display())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
    fn value(&self) -> &dyn Any {
        &self.value
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_attr(&self, name: &str) -> Option<RNWObjectRef> {
        TUPLE_NATIVE_FIELDS.with(|methods| methods.borrow().get(name).cloned())
    }
}

pub(super) fn register() -> Rc<RefCell<RNWType>> {
    register_cast(RNWTuple::rnw_type_id(), RNWString::rnw_type_id(), |obj| {
        Ok(RNWString::new(obj.display()))
    });
    register_cast(RNWTuple::rnw_type_id(), RNWList::rnw_type_id(), |obj| {
        Ok(RNWList::new(
            &obj.value()
                .downcast_ref::<Vec<RNWObjectRef>>()
                .unwrap()
                .clone(),
        ))
    });
    register_cast(RNWTuple::rnw_type_id(), RNWBoolean::rnw_type_id(), |obj| {
        Ok(RNWBoolean::new(
            !obj.as_any()
                .downcast_ref::<RNWTuple>()
                .unwrap()
                .value
                .is_empty(),
        ))
    });

    RNWType::new(RNWTuple::rnw_type_id(), RNWTuple::type_name())
}
