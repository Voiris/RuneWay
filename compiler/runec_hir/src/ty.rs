use runec_source::span::Spanned;
use crate::expression::SpannedHirExpr;
use crate::ids::HirId;
use crate::path::{HirGenericArg, HirPath};

#[derive(Debug, PartialEq)]
pub enum HirType<'src> {
    /// Path not yet resolved by name resolution.
    Unresolved(HirPath<'src>),

    // --- filled in by the resolver after name resolution ---
    Primitive(HirPrimitiveTy),
    Struct { def: HirId, generics: Box<[HirGenericArg<'src>]> },
    Enum   { def: HirId, generics: Box<[HirGenericArg<'src>]> },

    Unit,
    Tuple(Box<[SpannedHirType<'src>]>),
    Array {
        elem: Box<SpannedHirType<'src>>,
        len: Box<SpannedHirExpr<'src>>,
    },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum HirPrimitiveTy {
    I8, I16, I32, I64, I128,
    U8, U16, U32, U64, U128,
    F32, F64,
    Bool, Char, Str,
}

pub type SpannedHirType<'src> = Spanned<HirType<'src>>;
