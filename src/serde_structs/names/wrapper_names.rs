use serde::{self, Deserialize, Serialize};
use super::name_mapper::Name;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Var(pub Name);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Label(pub Name);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct FnName(pub Name);

impl FnName {
    pub fn to_label(&self) -> Label {
        Label(self.0)
    }
}
