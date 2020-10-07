mod dataflow_core;
pub mod reaching_defns;
pub mod live_vars;
mod cprop;

mod prelude {
    pub use super::dataflow_core::{worklist_solver, AnalysisNode, Direction};
    pub use crate::transformers::cfg::{Node, Block};
    pub use crate::serde_structs::structs::{Instr, Var, FnHeaders};
    pub use std::rc::Rc;
    pub use std::fmt::Display;
}

pub const ALLOWED_VALUES: &[&str] = &["reaching_defns", "live"];

pub use reaching_defns::reaching_definitions;
pub use live_vars::live_variables;

// just add types!
pub mod dehydrated {
    use std::hash::Hash;
    use std::collections::HashSet;

    pub fn set_union<T: Eq + Hash + Clone>(input: Vec<&HashSet<T>>) -> HashSet<T> {
        let mut out = HashSet::new();
        for set in input {
            out = out.union(set).cloned().collect()
        }
        out
    }

    pub fn set_intersection<T: Eq + Hash + Clone>(input: Vec<&HashSet<T>>) -> HashSet<T> {
        let mut out = input[0].clone();
        let mut iter = input.iter();
        iter.next();
        for &set in iter {
            out = out.intersection(set).cloned().collect()
        }
        out
    }
}
