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
            HirItem::Struct(s)   => s.id,
            HirItem::Enum(e)     => e.id,
            HirItem::Function(f) => f.id,
        }
    }

    pub fn name(&self) -> &SpannedStr<'src> {
        match self {
            HirItem::Struct(s)   => &s.name,
            HirItem::Enum(e)     => &e.name,
            HirItem::Function(f) => &f.name,
        }
    }

    pub fn span(&self) -> Span {
        match self {
            HirItem::Struct(s)   => s.span,
            HirItem::Enum(e)     => e.span,
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
