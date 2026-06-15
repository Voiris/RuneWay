#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MirFunctionId(u32);

impl MirFunctionId {
    pub fn from_usize(id: usize) -> Self {
        assert!(id <= u32::MAX as usize, "MirFunctionId overflow");
        Self(id as u32)
    }

    pub fn to_usize(self) -> usize {
        self.0 as usize
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MirBlockId(u32);

impl MirBlockId {
    pub fn from_usize(id: usize) -> Self {
        assert!(id <= u32::MAX as usize, "MirBlockId overflow");
        Self(id as u32)
    }

    pub fn to_usize(self) -> usize {
        self.0 as usize
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MirLocalId(u32);

impl MirLocalId {
    pub fn from_usize(id: usize) -> Self {
        assert!(id <= u32::MAX as usize, "MirLocalId overflow");
        Self(id as u32)
    }

    pub fn to_usize(self) -> usize {
        self.0 as usize
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MirConstantId(u32);

impl MirConstantId {
    pub fn from_usize(id: usize) -> Self {
        assert!(id <= u32::MAX as usize, "MirConstantId overflow");
        Self(id as u32)
    }

    pub fn to_usize(self) -> usize {
        self.0 as usize
    }
}
