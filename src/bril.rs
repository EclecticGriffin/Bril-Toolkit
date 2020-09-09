use serde::{self, Deserialize, Serialize};

pub type Name = String;

#[derive(Serialize, Deserialize, Debug)]
pub struct Program {
    functions: Vec<Function>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Function {
    name: String,
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
    Label {label: String},
    Const {
        op: Op,
        dest: Var,
        #[serde(rename="type")]
        r_type: Type,
        value: Literal,
    },
    Value {
        op: Op,
        dest: Var,
        // #[serde(rename="type")]
        r#type: Type,

        #[serde(default="Vec::new")]
        args: Vec<Var>,

        #[serde(default="Vec::new")]
        funcs: Vec<Func>,

        #[serde(default="Vec::new")]
        labels: Vec<Label>,
    },
    Effect {
        op: Op,

        #[serde(default="Vec::new")]
        args: Vec<Var>,

        #[serde(default="Vec::new")]
        funcs: Vec<Func>,

        #[serde(default="Vec::new")]
        labels: Vec<Label>,
    },
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

#[derive(Serialize, Deserialize, Debug)]
pub struct Var(pub String);

#[derive(Serialize, Deserialize, Debug)]
pub struct Func(pub String);

#[derive(Serialize, Deserialize, Debug)]
pub struct Label(pub String);

#[derive(Serialize, Deserialize, Debug)]
pub struct FnHeaders {
    name: String,
    #[serde(rename = "type")]
    r_type: Type,
}
