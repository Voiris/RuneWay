use runec_builtins::BuiltinId;

use crate::ids::{HirId, HirLocalId};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Res {
    Local(HirLocalId),
    Def(HirId),
    Builtin(BuiltinId),
}
