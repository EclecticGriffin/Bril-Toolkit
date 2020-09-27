use super::prelude::*;
use super::dehydrated::set_union;
use std::collections::HashSet;

type Data = HashSet<Var>;

fn transfer(input: &Data, instrs: &Block, _idx: usize) -> Data {
    let mut used_vars = Data::new();
    let mut killed = Data::new();

    for instr in instrs.0.iter() {
        match instr {
            Instr::Const { dest, ..} => {
                killed.insert(*dest);
            }
            Instr::Value { dest, args, .. } => {
                for arg in args.iter() {
                    if !killed.contains(arg) {
                        used_vars.insert(*arg);
                    }
                }
                killed.insert(*dest);
            }
            Instr::Effect { args, ..} => {
                for arg in args.iter() {
                    if !killed.contains(arg) {
                        used_vars.insert(*arg);
                    }
                }
            }
            _ => {}
        }
    }


    // I'm sorry for this oneliner
    used_vars.union(&(input - &killed)).cloned().collect()
}

pub fn live_variables(nodes: &[Rc<Node>]) -> Vec<AnalysisNode<Data>> {
    worklist_solver(nodes, Data::new(), transfer, set_union, Direction::Backward)
}
