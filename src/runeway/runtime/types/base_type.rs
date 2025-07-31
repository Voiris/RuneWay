use crate::runeway::core::ast::operators::{BinaryOperator, UnaryOperator};
use crate::runeway::core::errors::{RWResult, RuneWayError, RuneWayErrorKind};
use std::any::Any;
use std::cell::{Ref, RefCell};
use std::cmp::Ordering;
use std::fmt::Debug;
use std::rc::Rc;

pub trait RNWObject: Debug {
    fn rnw_type_id(&self) -> RNWTypeId;
    fn type_name(&self) -> &'static str;
    fn display(&self) -> String;
    fn value(&self) -> &dyn Any;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// param: name of field
    /// return: value of field if it exists
    fn get_attr(&self, _: &str) -> Option<RNWObjectRef> {
        None
    }
    fn set_attr(&mut self, _: &str, _: RNWObjectRef) -> RWResult<()> {
        Err(
            RuneWayError::new(RuneWayErrorKind::error_with_code("AttributeError")).with_message(
                format!(
                    "Object with type <{}> not supported attribute setting",
                    self.type_name()
                ),
            ),
        )
    }

    fn binary_operation(&self, _: RNWObjectRef, _: BinaryOperator) -> Option<RNWObjectRef> {
        None
    }

    fn unary_operation(&self, _: UnaryOperator) -> Option<RNWObjectRef> {
        None
    }

    fn call(&self, _: &[RNWObjectRef]) -> Option<RWResult<RNWObjectRef>> {
        None
    }
}

pub type RNWObjectRef = Rc<RefCell<dyn RNWObject>>;

fn binary_cmp(
    left_borrow: &Ref<dyn RNWObject>,
    right: &RNWObjectRef,
    binary_operator: BinaryOperator,
    ordering: Ordering,
) -> Option<Ordering> {
    match left_borrow.binary_operation(right.clone(), binary_operator) {
        Some(value) => {
            if let Some(b) = value.borrow().value().downcast_ref::<bool>() {
                if *b { Some(ordering) } else { None }
            } else {
                None
            }
        }
        None => None,
    }
}

pub fn partial_cmp(left: &RNWObjectRef, right: &RNWObjectRef) -> Option<Ordering> {
    let left_borrow = left.borrow();
    match binary_cmp(&left_borrow, right, BinaryOperator::Eq, Ordering::Equal) {
        Some(ord) => return Some(ord),
        None => (),
    }
    match binary_cmp(&left_borrow, right, BinaryOperator::LtEq, Ordering::Less) {
        Some(ord) => return Some(ord),
        None => (),
    }
    match binary_cmp(&left_borrow, right, BinaryOperator::GtEq, Ordering::Greater) {
        Some(ord) => return Some(ord),
        None => (),
    }
    None
}

pub type RNWTypeId = usize;

pub fn gen_rnw_type_id() -> RNWTypeId {
    static RNW_TYPE_ID_COUNTER: std::sync::atomic::AtomicUsize =
        std::sync::atomic::AtomicUsize::new(1usize);
    RNW_TYPE_ID_COUNTER.fetch_add(1usize, std::sync::atomic::Ordering::Relaxed)
}

#[macro_export]
macro_rules! assign_rnw_type_id {
    () => {
        pub fn rnw_type_id() -> crate::runeway::runtime::types::RNWTypeId {
            static RNW_TYPE_ID: once_cell::sync::OnceCell<
                crate::runeway::runtime::types::RNWTypeId,
            > = once_cell::sync::OnceCell::new();
            RNW_TYPE_ID
                .get_or_init(|| crate::runeway::runtime::types::gen_rnw_type_id())
                .clone()
        }
    };
}
