use serde::{self, Deserialize, Serialize};

use crate::utils::name_mapper::Name;

#[derive(Serialize, Deserialize, Debug)]
pub struct Program {
    pub functions: Vec<Function>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Function {
    name: FnName,
    #[serde(default = "Vec::new")]
    args: Vec<FnHeaders>,

    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    r_type: Option<Type>,

    instrs: Vec<Instr>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Type {
    Int,
    Bool,
    Ptr(Box<Type>),
    Float,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Literal {
    Int(i64),
    Bool(bool),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Instr {
    #[serde(rename = "label")]
    Label { label: Label },
    Const {
        op: Op,
        dest: Var,
        #[serde(rename = "type")]
        r_type: Type,
        value: Literal,
    },
    Value {
        op: Op,
        dest: Var,

        #[serde(rename="type")]
        r_type: Type,

        #[serde(default = "Vec::new")]
        args: Vec<Var>,

        #[serde(default = "Vec::new")]
        funcs: Vec<Func>,

        #[serde(default = "Vec::new")]
        labels: Vec<Label>,
    },
    Effect {
        op: Op,

        #[serde(default = "Vec::new")]
        args: Vec<Var>,

        #[serde(default = "Vec::new")]
        funcs: Vec<Func>,

        #[serde(default = "Vec::new")]
        labels: Vec<Label>,
    },
}

impl Instr {
    pub fn is_label(&self) -> bool {
        if let Instr::Label { .. } = &self {
            true
        } else {
            false
        }
    }

    pub fn is_terminator(&self) -> bool {
        match self {
            Instr::Label { .. } => true,
            Instr::Const { .. } => false,
            Instr::Effect { op: operation, .. } | Instr::Value { op: operation, .. } => {
                operation.is_terminator()
            }
        }
    }

    pub fn extract_label(&self) -> Option<Label> {
        if let Instr::Label {label} = &self {
            Some(label.clone())
        } else {
            None
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
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
}

impl Op {
    pub fn is_terminator(&self) -> bool {
        match self {
            Op::Jmp | Op::Br | Op::Ret => true,
            _ => false,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Var(pub Name);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Func(pub Name);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Label(pub Name);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct FnName(pub Name);

impl FnName {
    pub fn to_label(&self) -> Label {
        Label(self.0)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FnHeaders {
    name: Var,
    #[serde(rename = "type")]
    r_type: Type,
}

// Modified Structs

use super::transformers::cfg::{Node, construct_basic_block, connect_basic_blocks};

impl Function {
    pub fn form_function_blocks(self) -> CFGFunction {
        let mut instrs = construct_basic_block(self.instrs);
        connect_basic_blocks(&mut instrs);

        CFGFunction {
            name: self.name,
            args: self.args,
            r_type: self.r_type,
            instrs
        }
    }
}

#[derive(Debug)]
pub struct CFGFunction {
    pub name: FnName,
    pub args: Vec<FnHeaders>,

    pub r_type: Option<Type>,

    pub instrs: Vec<Node>,
}


use crate::utils::name_mapper::namer;
use std::fmt::{self, Display};

impl Display for Type {
    fn fmt(&self, f: & mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Type::Int => {write!(f, "int")}
            Type::Bool => {write!(f, "bool")}
            Type::Ptr(x) => {write!(f, "ptr<{}>", x)}
            Type::Float => {write!(f, "float")}
        }
    }
}

impl Display for Literal {
    fn fmt(&self, f: & mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Literal::Int(x) => {write!(f, "{}", x)}
            Literal::Bool(x) => {write!(f, "{}", x)}
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

impl Display for Instr {
    fn fmt(&self, f: & mut fmt::Formatter<'_>) -> fmt::Result {
        let namer = namer();
        match &self {
            Instr::Label { label } => {
                write!(f, ".{}", namer.get_string(&label.0))
            }
            Instr::Const { op, dest, r_type, value } => {
                write!(f, "{}: {} = const {}", namer.get_string(&dest.0), r_type, value)
            }
            Instr::Value { op, dest, r_type, args, funcs, labels } => {
                write!(f, "{}: {} = {}", namer.get_string(&dest.0), r_type, op);
                for func in funcs.iter(){
                    write!(f, " {}", namer.get_string(&func.0));
                }

                for arg in args.iter() {
                    write!(f," {}", namer.get_string(&arg.0));
                }

                for label in labels.iter() {
                    write!(f, " {}", namer.get_string(&label.0));
                }
                write!(f,"")
            }
            Instr::Effect { op, args, funcs, labels } => {
                write!(f, "{}", op);
                for func in funcs.iter(){
                    write!(f, " {}", namer.get_string(&func.0));
                }

                for arg in args.iter() {
                    write!(f," {}", namer.get_string(&arg.0));
                }

                for label in labels.iter() {
                    write!(f, " {}", namer.get_string(&label.0));
                }
                write!(f,"")
            }
        }
    }
}

impl Display for CFGFunction {
    fn fmt(&self, f: & mut fmt::Formatter<'_>) -> fmt::Result {
        let namer = namer();
        write!(f, "== Function: {} ==\n", namer.get_string(&self.name.0));
        write!(f, "args: {:?}\n", &self.args);
        if let Some(x) = &self.r_type {
            write!(f, "returns {}\n", x);
        }
        for node in self.instrs.iter() {
            write!(f, "\n{}\n", node);
        }
        write!(f, "")
    }
}
