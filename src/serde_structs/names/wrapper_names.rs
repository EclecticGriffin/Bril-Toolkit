use serde::{self, Deserialize, Serialize};
use super::name_mapper::{namer,Name};
use std::fmt::Display;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Var(pub Name);

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Label(pub Name);

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FnName(pub Name);


impl Display for Var {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<Var: {}>", namer().get_string(&self.0))
    }
}

impl Display for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<Label: {}>", namer().get_string(&self.0))
    }
}

impl Display for FnName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<FnName: {}>", namer().get_string(&self.0))
    }
}
