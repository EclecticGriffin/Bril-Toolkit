use super::cfg::{Block, Node};
use super::dominance::DominanceTree;
use crate::serde_structs::namer;
use crate::serde_structs::structs::{Instr, Label, Op, Type, Var, FnHeaders};
use std::collections::{HashMap, HashSet, VecDeque};
use std::ops::{Index, IndexMut};
use std::rc::{Rc, Weak};

fn identify_definitions(nodes: &[Rc<Node>]) -> HashMap<Var, (HashSet<Label>, Type)> {
    let mut var_map: HashMap<Var, (HashSet<Label>, Type)> = HashMap::new();
    for block in nodes {
        let instrs: &Block = &block.contents.borrow();
        for instr in instrs.0.iter() {
            match instr {
                Instr::Const { dest, r_type, .. } | Instr::Value { dest, r_type, .. } => {
                    let contains: bool = var_map.contains_key(dest);

                    if contains {
                        var_map.get_mut(dest).unwrap().0.insert(block.label());
                    } else {
                        let mut set = HashSet::<Label>::with_capacity(1);
                        set.insert(block.label());
                        var_map.insert(*dest, (set, r_type.clone()));
                    }
                }
                _ => {}
            }
        }
    }
    var_map
}

fn insert_phi_nodes(
    nodes: &mut [Rc<Node>],
) -> (DominanceTree, HashMap<Var, (HashSet<Label>, Type)>) {
    let mut def_map = identify_definitions(nodes);
    let dom_tree = DominanceTree::new(nodes);
    for (var, (defs, r_type)) in def_map.iter_mut() {
        let mut queue: VecDeque<Label> = defs.iter().cloned().collect();

        if queue.len() != 1 {
            while let Some(block_label) = queue.pop_front() {
                for block in dom_tree.compute_frontier(&block_label) {
                    let node = dom_tree.lookup_node(&block);
                    let contents: &mut Block = &mut node.contents.borrow_mut();
                    if contents.len() != 1 {
                        for instr in contents.0.iter_mut() {
                            if let Instr::Value {
                                op: Op::Phi,
                                dest,
                                labels,
                                args,
                                ..
                            } = instr
                            {
                                if dest == var {
                                    args.push(*var);
                                    labels.push(block_label);
                                    break;
                                }
                            }
                        }
                    }
                    let new = Instr::Value {
                        op: Op::Phi,
                        dest: *var,
                        r_type: r_type.clone(),
                        args: vec![*var],
                        funcs: vec![],
                        labels: vec![block_label],
                    };
                    contents.0.insert(1, new);
                    defs.insert(node.label());

                    if !queue.contains(&node.label()) {
                        queue.push_back(node.label());
                    }
                }
            }
        }
    }
    (dom_tree, def_map)
}

struct RenameStack {
    var_stacks: HashMap<Var, Vec<Var>>,
    layer: i64,
    pop_list: Vec<HashMap<Var, usize>>,
}

impl RenameStack {
    // TODO: Fix this definition
    fn new(vars: std::collections::hash_map::Keys<Var, (HashSet<Label>, Type)>) -> Self {
        let mut stack_map = HashMap::<Var, Vec<Var>>::with_capacity(vars.len());
        for var in vars {
            stack_map.insert(*var, Vec::new());
        }

        RenameStack {
            var_stacks: stack_map,
            layer: -1,
            pop_list: vec![],
        }
    }

    fn increase_layer(&mut self) {
        self.layer += 1;
        self.pop_list.push(HashMap::new())
    }

    fn decrease_layer(&mut self) {
        for (var, count) in self.pop_list.last().unwrap() {
            for _ in 0..*count {
                self.var_stacks.get_mut(var).unwrap().pop();
            }
        }
        self.pop_list.pop();
        self.layer -= 1;
    }

    fn push_var(&mut self, old_name: &Var, new_name: Var) {
        self.var_stacks.get_mut(old_name).unwrap().push(new_name)
    }

    fn get_top(&self, old_name: &Var) -> Option<Var> {
        match self.var_stacks.get(old_name).unwrap().last() {
            Some(v) => Some(*v),
            None => None,
        }
    }
}

fn rename(node: &mut Rc<Node>, dom_tree: &DominanceTree, stack: &mut RenameStack, headers: &[Var]) {
    stack.increase_layer();
    {
        let contents: &mut Block = &mut node.contents.borrow_mut();
        let block = &mut contents.0;
        for instr in block.iter_mut() {
            match instr {
                // Constants will only define a new name
                Instr::Const { dest, .. } => {
                    let new_name = Var(namer().fresh(&dest.0));
                    *dest = new_name;
                    stack.push_var(dest, new_name);
                }
                // Phi instrs we only update the new name, not the args
                Instr::Value {
                    op: Op::Phi, dest, ..
                } => {
                    let new_name = Var(namer().fresh(&dest.0));
                    *dest = new_name;
                    stack.push_var(dest, new_name);
                }
                // Otherwise, update args then the name
                Instr::Value { op, dest, args, .. } => {
                    for arg in args.iter_mut() {
                        *arg = match stack.get_top(arg) {
                            Some(var) => var,
                            None if headers.contains(arg) => *arg,
                            _ => panic!("Unknown variable {}", arg),
                        }
                    }
                    let new_name = Var(namer().fresh(&dest.0));
                    *dest = new_name;
                    stack.push_var(dest, new_name);
                }
                // Only update args for effects
                Instr::Effect { args, .. } => {
                    for arg in args.iter_mut() {
                        *arg = match stack.get_top(arg) {
                            Some(var) => var,
                            None if headers.contains(arg) => *arg,
                            _ => panic!("Unknown variable {}", arg),
                        }
                    }
                }
                _ => {}
            }
        }
    }

    for successor in node.successor_refs() {
        let contents: &mut Block = &mut node.contents.borrow_mut();
        let block = &mut contents.0;
        for instr in block.iter_mut() {
            if let Instr::Value { op: Op::Phi, args, dest, labels, ..} = instr {
                let mut index:usize = 0;
                let mut found = false;
                for (idx, arg) in args.iter().enumerate() {
                    if arg == dest {
                        index = idx;
                        found = true;
                        break
                    }
                }
                if !found {
                    panic!("No arg to rename?")
                }

                let var = args[index];

                args[index] = stack.get_top(&var).unwrap();
                labels[index] = node.label()
            }
        }
    }

    for mut child in dom_tree.get_children(&node.label()) {
        rename(&mut child, dom_tree, stack, headers);
    }

    stack.decrease_layer();

}

pub fn to_ssa(nodes: &mut Vec<Rc<Node>>, headers: &[FnHeaders]) {

    let (dom_tree, mut def_map) = insert_phi_nodes(&mut nodes[..]);
    for header in headers {
        def_map.entry(header.name).or_insert((HashSet::new(), header.r_type.clone()));
    }
    let mut stack = RenameStack::new(def_map.keys());
    let header_vars: Vec<Var> = headers.iter().map(|x|x.name).collect();

    rename(&mut nodes[0], &dom_tree, &mut stack, &header_vars[..])
}
