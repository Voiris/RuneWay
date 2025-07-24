use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use once_cell::sync::Lazy;
use crate::runeway::core::errors::{RWResult, RuneWayError};
use super::{RNWBoolean, RNWDict, RNWFloat, RNWInteger, RNWIterator, RNWNullType, RNWString, RNWTuple};
use crate::runeway::runtime::types::{RNWRegisteredNativeMethod, RNWObject, RNWObjectRef, RNWMethod, register_cast, RNWType};

#[derive(Debug, Clone)]
pub struct RNWList {
    value: Vec<RNWObjectRef>,
}

fn native_list_append(this: RNWObjectRef, args: &[RNWObjectRef]) -> RWResult<RNWObjectRef> {
    let mut binding = this.borrow_mut();
    let value = binding.as_any_mut().downcast_mut::<RNWList>().unwrap();
    value.value.push(args[0].clone());
    Ok(RNWNullType::new())
}

fn native_list_reverse(this: RNWObjectRef, _: &[RNWObjectRef]) -> RWResult<RNWObjectRef> {
    let mut binding = this.borrow_mut();
    let value = binding.as_any_mut().downcast_mut::<RNWList>().unwrap();
    value.value.reverse();
    Ok(RNWNullType::new())
}

fn native_list_is_empty(this: RNWObjectRef, _: &[RNWObjectRef]) -> RWResult<RNWObjectRef> {
    let binding = this.borrow();
    let value = binding.as_any().downcast_ref::<RNWList>().unwrap();
    Ok(RNWBoolean::new(value.value.is_empty()))
}

fn native_list_len(this: RNWObjectRef, _: &[RNWObjectRef]) -> RWResult<RNWObjectRef> {
    let binding = this.borrow();
    let value = binding.as_any().downcast_ref::<RNWList>().unwrap();
    Ok(RNWInteger::new(value.value.len() as i64))
}

fn native_list_slice(this: RNWObjectRef, args: &[RNWObjectRef]) -> RWResult<RNWObjectRef> {
    let binding = this.borrow();
    let value = binding.as_any().downcast_ref::<RNWList>().unwrap();
    let borrow = args[0].borrow();
    let &index = borrow.value().downcast_ref::<i64>().unwrap();

    if 0 <= index && index < value.value.len() as i64 {
        Ok(Rc::clone(&value.value[index as usize]))
    } else {
        panic!("List index out of range");
    }
}

fn native_list_iter(this: RNWObjectRef, _: &[RNWObjectRef]) -> RWResult<RNWObjectRef> {
    let binding = this.borrow();
    let value = binding.as_any().downcast_ref::<RNWList>().unwrap();

    Ok(RNWIterator::from_list(value.value.clone()))
}

thread_local! {
    static LIST_NATIVE_FIELDS: Lazy<RefCell<HashMap<&'static str, RNWObjectRef>>> = Lazy::new(|| {
        let mut map = HashMap::new();

        map.insert("append", RNWMethod::new(RNWRegisteredNativeMethod::new(
            "list.append".to_string(),
            Rc::new(native_list_append),
            vec![TypeId::of::<RNWList>(), TypeId::of::<dyn RNWObject>()],
        )));
        map.insert("reverse", RNWMethod::new(RNWRegisteredNativeMethod::new(
            "list.reverse".to_string(),
            Rc::new(native_list_reverse),
            vec![TypeId::of::<RNWList>()]
        )));
        map.insert("is_empty", RNWMethod::new(RNWRegisteredNativeMethod::new(
            "list.is_empty".to_string(),
            Rc::new(native_list_is_empty),
            vec![TypeId::of::<RNWList>()]
        )));
        map.insert("len", RNWMethod::new(RNWRegisteredNativeMethod::new(
            "list.len".to_string(),
            Rc::new(native_list_len),
            vec![TypeId::of::<RNWList>()]
        )));
        map.insert("slice", RNWMethod::new(RNWRegisteredNativeMethod::new(
            "list.slice".to_string(),
            Rc::new(native_list_slice),
            vec![TypeId::of::<RNWList>(), TypeId::of::<RNWInteger>()]
        )));
        map.insert("iter", RNWMethod::new(RNWRegisteredNativeMethod::new(
            "list.iter".to_string(),
            Rc::new(native_list_iter),
            vec![TypeId::of::<RNWList>()]
        )));

        RefCell::new(map)
    });
}

impl RNWList {
    pub fn new(value: &Vec<RNWObjectRef>) -> RNWObjectRef {
        Rc::new(RefCell::new(Self { value: value.clone() }))
    }

    pub fn type_name() -> &'static str { "list" }

    pub fn is_type_equals(other: RNWObjectRef) -> bool {
        TypeId::of::<Self>() == other.borrow().as_any().type_id()
    }
}

impl RNWObject for RNWList {
    fn type_name(&self) -> &'static str { Self::type_name() }
    fn display(&self) -> String {
        format!("[{}]",
                self.value.iter()
                    .map(|x| x.borrow().display())
                    .collect::<Vec<String>>().join(", ")
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
    fn as_object(&self) -> &dyn RNWObject {
        self
    }

    fn field(&self, name: &str) -> Option<RNWObjectRef> {
        LIST_NATIVE_FIELDS.with(
            |methods| methods.borrow().get(name).cloned()
        )
    }
}

pub(super) fn register() -> Rc<RefCell<RNWType>> {
    register_cast::<RNWList, RNWString>(|obj| {
        Ok(RNWString::new(obj.display()))
    });
    register_cast::<RNWList, RNWTuple>(|obj| {
        Ok(RNWTuple::new(&obj.value().downcast_ref::<Vec<RNWObjectRef>>().unwrap().clone()))
    });
    register_cast::<RNWList, RNWBoolean>(|obj| {
        Ok(RNWBoolean::new(!obj.as_any().downcast_ref::<RNWList>().unwrap().value.is_empty()))
    });

    RNWType::new::<RNWList>(RNWList::type_name())
}
