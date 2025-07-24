use std::fmt::Display;
use crate::runeway::compiler::ir::interface::const_value::IRConstValue;

pub enum IRInst {
    // General
    NoOp,
    Pop,
    Dup,
    Halt,

    // Literals and Constants
    PushInt(i64),
    PushFloat(f64),
    PushTrue,
    PushFalse,
    PushNull,
    LoadConst(IRConstValue),

    // Variables
    DefineFast(String),
    StoreFast(String),
    LoadFast(String),

    // Unary ops
    Neg,
    Not,

    // Binary ops
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,

    // Compare ops
    Eq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,

    // Functions
    Call(usize),
    Return,

    // Flow control
    Jump(usize),
    JumpIfTrue(usize),
    JumpIfFalse(usize),

    // Complex building
    BuiltList(usize),
    BuiltTuple(usize),
    BuiltDict(usize),
}

impl IRInst {
    pub fn to_u8(&self) -> u8 { // 0x00u8 -> 0xFFu8
        match self {
            // Group 0x0: General
            IRInst::NoOp => 0x00,
            IRInst::Pop => 0x01,
            IRInst::Dup => 0x02,
            IRInst::Halt => 0x03,

            // Group 0x1: Literals and Constants
            IRInst::PushInt( .. ) => 0x10,
            IRInst::PushFloat( .. ) => 0x11,
            IRInst::PushTrue => 0x12,
            IRInst::PushFalse => 0x13,
            IRInst::PushNull => 0x14,
            IRInst::LoadConst( .. ) => 0x15,

            // Group 0x2: Variables
            IRInst::DefineFast( .. ) => 0x20,
            IRInst::StoreFast( .. ) => 0x21,
            IRInst::LoadFast( .. ) => 0x22,

            // Group 0x3: Unary ops
            IRInst::Neg => 0x30,
            IRInst::Not => 0x31,

            // Group 0x4: Binary ops
            IRInst::Add => 0x40,
            IRInst::Sub => 0x41,
            IRInst::Mul => 0x42,
            IRInst::Div => 0x43,
            IRInst::Mod => 0x44,
            IRInst::Pow => 0x45,

            // Group 0x5: Compare ops
            IRInst::Eq => 0x50,
            IRInst::NotEq => 0x51,
            IRInst::Lt => 0x52,
            IRInst::LtEq => 0x53,
            IRInst::Gt => 0x54,
            IRInst::GtEq => 0x55,

            // Group 0x6: Functions
            IRInst::Call( .. ) => 0x60,
            IRInst::Return => 0x61,

            // Group 0x7: Flow control
            IRInst::Jump( .. ) => 0x70,
            IRInst::JumpIfTrue( .. ) => 0x71,
            IRInst::JumpIfFalse( .. ) => 0x72,

            // Group 0x8: Complex building
            IRInst::BuiltList( .. ) => 0x80,
            IRInst::BuiltTuple( .. ) => 0x81,
            IRInst::BuiltDict( .. ) => 0x82,
        }
    }
}

impl Display for IRInst {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // General
            IRInst::NoOp => write!(f, "NoOp"),
            IRInst::Pop => write!(f, "Pop"),
            IRInst::Dup => write!(f, "Dup"),
            IRInst::Halt => write!(f, "Halt"),

            // Literals and Constants
            IRInst::PushInt(i) => write!(f, "PushInt {}", i),
            IRInst::PushFloat(fl) => write!(f, "PushFloat {}", fl),
            IRInst::PushTrue => write!(f, "PushTrue"),
            IRInst::PushFalse => write!(f, "PushFalse"),
            IRInst::PushNull => write!(f, "PushNull"),
            IRInst::LoadConst(id) => write!(f, "PushConst {}", id),

            // Group 0x2: Variables
            IRInst::DefineFast(var) => write!(f, "DefineFast {}", var),
            IRInst::StoreFast(var) => write!(f, "StoreFast {}", var),
            IRInst::LoadFast(var) => write!(f, "LoadFast {}", var),

            // Group 0x3: Unary ops
            IRInst::Neg => write!(f, "Neg"),
            IRInst::Not => write!(f, "Not"),

            // Group 0x4: Binary ops
            IRInst::Add => write!(f, "Add"),
            IRInst::Sub => write!(f, "Sub"),
            IRInst::Mul => write!(f, "Mul"),
            IRInst::Div => write!(f, "Div"),
            IRInst::Mod => write!(f, "Mod"),
            IRInst::Pow => write!(f, "Pow"),

            // Group 0x5: Compare ops
            IRInst::Eq => write!(f, "Eq"),
            IRInst::NotEq => write!(f, "NotEq"),
            IRInst::Lt => write!(f, "Lt"),
            IRInst::LtEq => write!(f, "LtEq"),
            IRInst::Gt => write!(f, "Gt"),
            IRInst::GtEq => write!(f, "GtEq"),

            // Group 0x6: Functions
            IRInst::Call(args) => write!(f, "Call {}", args),
            IRInst::Return => write!(f, "Return"),

            // Group 0x7: Flow control
            IRInst::Jump(to) => write!(f, "Jump {}", to),
            IRInst::JumpIfTrue(to) => write!(f, "JumpIfTrue {}", to),
            IRInst::JumpIfFalse(to) => write!(f, "JumpIfFalse {}", to),

            // Group 0x8: Complex building
            IRInst::BuiltList(size) => write!(f, "BuiltList {}", size),
            IRInst::BuiltTuple(size) => write!(f, "BuiltTuple {}", size),
            IRInst::BuiltDict(size) => write!(f, "BuiltDict {}", size),
        }
    }
}
