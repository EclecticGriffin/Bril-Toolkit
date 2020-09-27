mod basic_types;
mod functions;
mod program;
mod names;
mod operations;
mod instructions;

pub use names::namer;

pub mod structs {
    // Collects the internal structures
    pub use super::basic_types::{Literal, Type};
    pub use super::names::{FnName, Var, Label};
    pub use super::program::{Program, CFGProgram};
    pub use super::instructions::Instr;
    pub use super::operations::Op;
    pub use super::functions::{CFGFunction, Function, FnHeaders};
}
