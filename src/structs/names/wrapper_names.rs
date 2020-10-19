use serde::{self, Deserialize, Serialize};
use super::name_mapper::{namer,Name};
use std::fmt::{Display, Debug};

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct Var(pub Name);

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct Label(pub Name);

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct FnName(pub Name);

impl Var {
    pub fn unwrap(&self) -> u64 {
        self.0.unwrap()
    }

    pub fn to_name(self) -> Name {
        self.0
    }
}

impl Label {
    pub fn unwrap(&self) -> u64 {
        self.0.unwrap()
    }

    pub fn to_name(self) -> Name {
        self.0
    }
}

impl FnName {
    pub fn unwrap(&self) -> u64 {
        self.0.unwrap()
    }

    pub fn to_name(self) -> Name {
        self.0
    }
}

impl Name {
    pub fn unwrap(&self) -> u64 {
        self.0
    }

    pub fn to_var(self) -> Var {
        Var(self)
    }

    pub fn to_label(self) -> Label {
        Label(self)
    }

    pub fn to_fname(self) -> FnName {
        FnName(self)
    }
}


impl Debug for Var {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<Var: {} [{:?}]>", namer().get_string(&self.0), &(self.0).0)
    }
}


impl Debug for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<Label: {} [{:?}]>", namer().get_string(&self.0), &self.0)
    }
}

impl Debug for FnName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<FnName: {} [{:?}]>", namer().get_string(&self.0), &self.0)
    }
}

impl Display for Var {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0)
    }
}

impl Display for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0)
    }
}

impl Display for FnName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0)
    }
}
