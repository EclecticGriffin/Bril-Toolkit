use std::fmt::{self,Display};
use serde::{self, Deserialize, Serialize};


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
