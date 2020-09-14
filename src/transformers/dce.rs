use std::collections::HashSet;
use super::cfg::Node;
use crate::serde_structs::structs::{Var, Instr};
use std::ops::Add;
use std::rc::Rc;

fn dce_inner(instrs: &mut Vec<Node>) -> bool {
    let mut assigned = HashSet::<Var>::new();
    let mut used = HashSet::<Var>::new();
    let mut delete_set = HashSet::<Var>::new();

    for Node(node) in instrs.iter() {
        for instr in node.contents.borrow_mut().iter() {
            match instr {
                Instr::Const { dest, .. } => {
                    assigned.insert(dest.clone());
                }
                Instr::Value { dest, args, ..} => {
                    assigned.insert(dest.clone());
                    for arg in args {
                        used.insert(arg.clone());
                    }
                }
                Instr::Effect { args ,..} => {
                    for arg in args {
                        used.insert(arg.clone());
                    }
                }
                Instr::Label { label } => {}
            }
        }
    }

    for var in assigned {
        if !used.contains(&var) {
            delete_set.insert(var);
        }
    }


    for Node(node) in instrs.iter_mut() {
        // this is dangerous, but probably fine as long as the
        // cfg is accessed in a consistent way
        let mut contents = node.contents.borrow_mut();
        contents.retain(|x| {
            if let Instr::Const { dest, .. } | Instr::Value {dest,..} = x {
                !delete_set.contains(&dest)
            } else {
                true
            }
        }
        );
    }

    !delete_set.is_empty()
}

pub fn dce(instrs: &mut Vec<Node>) {

    // this code is silly and I feel silly for writing it
    while dce_inner(instrs) {}

}
