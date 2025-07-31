use crate::assign_rnw_type_id;
use crate::runeway::builtins::types::{RNWBoolean, RNWFloat, RNWInteger, RNWNullType};
use crate::runeway::core::errors::RWResult;
use crate::runeway::runtime::types::{
    RNWMethod, RNWObject, RNWObjectRef, RNWRegisteredNativeMethod, RNWType, RNWTypeId,
};
use once_cell::unsync::Lazy;
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone)]
enum RNWIteratorKind {
    Range {
        current: i64,
        start: i64,
        end: i64,
        step: i64,
    },
    FloatRange {
        current: f64,
        start: f64,
        end: f64,
        step: f64,
    },
    List {
        items: Vec<RNWObjectRef>,
        index: usize,
    },
}

impl RNWIteratorKind {
    pub fn reset(&mut self) {
        match self {
            RNWIteratorKind::Range { current, start, .. } => {
                *current = *start;
            }
            RNWIteratorKind::FloatRange { current, start, .. } => {
                *current = *start;
            }
            RNWIteratorKind::List { items: _, index } => {
                *index = 0;
            }
        }
    }

    pub fn inner_next(&mut self) -> Option<RNWObjectRef> {
        match self {
            RNWIteratorKind::Range {
                current, end, step, ..
            } => {
                let result = if (*step > 0 && *current >= *end) || (*step < 0 && current <= end) {
                    None
                } else {
                    Some(RNWInteger::new(*current))
                };
                *current += *step;
                result
            }
            RNWIteratorKind::FloatRange {
                current, end, step, ..
            } => {
                let result = if (*step > 0.0 && *current >= *end) || (*step < 0.0 && current <= end)
                {
                    None
                } else {
                    Some(RNWFloat::new(*current))
                };
                *current += *step;
                result
            }
            RNWIteratorKind::List { items, index } => {
                let result = items.get(*index).cloned();
                *index += 1;
                result
            }
        }
    }

    pub fn is_infinite(&self) -> bool {
        match self {
            RNWIteratorKind::Range { step, .. } => *step == 0,
            RNWIteratorKind::FloatRange { step, .. } => *step == 0.0,
            RNWIteratorKind::List { .. } => false,
        }
    }

    pub fn display(&self) -> String {
        match self {
            RNWIteratorKind::Range {
                current,
                start,
                end,
                step,
            } => {
                format!(
                    "<range iterator {}..{}::{} at {}",
                    start, end, step, current
                )
            }
            RNWIteratorKind::FloatRange {
                current,
                start,
                end,
                step,
            } => {
                format!(
                    "<range iterator {}..{}::{} at {}",
                    start, end, step, current
                )
            }
            &RNWIteratorKind::List { index, .. } => format!("<list iterator at {}>", index),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RNWIterator {
    pub kind: RNWIteratorKind,
}

impl RNWIterator {
    pub fn from_i64_range(start: i64, end: i64, step: i64) -> RNWObjectRef {
        Rc::new(RefCell::new(Self {
            kind: RNWIteratorKind::Range {
                current: start,
                start,
                end,
                step,
            },
        }))
    }

    pub fn from_f64_range(start: f64, end: f64, step: f64) -> RNWObjectRef {
        Rc::new(RefCell::new(Self {
            kind: RNWIteratorKind::FloatRange {
                current: start,
                start,
                end,
                step,
            },
        }))
    }

    pub fn from_list(items: Vec<RNWObjectRef>) -> RNWObjectRef {
        Rc::new(RefCell::new(Self {
            kind: RNWIteratorKind::List { items, index: 0 },
        }))
    }

    pub fn type_name() -> &'static str {
        "iterator"
    }

    pub fn is_type_equals(other: &RNWObjectRef) -> bool {
        Self::rnw_type_id() == other.borrow().rnw_type_id()
    }

    pub fn next(&mut self) -> RNWObjectRef {
        self.kind.inner_next().unwrap_or_else(|| RNWNullType::new())
    }

    pub fn reset(&mut self) {
        self.kind.reset();
    }

    pub fn is_infinite(&self) -> bool {
        self.kind.is_infinite()
    }

    assign_rnw_type_id!();
}

impl Iterator for RNWIterator {
    type Item = RNWObjectRef;

    fn next(&mut self) -> Option<Self::Item> {
        self.kind.inner_next()
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
            vec![RNWIterator::rnw_type_id()]
        )));
        map.insert("reset", RNWMethod::new(RNWRegisteredNativeMethod::new(
            "iterator.reset".to_string(),
            Rc::new(native_iterator_reset),
            vec![RNWIterator::rnw_type_id()]
        )));
        map.insert("is_infinite", RNWMethod::new(RNWRegisteredNativeMethod::new(
            "iterator.is_infinite".to_string(),
            Rc::new(native_iterator_is_infinite),
            vec![RNWIterator::rnw_type_id()]
        )));

        RefCell::new(map)
    })
}

impl RNWObject for RNWIterator {
    fn rnw_type_id(&self) -> RNWTypeId {
        Self::rnw_type_id()
    }
    fn type_name(&self) -> &'static str {
        Self::type_name()
    }
    fn display(&self) -> String {
        self.kind.display()
    }
    fn value(&self) -> &dyn Any {
        self
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    //noinspection DuplicatedCode
    fn get_attr(&self, name: &str) -> Option<RNWObjectRef> {
        ITERATOR_NATIVE_FIELDS.with(|iter| iter.borrow().get(name).cloned())
    }
}

pub(super) fn register() -> Rc<RefCell<RNWType>> {
    RNWType::new(RNWIterator::rnw_type_id(), RNWIterator::type_name())
}
