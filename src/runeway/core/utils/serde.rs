use crate::runeway::builtins::types::{
    RNWBoolean, RNWDict, RNWFloat, RNWInteger, RNWList, RNWNullType, RNWString,
};
use crate::runeway::core::errors::{RWResult, RuneWayError, RuneWayErrorKind};
use crate::runeway::runtime::types::RNWObjectRef;
use colored::*;
use serde_json::{json, Value};
use std::collections::HashMap;

pub fn serde_evaluate(value: Value) -> RNWObjectRef {
    match value {
        Value::Null => RNWNullType::new(),
        Value::Bool(b) => RNWBoolean::new(b),
        Value::Number(n) => {
            if let Some(u) = n.as_u64() {
                RNWInteger::new(u as i64)
            } else if let Some(i) = n.as_i64() {
                RNWInteger::new(i)
            } else if let Some(f) = n.as_f64() {
                RNWFloat::new(f)
            } else {
                panic!("Internal: Number {:?} is not a number", n);
            }
        }
        Value::String(s) => RNWString::new(&s),
        Value::Array(value) => {
            let mut list = Vec::new();

            for v in value.iter() {
                list.push(serde_evaluate(v.clone()));
            }

            RNWList::new(&list)
        }
        Value::Object(obj) => {
            let mut map = HashMap::new();

            for (k, v) in obj.iter() {
                map.insert(k.clone(), serde_evaluate(v.clone()));
            }

            RNWDict::new(map)
        }
    }
}

pub fn serde_serialize(object: RNWObjectRef) -> RWResult<Value> {
    let borrow = object.borrow();
    if RNWNullType::is_type_equals(&object) {
        Ok(Value::Null)
    } else if let Some(i) = borrow.value().downcast_ref::<i64>() {
        Ok(json!(i))
    } else if let Some(f) = borrow.value().downcast_ref::<f64>() {
        Ok(json!(f))
    } else if let Some(s) = borrow.value().downcast_ref::<String>() {
        Ok(json!(s))
    } else if let Some(arr) = borrow.value().downcast_ref::<Vec<RNWObjectRef>>() {
        let mut vec = Vec::new();

        for v in arr.iter() {
            vec.push(serde_serialize(v.clone())?);
        }

        Ok(json!(vec))
    } else if let Some(obj) = borrow
        .value()
        .downcast_ref::<HashMap<String, RNWObjectRef>>()
    {
        let mut map = HashMap::new();

        for (k, v) in obj.iter() {
            map.insert(k.clone(), serde_serialize(v.clone())?);
        }

        Ok(json!(map))
    } else {
        Err(
            RuneWayError::new(RuneWayErrorKind::type_error()).with_message(format!(
                "Cannot serialize type: {}",
                borrow.type_name().bright_yellow()
            )),
        )
    }
}
