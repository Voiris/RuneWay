use std::ops::{Add, Sub};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct BytePos(u32); // Enough for 4GB of source code

impl BytePos {
    pub const MAX: usize = u32::MAX as usize;

    pub const fn from_usize(n: usize) -> BytePos {
        BytePos(n as u32)
    }

    pub const fn to_usize(&self) -> usize {
        self.0 as usize
    }
}

impl Add for BytePos {
    type Output = BytePos;

    fn add(self, rhs: BytePos) -> BytePos {
        BytePos(self.0 + rhs.0)
    }
}

impl Add<usize> for BytePos {
    type Output = BytePos;

    fn add(self, rhs: usize) -> BytePos {
        BytePos::from_usize(self.to_usize() + rhs)
    }
}

impl Sub for BytePos {
    type Output = BytePos;

    fn sub(self, rhs: BytePos) -> BytePos {
        BytePos(self.0 - rhs.0)
    }
}

impl Sub<usize> for BytePos {
    type Output = BytePos;

    fn sub(self, rhs: usize) -> BytePos {
        BytePos::from_usize(self.to_usize() - rhs)
    }
}
