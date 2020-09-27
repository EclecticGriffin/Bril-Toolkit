use std::fmt::{self, Display};
use serde::{self, Deserialize, Serialize};

use super::names::{FnName, Label, namer, Var};
use super::basic_types::{Literal, Type};
use super::operations::Op;

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
        funcs: Vec<FnName>,

        #[serde(default = "Vec::new")]
        labels: Vec<Label>,
    },
    Effect {
        op: Op,

        #[serde(default = "Vec::new")]
        args: Vec<Var>,

        #[serde(default = "Vec::new")]
        funcs: Vec<FnName>,

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
            Some(*label)
        } else {
            None
        }
    }
}

// impl Label {
//     pub fn make_instr(self) -> Instr {
//         Instr::Label {label: self}
//     }
// }

impl Display for Instr {
    fn fmt(&self, f: & mut fmt::Formatter<'_>) -> fmt::Result {
        let namer = namer();
        match &self {
            Instr::Label { label } => {
                write!(f, ".{}", namer.get_string(&label.0))
            }
            Instr::Const { dest, r_type, value, .. } => {
                write!(f, "{}: {} = const {}", namer.get_string(&dest.0), r_type, value)
            }
            Instr::Value { op, dest, r_type, args, funcs, labels } => {
                write!(f, "{}: {} = {}", namer.get_string(&dest.0), r_type, op)?;
                for func in funcs.iter(){
                    write!(f, " {}", namer.get_string(&func.0))?
                }

                for arg in args.iter() {
                    write!(f," {}", namer.get_string(&arg.0))?
                }

                for label in labels.iter() {
                    write!(f, " {}", namer.get_string(&label.0))?
                }
                Ok(())
            }
            Instr::Effect { op, args, funcs, labels } => {
                write!(f, "{}", op)?;
                for func in funcs.iter(){
                    write!(f, " {}", namer.get_string(&func.0))?;
                }

                for arg in args.iter() {
                    write!(f," {}", namer.get_string(&arg.0))?;
                }

                for label in labels.iter() {
                    write!(f, " {}", namer.get_string(&label.0))?;
                }
                Ok(())
            }
        }
    }
}
