use crate::assign_rnw_type_id;
use crate::runeway::builtins::types::{RNWBoolean, RNWFloat, RNWInteger, RNWString};
use crate::runeway::core::errors::RWResult;
use crate::runeway::core::utils::{i64_to_u64_shifted, u64_to_i64_shifted};
use crate::runeway::runtime::types::{
    register_cast, RNWMethod, RNWObject, RNWObjectRef, RNWRegisteredNativeMethod, RNWType,
    RNWTypeId,
};
use once_cell::unsync::Lazy;
use rand::{Rng, RngCore, SeedableRng};
use rand_pcg::Pcg32;
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::thread_local;

#[derive(Debug, Clone)]
pub struct RNWRandomNumberGenerator {
    seed: u64,
    rng: Pcg32,
}

fn get_next_u64(this: RNWObjectRef) -> u64 {
    let mut borrow = this.borrow_mut();
    let this = borrow
        .as_any_mut()
        .downcast_mut::<RNWRandomNumberGenerator>()
        .unwrap();
    this.rng.next_u64()
}

pub fn native_rng_positive(this: RNWObjectRef, _: &[RNWObjectRef]) -> RWResult<RNWObjectRef> {
    let next_u64 = get_next_u64(this);

    Ok(RNWInteger::new(u64_to_i64_shifted(next_u64).abs()))
}

pub fn native_rng_negative(this: RNWObjectRef, _: &[RNWObjectRef]) -> RWResult<RNWObjectRef> {
    let next_u64 = get_next_u64(this);

    let i = u64_to_i64_shifted(next_u64);

    Ok(RNWInteger::new(if i < 0 { i } else { -i }))
}

pub fn native_rng_random_int(this: RNWObjectRef, _: &[RNWObjectRef]) -> RWResult<RNWObjectRef> {
    let next_u64 = get_next_u64(this);

    Ok(RNWInteger::new(u64_to_i64_shifted(next_u64)))
}

pub fn native_rng_unit(this: RNWObjectRef, _: &[RNWObjectRef]) -> RWResult<RNWObjectRef> {
    let next_u64 = get_next_u64(this);

    Ok(RNWFloat::new((next_u64 as f64) / ((u64::MAX as f64) + 1.0)))
}

pub fn native_rng_random_bool(this: RNWObjectRef, _: &[RNWObjectRef]) -> RWResult<RNWObjectRef> {
    let next_u64 = get_next_u64(this);

    let high = next_u64 >> 32; // старшие 32 бита
    let low = next_u64 as u32; // младшие 32 бита

    Ok(RNWBoolean::new(high > (low as u64)))
}

pub fn native_rng_random_range(
    this: RNWObjectRef,
    args: &[RNWObjectRef],
) -> RWResult<RNWObjectRef> {
    let mut borrow = this.borrow_mut();
    let this = borrow
        .as_any_mut()
        .downcast_mut::<RNWRandomNumberGenerator>()
        .unwrap();

    let (start, end) = {
        let start_borrow = args.get(0).unwrap().borrow();
        let end_borrow = args.get(1).unwrap().borrow();

        let start = start_borrow.value().downcast_ref::<i64>().unwrap();
        let end = end_borrow.value().downcast_ref::<i64>().unwrap();

        (start.clone(), end.clone())
    };

    Ok(RNWInteger::new(this.rng.random_range::<i64, _>(start..end)))
}

