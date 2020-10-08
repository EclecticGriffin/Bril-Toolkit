use std::fmt::{self, Display};
use serde::{self, Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Copy, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Op {
    Const,
    // Arith
    Add,
    Mul,
    Sub,
    Div,
    // Comparison
    Eq,
    Lt,
    Gt,
    Le,
    Ge,
    // Logic
    Not,
    And,
    Or,
    // Control
    Jmp,
    Br,
    Call,
    Ret,
    // Misc
    Id,
    Print,
    Nop,
    // Memory Extension
    Alloc,
    Free,
    Store,
    Load,
    PtrAdd,
    // Float Extension
    FAdd,
    FMul,
    FSub,
    FDiv,

    FEq,
    FLt,
    FLe,
    FGt,
    FGe,
    //SSA
    Phi,
}

impl Op {
    pub fn is_terminator(&self) -> bool {
        match self {
            Op::Jmp | Op::Br | Op::Ret => true,
            _ => false,
        }
    }

    pub fn is_commutative(&self) -> bool {
        match self {
            Op::Add | Op::Mul | Op::Eq | Op::And | Op::Or => true,
            Op::FAdd | Op::FMul | Op::FEq => true,
            _ => false
        }
    }
}

impl Display for Op {
    fn fmt(&self, f: & mut fmt::Formatter<'_>) -> fmt::Result {
        let op = format!("{:?}", &self);
        let op = op.trim().to_ascii_lowercase();
        write!(f, "{}", op)
    }
}
