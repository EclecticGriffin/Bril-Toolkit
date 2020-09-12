use serde::{self, Deserialize, Serialize};
use super::functions::{CFGFunction, Function};

#[derive(Serialize, Deserialize, Debug)]
pub struct Program {
    pub functions: Vec<Function>,
}

pub struct CFGProgram {
    pub functions: Vec<CFGFunction>
}

impl Program {
    pub fn determine_cfg(self) -> CFGProgram {
        CFGProgram {
            functions: self.functions.into_iter().map(|f| f.make_cfg()).collect()
        }
    }

}

impl CFGProgram {
    pub fn make_serializeable(self) -> Program {
        Program {
            functions: self.functions.into_iter().map(|f| f.make_serializeable()).collect()
        }
    }

}
