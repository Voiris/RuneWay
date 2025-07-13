use std::any::{Any, TypeId};
use std::rc::Rc;
use crate::runeway::executor::runtime::types::base_type::RNWObjectRef;
use super::type_name_from_id;
use super::RNWObject;

pub type RNWNativeFunction = Rc<dyn Fn(&[RNWObjectRef]) -> RNWObjectRef>;
pub type RNWNativeMethod = Rc<dyn Fn(RNWObjectRef, &[RNWObjectRef]) -> RNWObjectRef>;

#[derive(Clone)]
pub struct RNWRegisteredNativeFunction {
    pub name: String,
    function: RNWNativeFunction,
    params: Vec<TypeId>,
}

impl RNWRegisteredNativeFunction {
    pub fn new(name: String, function: RNWNativeFunction, params: Vec<TypeId>) -> Self {
        Self { name, function, params }
    }

    pub fn call(&self, args: &[RNWObjectRef]) -> RNWObjectRef {
        match check_params(&self.params, args.iter().map(
            |v| v.borrow().as_any().type_id()
        ).collect(), &self.name) {
            Ok(()) => (self.function)(args),
            Err(e) => panic!("{}", e),
        }
    }
}

#[derive(Clone)]
pub struct RNWRegisteredNativeMethod {
    name: String,
    method: RNWNativeMethod,
    params: Vec<TypeId>,
}

impl RNWRegisteredNativeMethod {
    pub fn new(name: String, method: RNWNativeMethod, params: Vec<TypeId>) -> Self {
        Self { name, method, params }
    }

    pub fn call(&self, this: RNWObjectRef, args: &[RNWObjectRef]) -> RNWObjectRef {
        let mut combined_args = vec![this.borrow().as_any().type_id()];
        combined_args.extend(args.iter().map(
            |v| v.borrow().as_any().type_id()
        ).collect::<Vec<TypeId>>());

        match check_params(&self.params, combined_args, &self.name) {
            Ok(()) => (self.method)(this, args),
            Err(e) => panic!("{}", e),
        }
    }
}

fn check_params(params: &Vec<TypeId>, args: Vec<TypeId>, function_name: &String) -> Result<(), String> {
    if params.len() != args.len() {
        return Err(format!(
            "{} <{}(...)> expected {} parameters, but {} were given.",
            if function_name.contains(".") {
                "Method"
            } else {
                "Function"
            },
            function_name,
            params.len(),
            args.len()
        ))
    }

    for (param, arg) in params.iter().zip(&args) {
        if param != arg && param != &(TypeId::of::<dyn RNWObject>()) {
            return Err(format!(
                "{} <{}(...)> expected types: ({}), but ({}) were given.",
                if function_name.contains(".") {
                    "Method"
                } else {
                    "Function"
                },
                function_name,
                params.iter().map(
                    |type_id| type_name_from_id(type_id)
                ).collect::<Vec<&'static str>>().join(", "),
                args.iter().map(
                    |type_id| type_name_from_id(type_id)
                ).collect::<Vec<&'static str>>().join(", ")
            ))
        }
    }

    Ok(())
}
