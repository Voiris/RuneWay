use std::fmt;

use runec_abi::{RUNTIME_PRINT, RUNTIME_PRINTLN, RuntimeFunctionId};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct BuiltinId(u32);

impl BuiltinId {
    pub const fn from_index(index: usize) -> Self {
        assert!(index <= u32::MAX as usize);
        Self(index as u32)
    }

    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ContractId(u32);

impl ContractId {
    pub const fn from_index(index: usize) -> Self {
        assert!(index <= u32::MAX as usize);
        Self(index as u32)
    }

    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TypeBits {
    B8,
    B16,
    B32,
    B64,
    B128,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum PrimitiveType {
    Str,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TypeConstraint {
    Implements(ContractId),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BuiltinReturn {
    Unit,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BuiltinLowering {
    Runtime(RuntimeFunctionId),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ContractDecl {
    pub canonical_name: &'static str,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct BuiltinDecl {
    pub name: &'static str,
    pub params: &'static [TypeConstraint],
    pub ret: BuiltinReturn,
    pub lowering: BuiltinLowering,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct BuiltinContractImpl {
    pub contract_id: ContractId,
    pub for_type: PrimitiveType,
}

pub const DISPLAY_CONTRACT: ContractId = ContractId::from_index(0);
pub const PRINT: BuiltinId = BuiltinId::from_index(0);
pub const PRINTLN: BuiltinId = BuiltinId::from_index(1);

const DISPLAY_PARAM: &[TypeConstraint] = &[TypeConstraint::Implements(DISPLAY_CONTRACT)];

pub const CONTRACTS: &[ContractDecl] = &[ContractDecl {
    canonical_name: "core::fmt::Display",
}];

pub const BUILTINS: &[BuiltinDecl] = &[
    BuiltinDecl {
        name: "print",
        params: DISPLAY_PARAM,
        ret: BuiltinReturn::Unit,
        lowering: BuiltinLowering::Runtime(RUNTIME_PRINT),
    },
    BuiltinDecl {
        name: "println",
        params: DISPLAY_PARAM,
        ret: BuiltinReturn::Unit,
        lowering: BuiltinLowering::Runtime(RUNTIME_PRINTLN),
    },
];

pub const BUILTIN_CONTRACT_IMPLS: &[BuiltinContractImpl] = &[BuiltinContractImpl {
    contract_id: DISPLAY_CONTRACT,
    for_type: PrimitiveType::Str,
}];

pub fn builtin_from_name(name: &str) -> Option<BuiltinId> {
    BUILTINS
        .iter()
        .position(|decl| decl.name == name)
        .map(BuiltinId::from_index)
}

pub fn builtin_decl(id: BuiltinId) -> Option<&'static BuiltinDecl> {
    BUILTINS.get(id.index())
}

pub fn contract_decl(id: ContractId) -> Option<&'static ContractDecl> {
    CONTRACTS.get(id.index())
}

pub fn primitive_implements(ty: PrimitiveType, contract_id: ContractId) -> bool {
    BUILTIN_CONTRACT_IMPLS.iter().any(|implementation| {
        implementation.for_type == ty && implementation.contract_id == contract_id
    })
}

impl fmt::Display for ContractId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match contract_decl(*self) {
            Some(decl) => formatter.write_str(decl.canonical_name),
            None => write!(formatter, "<unknown contract {}>", self.index()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BuiltinLowering, DISPLAY_CONTRACT, PRINTLN, PrimitiveType, builtin_decl, builtin_from_name,
        primitive_implements,
    };
    use runec_abi::{RUNTIME_PRINTLN, runtime_function};

    #[test]
    fn resolves_builtin_from_its_language_name() {
        assert_eq!(builtin_from_name("println"), Some(PRINTLN));
        assert_eq!(builtin_from_name("unknown"), None);
    }

    #[test]
    fn display_is_implemented_for_strings() {
        assert!(primitive_implements(PrimitiveType::Str, DISPLAY_CONTRACT));
    }

    #[test]
    fn builtin_points_to_runtime_abi_declaration() {
        let builtin = builtin_decl(PRINTLN).expect("println declaration");
        assert_eq!(builtin.lowering, BuiltinLowering::Runtime(RUNTIME_PRINTLN));
        assert_eq!(
            runtime_function(RUNTIME_PRINTLN)
                .expect("runtime declaration")
                .symbol,
            "__runeway_println"
        );
    }
}
