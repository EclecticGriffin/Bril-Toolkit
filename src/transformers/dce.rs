use std::collections::{HashSet};
use super::cfg::{Node};
use crate::serde_structs::structs::{Var, Instr, };

pub fn local_dce(node: &Node) {
    let mut block = node.contents.borrow_mut();
    let tmp = std::mem::replace(&mut block.0, Vec::new());
    block.0 = trivial_local_dce(tmp);
}


pub fn trivial_local_dce(instrs: Vec<Instr>) -> Vec<Instr> {

    let mut used = HashSet::<Var>::new();
    let mut defined = HashSet::<Var>::new();

    // this feels like a crime
    let mut i: Vec<Instr> = instrs.into_iter().rev().map(|x| -> (bool, Instr) {
        match x {
            Instr::Const {ref dest, .. } => {
                if used.contains(dest) {
                    used.remove(dest);
                    (true, x)
                } else if defined.contains(dest) {
                    (false, x)
                } else {
                    defined.insert(*dest);
                    (true, x)
                }
            },
            Instr::Value {ref dest, ref args, ..} => {
                if used.contains(dest) {
                    used.remove(dest);
                    for v in args.iter() {
                        used.insert(*v);
                    }
                    (true, x)
                } else if defined.contains(dest) {
                    (false, x)
                } else {
                    for v in args.iter() {
                        used.insert(*v);
                    }
                    defined.insert(*dest);
                    (true, x)
                }
            },
            Instr::Effect {ref args, .. } => {
                for v in args.iter() {
                    used.insert(*v);
                }
                (true, x)
            },

            Instr::Label {..} => (true, x)
        }
    })
    .filter(|(t, _)| {*t})
    .map(|(_, i)| {i})
    .collect();

    i.reverse();
    i
}

pub fn trivial_global_dce(nodes: &mut Vec<Instr>) {
    let mut repeat = true;

    let mut used = HashSet::<Var>::new();
    let mut defined = HashSet::<Var>::new();

    while repeat {

    for instr in nodes.iter() {
        match instr {
            Instr::Const { dest, .. } => { defined.insert(*dest); }
            Instr::Value { dest, args,.. } => {
                defined.insert(*dest);
                for arg in args.iter() {
                    used.insert(*arg);
                }}
            Instr::Effect { args,.. } => {
                for arg in args.iter() {
                    used.insert(*arg);
                }
            }
            _ => {}
        }
    }

    let mut delete_set = HashSet::<Var>::new();

    for key in defined.iter() {
        if !used.contains(key) {
            delete_set.insert(*key);
        }
    }

    nodes.retain(|x| -> bool {
        match x {
            Instr::Const { dest, .. } | Instr::Value { dest, .. } => {!delete_set.contains(dest)}
            _ => true
        }
    });

    repeat = !delete_set.is_empty();
}
}
