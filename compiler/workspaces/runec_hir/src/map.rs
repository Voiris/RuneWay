use crate::ids::HirId;
use crate::item::HirItem;

pub struct HirMap<'src> {
    items: Vec<HirItem<'src>>,
}

impl<'src> HirMap<'src> {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Returns the `HirId` that will be assigned by the next `push`.
    /// Needed when an item's id must be known before the item is constructed.
    pub fn reserve_id(&self) -> HirId {
        HirId::from_usize(self.items.len())
    }

    pub fn push(&mut self, item: HirItem<'src>) -> HirId {
        let id = HirId::from_usize(self.items.len());
        debug_assert_eq!(
            id, item.id(),
            "HirItem.id does not match its position in HirMap"
        );
        self.items.push(item);
        id
    }

    pub fn get(&self, id: HirId) -> &HirItem<'src> {
        &self.items[id.to_usize()]
    }

    pub fn get_mut(&mut self, id: HirId) -> &mut HirItem<'src> {
        &mut self.items[id.to_usize()]
    }

    pub fn iter(&self) -> impl Iterator<Item = (HirId, &HirItem<'src>)> {
        self.items.iter()
            .enumerate()
            .map(|(i, it)| (HirId::from_usize(i), it))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (HirId, &mut HirItem<'src>)> {
        self.items.iter_mut()
            .enumerate()
            .map(|(i, it)| (HirId::from_usize(i), it))
    }
}

impl<'src> Default for HirMap<'src> {
    fn default() -> Self { Self::new() }
}
