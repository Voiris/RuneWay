use std::fmt;
use std::ops::BitXorAssign;

pub struct Hash64(u64);

impl Hash64 {
    pub const ZERO: Hash64 = Hash64(0);

    #[inline]
    pub const fn new(n: u64) -> Hash64 {
        Hash64(n)
    }

    #[inline]
    pub const fn into_u64(self) -> u64 {
        self.0
    }
}

impl BitXorAssign<u64> for Hash64 {
    fn bitxor_assign(&mut self, rhs: u64) {
        self.0 ^= rhs
    }
}

impl fmt::Debug for Hash64 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::LowerHex for Hash64 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::LowerHex::fmt(&self.0, f)
    }
}
