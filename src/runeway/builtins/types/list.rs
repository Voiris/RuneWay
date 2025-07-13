use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use once_cell::sync::Lazy;
use super::{RNWBoolean, RNWInteger, RNWIterator, RNWNullType, RNWString};
use crate::runeway::executor::runtime::types::{RNWRegisteredNativeMethod, RNWObject, RNWObjectRef};

#[derive(Debug, Clone)]
pub struct RNWList {
    value: Vec<RNWObjectRef>,
}

fn native_list_append(this: RNWObjectRef, args: &[RNWObjectRef]) -> RNWObjectRef {
    let mut binding = this.borrow_mut();
    let value = binding.as_any_mut().downcast_mut::<RNWList>().unwrap();
    value.value.push(args[0].clone());
    RNWNullType::new()
}

fn native_list_reverse(this: RNWObjectRef, _: &[RNWObjectRef]) -> RNWObjectRef {
    let mut binding = this.borrow_mut();
    let value = binding.as_any_mut().downcast_mut::<RNWList>().unwrap();
    value.value.reverse();
    RNWNullType::new()
}

fn native_list_is_empty(this: RNWObjectRef, _: &[RNWObjectRef]) -> RNWObjectRef {
    let binding = this.borrow();
    let value = binding.as_any().downcast_ref::<RNWList>().unwrap();
    RNWBoolean::new(value.value.is_empty())
}

fn native_list_len(this: RNWObjectRef, _: &[RNWObjectRef]) -> RNWObjectRef {
    let binding = this.borrow();
    let value = binding.as_any().downcast_ref::<RNWList>().unwrap();
    RNWInteger::new(value.value.len() as i64)
}

fn native_list_slice(this: RNWObjectRef, args: &[RNWObjectRef]) -> RNWObjectRef {
    let binding = this.borrow();
    let value = binding.as_any().downcast_ref::<RNWList>().unwrap();
    let borrow = args[0].borrow();
    let &index = borrow.value().downcast_ref::<i64>().unwrap();

    if 0 <= index && index < value.value.len() as i64 {
        Rc::clone(&value.value[index as usize])
    } else {
        panic!("List index out of range");
    }
}

fn native_list_to_string(this: RNWObjectRef, _: &[RNWObjectRef]) -> RNWObjectRef {
    let binding = this.borrow();
    let value = binding.as_any().downcast_ref::<RNWList>().unwrap();
    RNWString::new(value.display())
}

fn native_list_iter(this: RNWObjectRef, _: &[RNWObjectRef]) -> RNWObjectRef {
    let binding = this.borrow();
    let value = binding.as_any().downcast_ref::<RNWList>().unwrap();

    RNWIterator::from_list(value.value.clone())
}

thread_local! {
    static LIST_NATIVE_METHODS: Lazy<RefCell<HashMap<&'static str, RNWRegisteredNativeMethod>>> = Lazy::new(|| {
        let mut map = HashMap::new();

        map.insert("append", RNWRegisteredNativeMethod::new(
            "list.append".to_string(),
            Rc::new(native_list_append),
            vec![TypeId::of::<RNWList>(), TypeId::of::<dyn RNWObject>()],
        ));
        map.insert("reverse", RNWRegisteredNativeMethod::new(
            "list.reverse".to_string(),
            Rc::new(native_list_reverse),
            vec![TypeId::of::<RNWList>()]
        ));
        map.insert("is_empty", RNWRegisteredNativeMethod::new(
            "list.is_empty".to_string(),
            Rc::new(native_list_is_empty),
            vec![TypeId::of::<RNWList>()]
        ));
        map.insert("to_string", RNWRegisteredNativeMethod::new(
            "list.to_string".to_string(),
            Rc::new(native_list_to_string),
            vec![TypeId::of::<RNWList>()]
        ));
        map.insert("len", RNWRegisteredNativeMethod::new(
            "list.len".to_string(),
            Rc::new(native_list_len),
            vec![TypeId::of::<RNWList>()]
        ));
        map.insert("slice", RNWRegisteredNativeMethod::new(
            "list.slice".to_string(),
            Rc::new(native_list_slice),
            vec![TypeId::of::<RNWList>(), TypeId::of::<RNWInteger>()]
        ));
        map.insert("iter", RNWRegisteredNativeMethod::new(
            "list.iter".to_string(),
            Rc::new(native_list_iter),
            vec![TypeId::of::<RNWList>()]
        ));

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

    fn method(&self, name: &str) -> Option<RNWRegisteredNativeMethod> {
        LIST_NATIVE_METHODS.with(
            |methods| methods.borrow().get(name).cloned()
        )
    }
}