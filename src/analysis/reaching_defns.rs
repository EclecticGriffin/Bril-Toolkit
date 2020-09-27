use super::prelude::*;
use std::collections::HashSet;

type Data = HashSet<VarDef>;

#[derive(Hash, Clone, Eq, PartialEq)]
pub struct VarDef(Var, usize);

impl Display for VarDef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}_{}", self.0, self.1)
    }
}


fn merge(input: Vec<&Data>) -> Data {
    let mut out = Data::new();
    for set in input {
        out = out.union(set).cloned().collect()
    }
    out
}

fn transfer(input: &Data, instrs: &Block, idx: usize) -> Data {
    let mut out = Data::new();

    for instr in instrs.0.iter() {
        match instr {
            Instr::Const { dest, .. } | Instr::Value { dest, .. } => {
                let new = VarDef(*dest, idx);
                out.retain(|x| {x.0 != *dest});
                out.insert(new);
            }
            _ => {}
        }
    }
    input.union(&out).cloned().collect()
}

pub fn reaching_definitions(nodes: &[Rc<Node>]) -> Vec<AnalysisNode<Data>> {
    worklist_solver(nodes, Data::new(), transfer, merge, Direction::Forward)
}
