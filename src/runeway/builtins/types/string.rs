use crate::assign_rnw_type_id;
use crate::runeway::builtins::types::{RNWBoolean, RNWFloat, RNWInteger, RNWList};
use crate::runeway::core::ast::operators::BinaryOperator;
use crate::runeway::core::errors::RWResult;
use crate::runeway::runtime::types::{
    register_cast, RNWMethod, RNWObject, RNWObjectRef, RNWRegisteredNativeMethod, RNWType,
    RNWTypeId,
};
use once_cell::sync::Lazy;
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct RNWString {
    pub value: String,
}

fn native_string_to_int(this: RNWObjectRef, _: &[RNWObjectRef]) -> RWResult<RNWObjectRef> {
    Ok(RNWInteger::new(
        (*this)
            .borrow()
            .value()
            .downcast_ref::<String>()
            .unwrap()
            .parse::<i64>()?,
    ))
}

fn native_string_to_float(this: RNWObjectRef, _: &[RNWObjectRef]) -> RWResult<RNWObjectRef> {
    Ok(RNWFloat::new(
        (*this)
            .borrow()
            .value()
            .downcast_ref::<String>()
            .unwrap()
            .parse::<f64>()?,
    ))
}

fn native_string_to_list(this: RNWObjectRef, _: &[RNWObjectRef]) -> RWResult<RNWObjectRef> {
    let mut vec = Vec::new();
    let borrow = this.borrow();
    let string = borrow.value().downcast_ref::<String>().unwrap().clone();
    for c in string.chars() {
        vec.push(RNWString::new(c));
    }
    Ok(RNWList::new(&vec))
}

thread_local! {
    static STRING_NATIVE_FIELDS: Lazy<RefCell<HashMap<&'static str, RNWObjectRef>>> = Lazy::new(|| {
        let mut map = HashMap::new();

        map.insert("to_int", RNWMethod::new(RNWRegisteredNativeMethod::new(
            "string.to_int".to_string(),
            Rc::new(native_string_to_int),
            vec![RNWString::rnw_type_id()]
        )));
        map.insert("to_float", RNWMethod::new(RNWRegisteredNativeMethod::new(
            "string.to_float".to_string(),
            Rc::new(native_string_to_float),
            vec![RNWString::rnw_type_id()]
        )));
        map.insert("to_list", RNWMethod::new(RNWRegisteredNativeMethod::new(
            "string.to_list".to_string(),
            Rc::new(native_string_to_list),
            vec![RNWString::rnw_type_id()]
        )));

        RefCell::new(map)
    });
}

impl RNWString {
    pub fn new(value: impl ToString) -> RNWObjectRef {
        Rc::new(RefCell::new(Self {
            value: value.to_string(),
        }))
    }

    pub fn type_name() -> &'static str {
        "string"
    }

    pub fn is_type_equals(other: &RNWObjectRef) -> bool {
        Self::rnw_type_id() == other.borrow().rnw_type_id()
    }

    assign_rnw_type_id!();
}

impl RNWObject for RNWString {
    fn rnw_type_id(&self) -> RNWTypeId {
        Self::rnw_type_id()
    }
    fn type_name(&self) -> &'static str {
        Self::type_name()
    }

    fn display(&self) -> String {
        format!("\"{}\"", self.value)
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

    //noinspection DuplicatedCode
    fn get_attr(&self, name: &str) -> Option<RNWObjectRef> {
        STRING_NATIVE_FIELDS.with(|methods| methods.borrow().get(name).cloned())
    }

    fn binary_operation(
        &self,
        other: RNWObjectRef,
        binary_operator: BinaryOperator,
    ) -> Option<RNWObjectRef> {
        match (binary_operator, other.borrow().type_name()) {
            (BinaryOperator::Add, "string") => Some(RNWString::new(format!(
                "{}{}",
                self.value,
                other.borrow().value().downcast_ref::<String>().unwrap()
            ))),
            (BinaryOperator::Eq, "string") => Some(RNWBoolean::new(
                self.value == *other.borrow().value().downcast_ref::<String>().unwrap(),
            )),
            (BinaryOperator::NotEq, "string") => Some(RNWBoolean::new(
                self.value != *other.borrow().value().downcast_ref::<String>().unwrap(),
            )),
            _ => None,
        }
    }
}

pub(super) fn register() -> Rc<RefCell<RNWType>> {
    register_cast(RNWString::rnw_type_id(), RNWInteger::rnw_type_id(), |obj| {
        Ok(RNWInteger::new(
            obj.value()
                .downcast_ref::<String>()
                .unwrap()
                .parse::<i64>()?,
        ))
    });
    register_cast(RNWString::rnw_type_id(), RNWFloat::rnw_type_id(), |obj| {
        Ok(RNWFloat::new(
            obj.value()
                .downcast_ref::<String>()
                .unwrap()
                .parse::<f64>()?,
        ))
    });
    register_cast(RNWString::rnw_type_id(), RNWList::rnw_type_id(), |obj| {
        let mut vec = Vec::new();
        let string = obj.value().downcast_ref::<String>().unwrap();
        for c in string.chars() {
            vec.push(RNWString::new(c));
        }
        Ok(RNWList::new(&vec))
    });
    register_cast(RNWString::rnw_type_id(), RNWBoolean::rnw_type_id(), |obj| {
        Ok(RNWBoolean::new(
            !obj.as_any()
                .downcast_ref::<RNWString>()
                .unwrap()
                .value
                .is_empty(),
        ))
    });

    RNWType::new(RNWString::rnw_type_id(), RNWString::type_name())
}
