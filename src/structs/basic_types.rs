use std::fmt::{self,Display};
use std::ops::{Add, Mul, Div, Sub, BitAnd, BitOr, Not};

use serde::{self, Deserialize, Serialize};


#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Type {
    Int,
    Bool,
    Ptr(Box<Type>),
    Float,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Literal {
    Int(i64),
    Bool(bool),
    Float(f64)
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
            Literal::Float(x) => {write!(f, "{}", x)}
        }
    }
}

impl Add for Literal {
    type Output = Literal;

    fn add(self, rhs: Self) -> Self::Output {
        if let Literal::Int(v1) = self {
            if let Literal::Int(v2) = rhs {
                return Literal::Int(v1 + v2)
            }
        }
        panic!("Type mismatch, cannot add {:?} + {:?}", self, rhs);
}
}

impl Mul for Literal {
    type Output = Literal;

    fn mul(self, rhs: Self) -> Self::Output {
        if let Literal::Int(v1) = self {
            if let Literal::Int(v2) = rhs {
                return Literal::Int(v1 * v2)
            }
        }
        panic!("Type mismatch, cannot mul {:?} * {:?}", self, rhs);
    }
}

impl Div for Literal {
    type Output = Literal;

    fn div(self, rhs: Self) -> Self::Output {
        if let Literal::Int(v1) = self {
            if let Literal::Int(v2) = rhs {
                return Literal::Int(v1 / v2)
            }
        }
        panic!("Type mismatch, cannot div {:?} / {:?}", self, rhs);
    }
}

impl Sub for Literal {
    type Output = Literal;

    fn sub(self, rhs: Self) -> Self::Output {
        if let Literal::Int(v1) = self {
            if let Literal::Int(v2) = rhs {
                return Literal::Int(v1 - v2)
            }
        }
        panic!("Type mismatch, cannot sub {:?} - {:?}", self, rhs);
    }
}

impl BitAnd for Literal {
    type Output = Literal;

    fn bitand(self, rhs: Self) -> Self::Output {
        if let Literal::Bool(v1) = self {
            if let Literal::Bool(v2) = rhs {
                return Literal::Bool(v1 && v2)
            }
        }
        panic!("Type mismatch, cannot and {:?} && {:?}", self, rhs);
    }
}

impl BitOr for Literal {
    type Output = Literal;

    fn bitor(self, rhs: Self) -> Self::Output {
        if let Literal::Bool(v1) = self {
            if let Literal::Bool(v2) = rhs {
                return Literal::Bool(v1 || v2)
            }
        }
        panic!("Type mismatch, cannot or {:?} || {:?}", self, rhs);
    }
}

impl Not for Literal {
    type Output = Literal;

    fn not(self) -> Self::Output {
        if let Literal::Bool(v1) = self {
                return Literal::Bool(!v1)

        }
        panic!("Type mismatch, cannot not !{:?}", self);
    }
}

impl PartialEq for Literal {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Literal::Int(v1), Literal::Int(v2)) => v1 == v2,
            (Literal::Bool(v1), Literal::Bool(v2)) => v1 == v2,
            (Literal::Float(v1), Literal::Float(v2)) => v1 == v2,
            _ => false
        }
    }
}
