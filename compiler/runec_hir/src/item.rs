use runec_ast::SpannedStr;
use runec_source::span::Span;

use crate::ids::HirId;
use crate::statement::HirBlock;
use crate::ty::SpannedHirType;

#[derive(Debug, PartialEq)]
pub enum HirItem<'src> {
    Struct(HirStruct<'src>),
    Enum(HirEnum<'src>),
    Function(HirFunction<'src>),
}

impl<'src> HirItem<'src> {
    pub fn id(&self) -> HirId {
        match self {
            HirItem::Struct(s) => s.id,
            HirItem::Enum(e) => e.id,
            HirItem::Function(f) => f.id,
        }
    }

    pub fn name(&self) -> &SpannedStr<'src> {
        match self {
            HirItem::Struct(s) => &s.name,
            HirItem::Enum(e) => &e.name,
            HirItem::Function(f) => &f.name,
        }
    }

    pub fn span(&self) -> Span {
        match self {
            HirItem::Struct(s) => s.span,
            HirItem::Enum(e) => e.span,
            HirItem::Function(f) => f.span,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct HirFunction<'src> {
    pub id: HirId,
    pub name: SpannedStr<'src>,
    pub params: Box<[HirFunctionParam<'src>]>,
    pub ret_ty: SpannedHirType<'src>,
    pub body: HirBlock<'src>,
    pub span: Span,
}

#[derive(Debug, PartialEq)]
pub struct HirFunctionParam<'src> {
    pub name: SpannedStr<'src>,
    pub ty: SpannedHirType<'src>,
    pub span: Span,
}

#[derive(Debug, PartialEq)]
pub struct HirStruct<'src> {
    pub id: HirId,
    pub name: SpannedStr<'src>,
    pub fields: Box<[HirField<'src>]>,
    pub span: Span,
}

#[derive(Debug, PartialEq)]
pub struct HirField<'src> {
    pub name: SpannedStr<'src>,
    pub ty: SpannedHirType<'src>,
    pub span: Span,
}

#[derive(Debug, PartialEq)]
pub struct HirEnum<'src> {
    pub id: HirId,
    pub name: SpannedStr<'src>,
    pub variants: Box<[HirVariant<'src>]>,
    pub span: Span,
}

#[derive(Debug, PartialEq)]
pub struct HirVariant<'src> {
    pub name: SpannedStr<'src>,
    pub payload: HirVariantPayload<'src>,
    pub span: Span,
}

#[derive(Debug, PartialEq)]
pub enum HirVariantPayload<'src> {
    Unit,
    Tuple(Box<[SpannedHirType<'src>]>),
    Struct(Box<[HirField<'src>]>),
}

#[cfg(test)]
mod tests {
    use runec_ast::SpannedStr;
    use runec_source::byte_pos::BytePos;
    use runec_source::source_map::SourceId;
    use runec_source::span::{Span, Spanned};

    use super::{HirEnum, HirFunction, HirItem, HirStruct};
    use crate::ids::HirId;
    use crate::statement::HirBlock;
    use crate::ty::HirType;

    fn span(lo: usize, hi: usize) -> Span {
        Span::new(BytePos::from_usize(lo), BytePos::from_usize(hi), SourceId::from_usize(0))
    }

    #[test]
    fn exposes_item_identity_and_span() {
        let struct_span = span(1, 7);
        let enum_span = span(8, 14);
        let function_span = span(15, 24);
        let items = [
            (
                HirItem::Struct(HirStruct {
                    id: HirId::from_usize(0),
                    name: SpannedStr::new("Record", struct_span),
                    fields: Box::new([]),
                    span: struct_span,
                }),
                HirId::from_usize(0),
                "Record",
                struct_span,
            ),
            (
                HirItem::Enum(HirEnum {
                    id: HirId::from_usize(1),
                    name: SpannedStr::new("Choice", enum_span),
                    variants: Box::new([]),
                    span: enum_span,
                }),
                HirId::from_usize(1),
                "Choice",
                enum_span,
            ),
            (
                HirItem::Function(HirFunction {
                    id: HirId::from_usize(2),
                    name: SpannedStr::new("main", function_span),
                    params: Box::new([]),
                    ret_ty: Spanned::new(HirType::Unit, function_span),
                    body: HirBlock { stmts: Box::new([]), tail: None, span: function_span },
                    span: function_span,
                }),
                HirId::from_usize(2),
                "main",
                function_span,
            ),
        ];

        for (item, expected_id, expected_name, expected_span) in items {
            assert_eq!(item.id(), expected_id);
            assert_eq!(item.name().node, expected_name);
            assert_eq!(item.span(), expected_span);
        }
    }
}