thread_local! {
    static RNG_NATIVE_FIELDS: Lazy<RefCell<HashMap<&'static str, RNWObjectRef>>> = Lazy::new(|| {
        let mut map = HashMap::new();

        map.insert("positive", RNWMethod::new(RNWRegisteredNativeMethod::new(
            "Rng.positive".to_string(),
            Rc::new(native_rng_positive),
            vec![RNWRandomNumberGenerator::rnw_type_id()]
        )));
        map.insert("negative", RNWMethod::new(RNWRegisteredNativeMethod::new(
            "Rng.negative".to_string(),
            Rc::new(native_rng_negative),
            vec![RNWRandomNumberGenerator::rnw_type_id()]
        )));
        map.insert("rand_int", RNWMethod::new(RNWRegisteredNativeMethod::new(
            "Rng.rand_int".to_string(),
            Rc::new(native_rng_random_int),
            vec![RNWRandomNumberGenerator::rnw_type_id()]
        )));
        map.insert("unit", RNWMethod::new(RNWRegisteredNativeMethod::new(
            "Rng.unit".to_string(),
            Rc::new(native_rng_unit),
            vec![RNWRandomNumberGenerator::rnw_type_id()]
        )));
        map.insert("rand_bool", RNWMethod::new(RNWRegisteredNativeMethod::new(
            "Rng.rand_bool".to_string(),
            Rc::new(native_rng_random_bool),
            vec![RNWRandomNumberGenerator::rnw_type_id()]
        )));
        map.insert("rand_range", RNWMethod::new(RNWRegisteredNativeMethod::new(
            "Rng.rand_range".to_string(),
            Rc::new(native_rng_random_range),
            vec![RNWRandomNumberGenerator::rnw_type_id(), RNWInteger::rnw_type_id(), RNWInteger::rnw_type_id()]
        )));

        RefCell::new(map)
    });
}

// Integer implements
impl RNWRandomNumberGenerator {
    pub fn new() -> RNWObjectRef {
        let seed = rand::random::<u64>();
        Rc::new(RefCell::new(Self {
            seed,
            rng: Pcg32::seed_from_u64(seed),
        }))
    }

    pub fn new_with_seed(seed_obj: &RNWObjectRef) -> RNWObjectRef {
        let seed = {
            let seed_borrow = seed_obj.borrow();
            seed_borrow.value().downcast_ref::<i64>().unwrap().clone()
        };
        let seed = i64_to_u64_shifted(seed);
        Rc::new(RefCell::new(Self {
            seed,
            rng: Pcg32::seed_from_u64(seed),
        }))
    }

    pub fn is_type_equals(other: &RNWObjectRef) -> bool {
        Self::rnw_type_id() == other.borrow().rnw_type_id()
    }

    pub fn type_name() -> &'static str {
        "Rng"
    }

    assign_rnw_type_id!();
}

impl RNWObject for RNWRandomNumberGenerator {
    fn rnw_type_id(&self) -> RNWTypeId {
        Self::rnw_type_id()
    }
    fn type_name(&self) -> &'static str {
        Self::type_name()
    }
    fn display(&self) -> String {
        format!("<Rng seed={:#x}>", self.seed)
    }
    fn value(&self) -> &dyn Any {
        &self.rng
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_attr(&self, name: &str) -> Option<RNWObjectRef> {
        RNG_NATIVE_FIELDS.with(|methods| methods.borrow().get(name).cloned())
    }
}

fn native_type_rng_new(_: RNWObjectRef, _: &[RNWObjectRef]) -> RWResult<RNWObjectRef> {
    Ok(RNWRandomNumberGenerator::new())
}

fn native_type_rng_from_seed(_: RNWObjectRef, args: &[RNWObjectRef]) -> RWResult<RNWObjectRef> {
    Ok(RNWRandomNumberGenerator::new_with_seed(
        args.get(0).unwrap(),
    ))
}

pub(super) fn register() -> Rc<RefCell<RNWType>> {
    register_cast(
        RNWRandomNumberGenerator::rnw_type_id(),
        RNWString::rnw_type_id(),
        |obj| Ok(RNWString::new(obj.display())),
    );

    let mut type_fields = HashMap::new();

    type_fields.insert(
        "new".to_string(),
        RNWMethod::new(RNWRegisteredNativeMethod::new(
            "Rng.new".to_string(),
            Rc::new(native_type_rng_new),
            vec![RNWType::rnw_type_id()],
        )),
    );
    type_fields.insert(
        "from_seed".to_string(),
        RNWMethod::new(RNWRegisteredNativeMethod::new(
            "Rng.from_seed".to_string(),
            Rc::new(native_type_rng_from_seed),
            vec![RNWType::rnw_type_id(), RNWInteger::rnw_type_id()],
        )),
    );

    RNWType::new_with_fields(
        RNWRandomNumberGenerator::rnw_type_id(),
        RNWRandomNumberGenerator::type_name(),
        type_fields,
    )
}
