use std::any::{Any, TypeId};
use std::cell::{RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use once_cell::sync::Lazy;
use crate::runeway::core::ast::operators::BinaryOperator;
use crate::runeway::executor::runtime::types::{
    RNWObject, RNWRegisteredNativeMethod, RNWObjectRef};
use crate::runeway::builtins::types::{RNWFloat, RNWInteger};

#[derive(Debug, Clone)]
pub struct RNWString {
    pub value: String,
}

fn native_string_to_int(this: RNWObjectRef, _: &[RNWObjectRef]) -> RNWObjectRef {
    RNWInteger::new(
            (*this).borrow().value().downcast_ref::<String>().unwrap().parse::<i64>().unwrap()
    )
}

fn native_string_to_float(this: RNWObjectRef, _: &[RNWObjectRef]) -> RNWObjectRef {
    RNWFloat::new(
        (*this).borrow().value().downcast_ref::<String>().unwrap().parse::<f64>().unwrap()
    )
}

fn native_string_to_string(this: RNWObjectRef, _: &[RNWObjectRef]) -> RNWObjectRef {
    this.clone()
}

thread_local! {
    static STRING_NATIVE_METHODS: Lazy<RefCell<HashMap<&'static str, RNWRegisteredNativeMethod>>> = Lazy::new(|| {
        let mut map = HashMap::new();

        map.insert("to_int", RNWRegisteredNativeMethod::new(
            "string.to_int".to_string(),
            Rc::new(native_string_to_int),
            vec![TypeId::of::<RNWString>()]
        ));
        map.insert("to_float", RNWRegisteredNativeMethod::new(
            "string.to_float".to_string(),
            Rc::new(native_string_to_float),
            vec![TypeId::of::<RNWString>()]
        ));
        map.insert("to_string", RNWRegisteredNativeMethod::new(
            "string.to_string".to_string(),
            Rc::new(native_string_to_string),
            vec![TypeId::of::<RNWString>()]
        ));

        RefCell::new(map)
    });
}

impl RNWString {
    pub fn new(value: String) -> RNWObjectRef {
        Rc::new(RefCell::new(Self { value }))
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

    fn method(&self, name: &str) -> Option<RNWRegisteredNativeMethod> {
        STRING_NATIVE_METHODS.with(
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
            _ => None
        }
    }
}
