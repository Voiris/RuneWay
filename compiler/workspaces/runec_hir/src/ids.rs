pub struct HirId(u32);

impl HirId {
    pub fn from_usize(id: usize) -> HirId {
        assert!(id <= u32::MAX as usize, "HirId overflow");
        HirId(id as u32)
    }
    pub fn to_usize(&self) -> usize {
        self.0 as usize
    }
}