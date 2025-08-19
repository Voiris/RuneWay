use std::fmt::Display;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum Opcode {
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
    LoadConst(usize),

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

impl Opcode {
    pub fn to_u8(&self) -> u8 {
        // 0x00u8 -> 0xFFu8
        match self {
            // Group 0x0: General
            Opcode::NoOp => 0x00,
            Opcode::Pop => 0x01,
            Opcode::Dup => 0x02,
            Opcode::Halt => 0x03,

            // Group 0x1: Literals and Constants
            Opcode::PushInt(..) => 0x10,
            Opcode::PushFloat(..) => 0x11,
            Opcode::PushTrue => 0x12,
            Opcode::PushFalse => 0x13,
            Opcode::PushNull => 0x14,
            Opcode::LoadConst(..) => 0x15,

            // Group 0x2: Variables
            Opcode::DefineFast(..) => 0x20,
            Opcode::StoreFast(..) => 0x21,
            Opcode::LoadFast(..) => 0x22,

            // Group 0x3: Unary ops
            Opcode::Neg => 0x30,
            Opcode::Not => 0x31,

            // Group 0x4: Binary ops
            Opcode::Add => 0x40,
            Opcode::Sub => 0x41,
            Opcode::Mul => 0x42,
            Opcode::Div => 0x43,
            Opcode::Mod => 0x44,
            Opcode::Pow => 0x45,

            // Group 0x5: Compare ops
            Opcode::Eq => 0x50,
            Opcode::NotEq => 0x51,
            Opcode::Lt => 0x52,
            Opcode::LtEq => 0x53,
            Opcode::Gt => 0x54,
            Opcode::GtEq => 0x55,

            // Group 0x6: Functions
            Opcode::Call(..) => 0x60,
            Opcode::Return => 0x61,

            // Group 0x7: Flow control
            Opcode::Jump(..) => 0x70,
            Opcode::JumpIfTrue(..) => 0x71,
            Opcode::JumpIfFalse(..) => 0x72,

            // Group 0x8: Complex building
            Opcode::BuiltList(..) => 0x80,
            Opcode::BuiltTuple(..) => 0x81,
            Opcode::BuiltDict(..) => 0x82,
        }
    }
}

impl Display for Opcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // General
            Opcode::NoOp => write!(f, "NoOp"),
            Opcode::Pop => write!(f, "Pop"),
            Opcode::Dup => write!(f, "Dup"),
            Opcode::Halt => write!(f, "Halt"),

            // Literals and Constants
            Opcode::PushInt(i) => write!(f, "PushInt {}", i),
            Opcode::PushFloat(fl) => write!(f, "PushFloat {}", fl),
            Opcode::PushTrue => write!(f, "PushTrue"),
            Opcode::PushFalse => write!(f, "PushFalse"),
            Opcode::PushNull => write!(f, "PushNull"),
            Opcode::LoadConst(id) => write!(f, "PushConst {}", id),

            // Group 0x2: Variables
            Opcode::DefineFast(var) => write!(f, "DefineFast {}", var),
            Opcode::StoreFast(var) => write!(f, "StoreFast {}", var),
            Opcode::LoadFast(var) => write!(f, "LoadFast {}", var),

            // Group 0x3: Unary ops
            Opcode::Neg => write!(f, "Neg"),
            Opcode::Not => write!(f, "Not"),

            // Group 0x4: Binary ops
            Opcode::Add => write!(f, "Add"),
            Opcode::Sub => write!(f, "Sub"),
            Opcode::Mul => write!(f, "Mul"),
            Opcode::Div => write!(f, "Div"),
            Opcode::Mod => write!(f, "Mod"),
            Opcode::Pow => write!(f, "Pow"),

            // Group 0x5: Compare ops
            Opcode::Eq => write!(f, "Eq"),
            Opcode::NotEq => write!(f, "NotEq"),
            Opcode::Lt => write!(f, "Lt"),
            Opcode::LtEq => write!(f, "LtEq"),
            Opcode::Gt => write!(f, "Gt"),
            Opcode::GtEq => write!(f, "GtEq"),

            // Group 0x6: Functions
            Opcode::Call(args) => write!(f, "Call {}", args),
            Opcode::Return => write!(f, "Return"),

            // Group 0x7: Flow control
            Opcode::Jump(to) => write!(f, "Jump {}", to),
            Opcode::JumpIfTrue(to) => write!(f, "JumpIfTrue {}", to),
            Opcode::JumpIfFalse(to) => write!(f, "JumpIfFalse {}", to),

            // Group 0x8: Complex building
            Opcode::BuiltList(size) => write!(f, "BuiltList {}", size),
            Opcode::BuiltTuple(size) => write!(f, "BuiltTuple {}", size),
            Opcode::BuiltDict(size) => write!(f, "BuiltDict {}", size),
        }
    }
}
