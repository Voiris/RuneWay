use std::cell::RefCell;
use super::RNWObject;
use super::{type_name_from_id, RNWTypeId};
use crate::runeway::core::errors::{RWResult, RuneWayError, RuneWayErrorKind};
use crate::runeway::core::utils::assert::assert_incorrect_type;
use crate::runeway::runtime::types::base_type::RNWObjectRef;
use std::rc::Rc;

pub type RNWNativeFunction = Rc<dyn Fn(&[RNWObjectRef]) -> RWResult<RNWObjectRef>>;
pub type RNWNativeMethod = Rc<dyn Fn(RNWObjectRef, &[RNWObjectRef]) -> RWResult<RNWObjectRef>>;

#[derive(Clone)]
pub struct RNWRegisteredNativeFunction {
    pub name: String,
    function: RNWNativeFunction,
    params: Vec<RNWTypeId>,
    return_type: Option<RNWTypeId>,
    unlimited: bool,
}

impl RNWRegisteredNativeFunction {
    pub fn new(name: String, function: RNWNativeFunction, params: Vec<RNWTypeId>) -> Rc<Self> {
        Rc::new(Self {
            name,
            function,
            params,
            return_type: None,
            unlimited: false,
        })
    }

    pub fn new_unlimited(
        name: String,
        function: RNWNativeFunction,
        params: Vec<RNWTypeId>,
    ) -> Rc<Self> {
        Rc::new(Self {
            name,
            function,
            params,
            return_type: None,
            unlimited: true,
        })
    }

    pub fn new_with_return_type(
        name: String,
        function: RNWNativeFunction,
        params: Vec<RNWTypeId>,
        return_type: Option<RNWTypeId>,
    ) -> Rc<Self> {
        Rc::new(Self {
            name,
            function,
            params,
            return_type,
            unlimited: false,
        })
    }

    pub fn new_unlimited_with_return_type(
        name: String,
        function: RNWNativeFunction,
        params: Vec<RNWTypeId>,
        return_type: Option<RNWTypeId>,
    ) -> Rc<Self> {
        Rc::new(Self {
            name,
            function,
            params,
            return_type,
            unlimited: true,
        })
    }

    pub fn call(&self, args: &[RNWObjectRef]) -> RWResult<RNWObjectRef> {
        match check_params(
            &self.params,
            args.iter().map(|v| v.borrow().rnw_type_id()).collect(),
            &self.name,
            self.unlimited,
        ) {
            Ok(()) => match (self.function)(args) {
                Ok(r) => {
                    if let Some(return_type) = self.return_type {
                        let borrow = r.borrow();
                        assert_incorrect_type(return_type, borrow.rnw_type_id())?;
                    }

                    Ok(r)
                }
                Err(e) => Err(e),
            },
            Err(e) => Err(e),
        }
    }
}

#[derive(Clone)]
pub struct RNWRegisteredNativeMethod {
    pub name: String,
    method: RNWNativeMethod,
    params: Vec<RNWTypeId>,
    return_type: Option<RNWTypeId>,
    unlimited: bool,
}

impl RNWRegisteredNativeMethod {
    pub fn new(name: String, method: RNWNativeMethod, params: Vec<RNWTypeId>) -> Rc<Self> {
        Rc::new(Self {
            name,
            method,
            params,
            return_type: None,
            unlimited: false,
        })
    }

    pub fn new_unlimited(
        name: String,
        method: RNWNativeMethod,
        params: Vec<RNWTypeId>,
    ) -> Rc<Self> {
        Rc::new(Self {
            name,
            method,
            params,
            return_type: None,
            unlimited: true,
        })
    }

    pub fn new_with_return_type(
        name: String,
        method: RNWNativeMethod,
        params: Vec<RNWTypeId>,
        return_type: Option<RNWTypeId>,
    ) -> Rc<Self> {
        Rc::new(Self {
            name,
            method,
            params,
            return_type,
            unlimited: false,
        })
    }

    pub fn call(&self, this: RNWObjectRef, args: &[RNWObjectRef]) -> RWResult<RNWObjectRef> {
        let mut combined_args = vec![this.borrow().rnw_type_id()];
        combined_args.extend(
            args.iter()
                .map(|v| v.borrow().rnw_type_id())
                .collect::<Vec<_>>(),
        );

        match check_params(&self.params, combined_args, &self.name, self.unlimited) {
            Ok(()) => (self.method)(this, args),
            Err(e) => Err(e),
        }
    }
}

fn check_params(
    params: &Vec<RNWTypeId>,
    args: Vec<RNWTypeId>,
    function_name: &String,
    unlimited: bool,
) -> RWResult<()> {
    if unlimited && (args.len() < params.len()) || (!unlimited && params.len() != args.len()) {
        return Err(
            RuneWayError::new(RuneWayErrorKind::error_with_code("ArgumentsError")).with_message(
                if unlimited {
                    format!(
                        "{} <{}(...)> expects minimum {} argument(s), but {} were provided.",
                        if function_name.contains(".") {
                            "Method"
                        } else {
                            "Function"
                        },
                        function_name,
                        params.len(),
                        args.len()
                    )
                } else {
                    format!(
                        "{} <{}(...)> expects {} argument(s), but {} were provided.",
                        if function_name.contains(".") {
                            "Method"
                        } else {
                            "Function"
                        },
                        function_name,
                        params.len(),
                        args.len()
                    )
                },
            ),
        );
    }

    for (param, arg) in params.iter().zip(&args) {
        if param != arg && param != &0 {
            return Err(
                RuneWayError::new(RuneWayErrorKind::type_error()).with_message(format!(
                    "{} <{}(...)> expects types: ({}), but ({}) were provided.",
                    if function_name.contains(".") {
                        "Method"
                    } else {
                        "Function"
                    },
                    function_name,
                    params
                        .iter()
                        .map(|type_id| type_name_from_id(*type_id))
                        .collect::<Vec<&'static str>>()
                        .join(", "),
                    args.iter()
                        .map(|type_id| type_name_from_id(*type_id))
                        .collect::<Vec<&'static str>>()
                        .join(", ")
                )),
            );
        }
    }

    Ok(())
}

pub enum RNWRegisteredCallable {
    Function(RNWRegisteredNativeFunction),
    Method(RNWRegisteredNativeMethod),
}
