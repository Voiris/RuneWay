use crate::assign_rnw_type_id;
use crate::runeway::builtins::types::{RNWBoolean, RNWInteger, RNWList, RNWNullType, RNWString};
use crate::runeway::core::errors::{RWResult, RuneWayError, RuneWayErrorKind};
use crate::runeway::runtime::types::{
    register_cast, RNWMethod, RNWObject, RNWObjectRef, RNWRegisteredNativeMethod, RNWType,
    RNWTypeId,
};
use once_cell::unsync::Lazy;
use std::any::Any;
use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

fn check_string_key(borrow: Ref<dyn RNWObject>) -> RWResult<String> {
    match borrow.value().downcast_ref::<String>() {
        Some(k) => Ok(k.clone()),
        None => Err(
            RuneWayError::new(RuneWayErrorKind::error_with_code("KeyError"))
                .with_message("Key must be a string"),
        ),
    }
}

#[derive(Debug, Clone)]
pub struct RNWDict {
    pub entries: HashMap<String, RNWObjectRef>,
}

// Native methods
fn native_dict_slice(this: RNWObjectRef, args: &[RNWObjectRef]) -> RWResult<RNWObjectRef> {
    let borrow = args.get(0).unwrap().borrow();
    let key = check_string_key(borrow)?;

    let borrow = this.borrow();
    let dict = borrow.as_any().downcast_ref::<RNWDict>().unwrap();
    dict.entries.get(&key).cloned().ok_or(
        RuneWayError::new(RuneWayErrorKind::error_with_code("KeyError"))
            .with_message("Key not found in dictionary"),
    )
}

fn native_dict_get(this: RNWObjectRef, args: &[RNWObjectRef]) -> RWResult<RNWObjectRef> {
    let borrow = args.get(0).unwrap().borrow();
    let key = check_string_key(borrow)?;

    let borrow = this.borrow();
    let dict = borrow.as_any().downcast_ref::<RNWDict>().unwrap();
    Ok(dict
        .entries
        .get(&key)
        .cloned()
        .unwrap_or_else(|| RNWNullType::new()))
}

fn native_dict_keys(this: RNWObjectRef, _: &[RNWObjectRef]) -> RWResult<RNWObjectRef> {
    let binding = this.borrow();
    let dict = binding.as_any().downcast_ref::<RNWDict>().unwrap();
    let keys = dict
        .entries
        .keys()
        .cloned()
        .map(|k| RNWString::new(k))
        .collect::<Vec<_>>();
    Ok(RNWList::new(&keys))
}

fn native_dict_values(this: RNWObjectRef, _: &[RNWObjectRef]) -> RWResult<RNWObjectRef> {
    let binding = this.borrow();
    let dict = binding.as_any().downcast_ref::<RNWDict>().unwrap();
    let values = dict.entries.values().cloned().collect::<Vec<_>>();
    Ok(RNWList::new(&values))
}

// TODO: dict_iter -> tuple((key, value))

fn native_dict_len(this: RNWObjectRef, _: &[RNWObjectRef]) -> RWResult<RNWObjectRef> {
    let binding = this.borrow();
    let dict = binding.as_any().downcast_ref::<RNWDict>().unwrap();
    Ok(RNWInteger::new(dict.entries.len() as i64))
}

fn native_dict_insert(this: RNWObjectRef, args: &[RNWObjectRef]) -> RWResult<RNWObjectRef> {
    let key_obj = args.get(0).unwrap();
    let val_obj = args.get(1).unwrap();

    let key = {
        let key_borrow = key_obj.borrow();
        match key_borrow.value().downcast_ref::<String>() {
            Some(k) => k.clone(),
            None => {
                return Err(
                    RuneWayError::new(RuneWayErrorKind::type_error()).with_message(format!(
                        "Cannot cast type <{}> to string",
                        key_borrow.type_name()
                    )),
                );
            }
        }
    };
    let mut borrow = this.borrow_mut();
    let dict = borrow.as_any_mut().downcast_mut::<RNWDict>().unwrap();
    dict.entries.insert(key, val_obj.clone());

    Ok(RNWNullType::new())
}

thread_local! {
    static DICT_NATIVE_FIELDS: Lazy<RefCell<HashMap<&'static str, RNWObjectRef>>> = Lazy::new(|| {
        let mut map = HashMap::new();

        map.insert("get_", RNWMethod::new(RNWRegisteredNativeMethod::new(
            "dict.get_".to_string(),
            Rc::new(native_dict_get),
            vec![RNWDict::rnw_type_id(), 0]
        )));
        map.insert("keys", RNWMethod::new(RNWRegisteredNativeMethod::new(
            "dict.keys".to_string(),
            Rc::new(native_dict_keys),
            vec![RNWDict::rnw_type_id()]
        )));
        map.insert("values", RNWMethod::new(RNWRegisteredNativeMethod::new(
            "dict.values".to_string(),
            Rc::new(native_dict_values),
            vec![RNWDict::rnw_type_id()]
        )));
        map.insert("len", RNWMethod::new(RNWRegisteredNativeMethod::new(
            "dict.len".to_string(),
            Rc::new(native_dict_len),
            vec![RNWDict::rnw_type_id()]
        )));
        map.insert("insert", RNWMethod::new(RNWRegisteredNativeMethod::new(
            "dict.insert".to_string(),
            Rc::new(native_dict_insert),
            vec![RNWDict::rnw_type_id(), 0, 0]
        )));
        map.insert("slice", RNWMethod::new(RNWRegisteredNativeMethod::new(
            "dict.slice".to_string(),
            Rc::new(native_dict_slice),
            vec![RNWDict::rnw_type_id(), RNWString::rnw_type_id()]
        )));

        RefCell::new(map)
    });
}

impl RNWDict {
    pub fn new(entries: HashMap<String, RNWObjectRef>) -> RNWObjectRef {
        Rc::new(RefCell::new(Self { entries }))
    }

    pub fn type_name() -> &'static str {
        "dict"
    }

    pub fn is_type_equals(other: &RNWObjectRef) -> bool {
        Self::rnw_type_id() == other.borrow().rnw_type_id()
    }

    assign_rnw_type_id!();
}

impl RNWObject for RNWDict {
    fn rnw_type_id(&self) -> RNWTypeId {
        Self::rnw_type_id()
    }
    fn type_name(&self) -> &'static str {
        Self::type_name()
    }
    fn display(&self) -> String {
        format!(
            "{{{}}}",
            self.entries
                .iter()
                .map(|(k, v)| format!("\"{}\": {}", k, v.borrow().display()))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
    fn value(&self) -> &dyn Any {
        &self.entries
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_attr(&self, name: &str) -> Option<RNWObjectRef> {
        DICT_NATIVE_FIELDS.with(|methods| methods.borrow().get(name).cloned())
    }
}

pub(super) fn register() -> Rc<RefCell<RNWType>> {
    register_cast(RNWDict::rnw_type_id(), RNWString::rnw_type_id(), |obj| {
        Ok(RNWString::new(obj.display()))
    });
    register_cast(RNWDict::rnw_type_id(), RNWBoolean::rnw_type_id(), |obj| {
        Ok(RNWBoolean::new(
            obj.as_any()
                .downcast_ref::<RNWDict>()
                .unwrap()
                .entries
                .len()
                > 0,
        ))
    });

    RNWType::new(RNWDict::rnw_type_id(), RNWDict::type_name())
}
