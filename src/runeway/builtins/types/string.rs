use std::any::{Any, TypeId};
use std::cell::{RefCell};
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::rc::Rc;
use once_cell::sync::Lazy;
use crate::runeway::core::ast::operators::BinaryOperator;
use crate::runeway::runtime::types::{RNWObject, RNWRegisteredNativeMethod, RNWObjectRef, RNWMethod, register_cast, RNWType};
use crate::runeway::builtins::types::{RNWBoolean, RNWDict, RNWFloat, RNWInteger, RNWList, RNWNullType};
use crate::runeway::core::errors::RWResult;

#[derive(Debug, Clone)]
pub struct RNWString {
    pub value: String,
}

fn native_string_to_int(this: RNWObjectRef, _: &[RNWObjectRef]) -> RWResult<RNWObjectRef> {
    Ok(RNWInteger::new(
            (*this).borrow().value().downcast_ref::<String>().unwrap().parse::<i64>().unwrap()
    ))
}

fn native_string_to_float(this: RNWObjectRef, _: &[RNWObjectRef]) -> RWResult<RNWObjectRef> {
    Ok(RNWFloat::new(
        (*this).borrow().value().downcast_ref::<String>().unwrap().parse::<f64>().unwrap()
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
            vec![TypeId::of::<RNWString>()]
        )));
        map.insert("to_float", RNWMethod::new(RNWRegisteredNativeMethod::new(
            "string.to_float".to_string(),
            Rc::new(native_string_to_float),
            vec![TypeId::of::<RNWString>()]
        )));
        map.insert("to_list", RNWMethod::new(RNWRegisteredNativeMethod::new(
            "string.to_list".to_string(),
            Rc::new(native_string_to_list),
            vec![TypeId::of::<RNWString>()]
        )));

        RefCell::new(map)
    });
}

impl RNWString {
    pub fn new(value: impl ToString) -> RNWObjectRef {
        Rc::new(RefCell::new(Self { value: value.to_string() }))
    }

    pub fn type_name() -> &'static str { "string" }

    pub fn is_type_equals(other: RNWObjectRef) -> bool {
        std::any::TypeId::of::<Self>() == other.borrow().as_any().type_id()
    }
}

impl RNWObject for RNWString {
    fn type_name(&self) -> &'static str { Self::type_name() }

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
    fn as_object(&self) -> &dyn RNWObject {
        self
    }

    //noinspection DuplicatedCode
    fn field(&self, name: &str) -> Option<RNWObjectRef> {
        STRING_NATIVE_FIELDS.with(
            |methods| methods.borrow().get(name).cloned()
        )
    }

    fn binary_operation(&self, other: RNWObjectRef, binary_operator: BinaryOperator)-> Option<RNWObjectRef> {
        match (binary_operator, other.borrow().type_name()) {
            (BinaryOperator::Add, "string") => {
                Some(RNWString::new(format!(
                    "{}{}",
                    self.value,
                    other.borrow().value().downcast_ref::<String>().unwrap()
                )))
            }
            (BinaryOperator::Eq, "string") => {
                Some(RNWBoolean::new(
                    self.value == *other
                        .borrow().value().downcast_ref::<String>().unwrap()
                ))
            }
            (BinaryOperator::NotEq, "string") => {
                Some(RNWBoolean::new(
                    self.value != *other
                        .borrow().value().downcast_ref::<String>().unwrap()
                ))
            }
            _ => None
        }
    }
}

pub(super) fn register() -> Rc<RefCell<RNWType>> {
    register_cast::<RNWString, RNWInteger>(|obj| {
        Ok(RNWInteger::new(
            obj.value().downcast_ref::<String>().unwrap().parse::<i64>()?
        ))
    });
    register_cast::<RNWString, RNWFloat>(|obj| {
        Ok(RNWFloat::new(
            obj.value().downcast_ref::<String>().unwrap().parse::<f64>()?
        ))
    });
    register_cast::<RNWString, RNWList>(|obj| {
        let mut vec = Vec::new();
        let string = obj.value().downcast_ref::<String>().unwrap();
        for c in string.chars() {
            vec.push(RNWString::new(c));
        }
        Ok(RNWList::new(&vec))
    });
    register_cast::<RNWString, RNWBoolean>(|obj| {
        Ok(RNWBoolean::new(!obj.as_any().downcast_ref::<RNWString>().unwrap().value.is_empty()))
    });

    RNWType::new::<RNWString>(RNWString::type_name())
}
