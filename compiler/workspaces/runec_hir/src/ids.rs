#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HirId(u32);

impl HirId {
    pub fn from_usize(id: usize) -> HirId {
        assert!(id <= u32::MAX as usize, "HirId overflow");
        HirId(id as u32)
    }
    pub fn to_usize(self) -> usize {
        self.0 as usize
    }
}

/// Index of a local variable or parameter within a specific function body.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HirLocalId(u32);

impl HirLocalId {
    pub fn from_usize(id: usize) -> HirLocalId {
        assert!(id <= u32::MAX as usize, "HirLocalId overflow");
        HirLocalId(id as u32)
    }
    pub fn to_usize(self) -> usize {
        self.0 as usize
    }
}
