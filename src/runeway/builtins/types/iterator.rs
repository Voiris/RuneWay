use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use once_cell::unsync::Lazy;
use crate::runeway::builtins::types::{RNWBoolean, RNWDict, RNWFloat, RNWInteger, RNWNullType, RNWString};
use crate::runeway::core::errors::RWResult;
use crate::runeway::runtime::types::{register_cast, RNWMethod, RNWObject, RNWObjectRef, RNWRegisteredNativeMethod, RNWType};

#[derive(Debug, Clone)]
enum RNWIteratorKind {
    Range { current: i64, start: i64, end: i64, step: i64 },
    List { items: Vec<RNWObjectRef>, index: usize },
}

impl RNWIteratorKind {
    pub fn reset(&mut self) {
        match self {
            RNWIteratorKind::Range { current, start, .. } => {
                *current = *start;
            }
            RNWIteratorKind::List { items: _, index } => {
                *index = 0;
            }
        }
    }

    pub fn next(&mut self) -> RNWObjectRef {
        match self {
            RNWIteratorKind::Range { current, end, step, .. } => {
                *current += *step;
                if (*step > 0 && *current >= *end) || (*step < 0 && current <= end) {
                    RNWNullType::new()
                } else {
                    RNWInteger::new(*current)
                }
            }
            RNWIteratorKind::List { items, index } => {
                let result = match items.get(*index) {
                    Some(item) => item.clone(),
                    None => RNWNullType::new()
                };
                *index += 1;
                result
            }
        }
    }

    pub fn is_infinite(&self) -> bool {
        match self {
            RNWIteratorKind::Range { step, .. } => {
                *step == 0
            }
            RNWIteratorKind::List { .. } => false
        }
    }

    pub fn display(&self) -> String {
        match self {
            RNWIteratorKind::Range { current, start, end, step } => {
                format!(
                    "<range iterator {}..{}::{} at {}",
                    start,
                    end,
                    step,
                    current
                )
            },
            &RNWIteratorKind::List { index, .. } => format!("<list iterator at {}>", index),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RNWIterator {
    pub kind: RNWIteratorKind,
}

impl RNWIterator {
    pub fn from_range(start: i64, end: i64, step: i64) -> RNWObjectRef {
        Rc::new(RefCell::new(Self {
            kind: RNWIteratorKind::Range {
                current: 0,
                start,
                end,
                step,
            }
        }))
    }

    pub fn from_list(items: Vec<RNWObjectRef>) -> RNWObjectRef {
        Rc::new(RefCell::new(Self {
            kind: RNWIteratorKind::List {
                items,
                index: 0,
            }
        }))
    }

    pub fn type_name() -> &'static str { "iterator" }
    pub fn is_type_equals(other: RNWObjectRef) -> bool {
        other.borrow().type_name() == Self::type_name()
    }

    pub fn next(&mut self) -> RNWObjectRef {
        self.kind.next()
    }

    pub fn reset(&mut self) {
        self.kind.reset();
    }

    pub fn is_infinite(&self) -> bool {
        self.kind.is_infinite()
    }
}

fn native_iterator_next(this: RNWObjectRef, _: &[RNWObjectRef]) -> RWResult<RNWObjectRef> {
    let mut borrow = this.borrow_mut();
    let iter = borrow.as_any_mut().downcast_mut::<RNWIterator>().unwrap();

    Ok(iter.next())
}

fn native_iterator_reset(this: RNWObjectRef, _: &[RNWObjectRef]) -> RWResult<RNWObjectRef> {
    let mut borrow = this.borrow_mut();
    let iter = borrow.as_any_mut().downcast_mut::<RNWIterator>().unwrap();

    iter.reset();

    drop(borrow);

    Ok(RNWNullType::new())
}

fn native_iterator_is_infinite(this: RNWObjectRef, _: &[RNWObjectRef]) -> RWResult<RNWObjectRef> {
    let mut borrow = this.borrow_mut();
    let iter = borrow.as_any_mut().downcast_mut::<RNWIterator>().unwrap();

    Ok(RNWBoolean::new(iter.is_infinite()))
}

thread_local! {
    static ITERATOR_NATIVE_FIELDS: Lazy<RefCell<HashMap<&'static str, RNWObjectRef>>> = Lazy::new(|| {
        let mut map = HashMap::new();

        map.insert("next", RNWMethod::new(RNWRegisteredNativeMethod::new(
            "iterator.next".to_string(),
            Rc::new(native_iterator_next),
            vec![TypeId::of::<RNWIterator>()]
        )));
        map.insert("reset", RNWMethod::new(RNWRegisteredNativeMethod::new(
            "iterator.reset".to_string(),
            Rc::new(native_iterator_reset),
            vec![TypeId::of::<RNWIterator>()]
        )));
        map.insert("is_infinite", RNWMethod::new(RNWRegisteredNativeMethod::new(
            "iterator.is_infinite".to_string(),
            Rc::new(native_iterator_is_infinite),
            vec![TypeId::of::<RNWIterator>()]
        )));

        RefCell::new(map)
    })
}

impl RNWObject for RNWIterator {
    fn type_name(&self) -> &'static str { Self::type_name() }
    fn display(&self) -> String {
        self.kind.display()
    }
    fn value(&self) -> &dyn Any { self }
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn as_object(&self) -> &dyn RNWObject { self }

    //noinspection DuplicatedCode
    fn field(&self, name: &str) -> Option<RNWObjectRef> {
        ITERATOR_NATIVE_FIELDS.with(|iter| {
            iter.borrow().get(name).cloned()
        })
    }
}

pub(super) fn register() -> Rc<RefCell<RNWType>> {
    RNWType::new::<RNWIterator>(RNWIterator::type_name())
}
