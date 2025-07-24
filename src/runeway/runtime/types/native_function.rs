use std::any::{Any, TypeId};
use std::rc::Rc;
use crate::runeway::core::errors::{RWResult, RuneWayError, RuneWayErrorKind};
use crate::runeway::runtime::types::base_type::RNWObjectRef;
use super::type_name_from_id;
use super::RNWObject;

pub type RNWNativeFunction = Rc<dyn Fn(&[RNWObjectRef]) -> RWResult<RNWObjectRef>>;
pub type RNWNativeMethod = Rc<dyn Fn(RNWObjectRef, &[RNWObjectRef]) -> RWResult<RNWObjectRef>>;

#[derive(Clone)]
pub struct RNWRegisteredNativeFunction {
    pub name: String,
    function: RNWNativeFunction,
    params: Vec<TypeId>,
    unlimited: bool,
}

impl RNWRegisteredNativeFunction {
    pub fn new(name: String, function: RNWNativeFunction, params: Vec<TypeId>) -> Rc<Self> {
        Rc::new(Self { name, function, params, unlimited: false })
    }

    pub fn new_unlimited(name: String, function: RNWNativeFunction, params: Vec<TypeId>) -> Rc<Self> {
        Rc::new(Self { name, function, params, unlimited: true })
    }

    pub fn call(&self, args: &[RNWObjectRef]) -> RWResult<RNWObjectRef> {
        match check_params(&self.params, args.iter().map(
            |v| v.borrow().as_any().type_id()
        ).collect(), &self.name, self.unlimited) {
            Ok(()) => (self.function)(args),
            Err(e) => Err(e),
        }
    }
}

#[derive(Clone)]
pub struct RNWRegisteredNativeMethod {
    pub name: String,
    method: RNWNativeMethod,
    params: Vec<TypeId>,
    unlimited: bool,
}

impl RNWRegisteredNativeMethod {
    pub fn new(name: String, method: RNWNativeMethod, params: Vec<TypeId>) -> Rc<Self> {
        Rc::new(Self { name, method, params, unlimited: false })
    }

    pub fn new_unlimited(name: String, method: RNWNativeMethod, params: Vec<TypeId>) -> Rc<Self> {
        Rc::new(Self { name, method, params, unlimited: true })
    }

    pub fn call(&self, this: RNWObjectRef, args: &[RNWObjectRef]) -> RWResult<RNWObjectRef> {
        let mut combined_args = vec![this.borrow().as_any().type_id()];
        combined_args.extend(args.iter().map(
            |v| v.borrow().as_any().type_id()
        ).collect::<Vec<TypeId>>());

        match check_params(&self.params, combined_args, &self.name, self.unlimited) {
            Ok(()) => (self.method)(this, args),
            Err(e) => Err(e),
        }
    }
}

fn check_params(params: &Vec<TypeId>, args: Vec<TypeId>, function_name: &String,
                unlimited: bool) -> RWResult<()> {
    if (unlimited && (args.len() < params.len()) || (!unlimited && params.len() != args.len())) {
        return Err(
            RuneWayError::new(RuneWayErrorKind::Runtime(Some("ArgumentsError".to_string())))
                .with_message(
                    if unlimited {
                        format!(
                            "{} <{}(...)> expects minimum {} argument(s), but {} were provided.",
                            if function_name.contains(".") { "Method" } else { "Function" },
                            function_name,
                            params.len(),
                            args.len()
                        )
                    } else {
                        format!(
                            "{} <{}(...)> expects {} argument(s), but {} were provided.",
                            if function_name.contains(".") { "Method" } else { "Function" },
                            function_name,
                            params.len(),
                            args.len()
                        )
                    }
                )
        )
    }

    for (param, arg) in params.iter().zip(&args) {
        if param != arg && param != &(TypeId::of::<dyn RNWObject>()) {
            return Err(
                RuneWayError::new(RuneWayErrorKind::Runtime(Some("TypeError".to_string())))
                    .with_message(
                        format!(
                            "{} <{}(...)> expects types: ({}), but ({}) were provided.",
                            if function_name.contains(".") { "Method" } else { "Function" },
                            function_name,
                            params.iter().map(
                                |type_id| type_name_from_id(type_id)
                            ).collect::<Vec<&'static str>>().join(", "),
                            args.iter().map(
                                |type_id| type_name_from_id(type_id)
                            ).collect::<Vec<&'static str>>().join(", ")
                        )
                    )
            )
        }
    }

    Ok(())
}
