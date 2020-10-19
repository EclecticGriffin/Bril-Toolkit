mod basic_types;
mod functions;
mod program;
mod names;
mod operations;
mod instructions;

pub use names::namer;

// Collects the internal structures
pub use basic_types::{Literal, Type};
pub use names::{FnName, Var, Label};
pub use program::{Program, CFGProgram};
pub use instructions::Instr;
pub use operations::Op;
pub use functions::{CFGFunction, Function, FnHeaders};
