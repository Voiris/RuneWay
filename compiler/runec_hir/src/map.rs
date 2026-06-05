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
            id,
            item.id(),
            "HirItem.id does not match its position in HirMap"
        );
        self.items.push(item);
        id
    }

    pub fn get(&self, id: HirId) -> &HirItem<'src> {
        &self.items[id.to_usize()]
    }

    pub fn try_get(&self, id: HirId) -> Option<&HirItem<'src>> {
        self.items.get(id.to_usize())
    }

    pub fn get_mut(&mut self, id: HirId) -> &mut HirItem<'src> {
        &mut self.items[id.to_usize()]
    }

    pub fn try_get_mut(&mut self, id: HirId) -> Option<&mut HirItem<'src>> {
        self.items.get_mut(id.to_usize())
    }

    pub fn iter(&self) -> impl Iterator<Item = (HirId, &HirItem<'src>)> {
        self.items
            .iter()
            .enumerate()
            .map(|(i, it)| (HirId::from_usize(i), it))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (HirId, &mut HirItem<'src>)> {
        self.items
            .iter_mut()
            .enumerate()
            .map(|(i, it)| (HirId::from_usize(i), it))
    }
}

impl<'src> Default for HirMap<'src> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use runec_ast::SpannedStr;
    use runec_source::byte_pos::BytePos;
    use runec_source::source_map::SourceId;
    use runec_source::span::{Span, Spanned};

    use crate::ids::HirId;
    use crate::item::{HirFunction, HirItem};
    use crate::statement::HirBlock;
    use crate::ty::HirType;

    use super::HirMap;

    fn span() -> Span {
        Span::new(
            BytePos::from_usize(0),
            BytePos::from_usize(0),
            SourceId::from_usize(0),
        )
    }

    fn function(id: HirId) -> HirItem<'static> {
        HirItem::Function(HirFunction {
            id,
            name: SpannedStr::new("main", span()),
            params: Box::new([]),
            ret_ty: Spanned::new(HirType::Unit, span()),
            body: HirBlock {
                stmts: Box::new([]),
                tail: None,
                span: span(),
            },
            span: span(),
        })
    }

    #[test]
    fn try_get_returns_none_for_unknown_id() {
        let map = HirMap::new();
        assert!(map.try_get(HirId::from_usize(0)).is_none());
    }

    #[test]
    fn try_get_mut_accesses_existing_item() {
        let mut map = HirMap::new();
        let id = map.reserve_id();
        map.push(function(id));

        let HirItem::Function(function) = map.try_get_mut(id).expect("function should exist")
        else {
            panic!("expected function");
        };
        function.name.node = "renamed";

        assert_eq!(
            map.try_get(id).expect("function should exist").name().node,
            "renamed"
        );
    }
}
