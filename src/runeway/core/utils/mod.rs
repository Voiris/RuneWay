use std::rc::Rc;

pub mod assert;
pub mod serde;

const SHIFT: u64 = 1u64 << 63; // = 2^63 = i64::MIN as u64

pub fn i64_to_u64_shifted(x: i64) -> u64 {
    (x as u64).wrapping_add(SHIFT)
}

pub fn u64_to_i64_shifted(x: u64) -> i64 {
    x.wrapping_sub(SHIFT) as i64
}

pub fn get_rc_id<T: ?Sized>(x: Rc<T>) -> u64 {
    let ptr = Rc::<T>::as_ptr(&x) as *const ();
    ptr as u64
}
