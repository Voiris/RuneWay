use std::ops::{Add, Sub};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct BytePos(u32); // Enough for 4GB of source code

impl BytePos {
    pub const MAX: usize = u32::MAX as usize;

    pub const fn from_usize(n: usize) -> BytePos {
        assert!(n <= Self::MAX, "BytePos overflow");
        BytePos(n as u32)
    }

    pub const fn to_usize(&self) -> usize {
        self.0 as usize
    }

    pub const fn checked_add(self, rhs: usize) -> Option<BytePos> {
        match self.to_usize().checked_add(rhs) {
            Some(value) if value <= Self::MAX => Some(BytePos(value as u32)),
            _ => None,
        }
    }

    pub const fn checked_sub(self, rhs: usize) -> Option<BytePos> {
        match self.to_usize().checked_sub(rhs) {
            Some(value) => Some(BytePos(value as u32)),
            None => None,
        }
    }
}

impl Add for BytePos {
    type Output = BytePos;

    fn add(self, rhs: BytePos) -> BytePos {
        assert!(self.0 <= u32::MAX - rhs.0, "BytePos overflow");
        BytePos(self.0 + rhs.0)
    }
}

impl Add<usize> for BytePos {
    type Output = BytePos;

    fn add(self, rhs: usize) -> BytePos {
        assert!(rhs <= Self::MAX - self.0 as usize, "BytePos overflow");
        BytePos::from_usize(self.to_usize() + rhs)
    }
}

impl Sub for BytePos {
    type Output = BytePos;

    fn sub(self, rhs: BytePos) -> BytePos {
        assert!(self.0 >= rhs.0, "BytePos underflow");
        BytePos(self.0 - rhs.0)
    }
}

impl Sub<usize> for BytePos {
    type Output = BytePos;

    fn sub(self, rhs: usize) -> BytePos {
        assert!(self.0 as usize >= rhs, "BytePos underflow");
        BytePos::from_usize(self.to_usize() - rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::BytePos;

    #[test]
    #[should_panic(expected = "BytePos overflow")]
    #[cfg(target_pointer_width = "64")]
    fn from_usize_rejects_overflow() {
        BytePos::from_usize(BytePos::MAX + 1);
    }

    #[test]
    #[should_panic(expected = "BytePos overflow")]
    fn addition_rejects_overflow() {
        let _ = BytePos::from_usize(BytePos::MAX) + 1;
    }

    #[test]
    #[should_panic(expected = "BytePos underflow")]
    fn subtraction_rejects_underflow() {
        let _ = BytePos::from_usize(0) - 1;
    }

    #[test]
    fn checked_arithmetic_reports_bounds() {
        let position = BytePos::from_usize(10);

        assert_eq!(position.checked_add(5), Some(BytePos::from_usize(15)));
        assert_eq!(position.checked_sub(5), Some(BytePos::from_usize(5)));
        assert_eq!(BytePos::from_usize(BytePos::MAX).checked_add(1), None);
        assert_eq!(BytePos::from_usize(0).checked_sub(1), None);
    }
}
