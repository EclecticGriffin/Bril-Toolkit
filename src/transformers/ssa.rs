use super::cfg::{Block, Node, Link};
use super::dominance::DominanceTree;
use crate::serde_structs::namer;
use crate::serde_structs::structs::{Instr, Label, Op, Type, Var, FnHeaders};
use std::collections::{HashMap, HashSet, VecDeque};
use std::ops::{Index, IndexMut};
use std::rc::{Rc, Weak};

fn identify_definitions(nodes: &[Rc<Node>], headers: &[FnHeaders]) -> HashMap<Var, (HashSet<Label>, Type)> {
    let mut var_map: HashMap<Var, (HashSet<Label>, Type)> = HashMap::new();
    {
    let contents: &mut Block = &mut nodes[0].contents.borrow_mut();

    for header in headers {
        let mut set = HashSet::<Label>::with_capacity(1);
        set.insert(nodes[0].label());
        var_map.insert(header.name, (set, header.r_type.clone()));
        contents.0.insert(1, Instr::Value {
            op: Op::Id,
            dest: header.name,
            r_type: header.r_type.clone(),
            args: vec! [header.name],
            funcs: Vec::new(),
            labels: Vec::new(),

        })
    }
}

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
    nodes: &mut [Rc<Node>], headers: &[FnHeaders]
) -> (DominanceTree, HashMap<Var, (HashSet<Label>, Type)>) {

    let mut def_map = identify_definitions(nodes, headers);
    let dom_tree = DominanceTree::new(nodes);
    // eprintln!("dom tree computed");

    for (var, (defs, r_type)) in def_map.iter_mut() {
        let mut queue: VecDeque<Label> = defs.iter().cloned().collect();
        let def_len = queue.len();

        if queue.len() != 1 {
            // eprintln!(">>>> checking defs of {} total {}", var, def_len);
            while let Some(block_label) = queue.pop_front() {
                // eprintln!("queue length {}", queue.len());
                // eprintln!("frontier for {} is {:?} ", block_label, dom_tree.compute_frontier(&block_label));
                for block in dom_tree.compute_frontier(&block_label) {
                    // eprintln!("frontier for {} contains {}", block_label, block);

                    let node = dom_tree.lookup_node(&block);
                    let contents: &mut Block = &mut node.contents.borrow_mut();
                    if contents.len() != 1 {
                        let mut found = false;
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
                                    found = true;
                                    break
                                }
                            }
                        }

                        if found {
                            continue;
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
                    // eprintln!("Inserting phi node for {} in {}", var, node.label());
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
    fn new(vars: std::collections::hash_map::Keys<Var, (HashSet<Label>, Type)>, headers: &[FnHeaders]) -> Self {
        let mut stack_map = HashMap::<Var, Vec<Var>>::with_capacity(vars.len());
        for var in vars {
            // eprintln!("[[[[[[[[[[[INSERTING {}", var);
            stack_map.insert(*var, Vec::new());
        }

        for header in headers {
            stack_map.get_mut(&header.name).unwrap().push(header.name);
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

    fn contains(&self, target: &Var) -> bool {
        self.var_stacks.contains_key(target)
    }

    fn push_var(&mut self, old_name: &Var, new_name: Var) {
        // eprintln!("Old name: {}", old_name);
        self.var_stacks.get_mut(old_name).unwrap().push(new_name)
    }

    fn get_top(&self, old_name: &Var) -> Option<Var> {
        // eprintln!("Old name: {}", old_name);
        match self.var_stacks.get(old_name).unwrap().last() {
            Some(v) => Some(*v),
            None => {
                // eprintln!("There's no rename for {}", old_name);
                None},
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
                    stack.push_var(dest, new_name);
                    *dest = new_name;

                }
                // Phi instrs we only update the new name, not the args
                Instr::Value {
                    op: Op::Phi, dest, ..
                } => {
                    let new_name = Var(namer().fresh(&dest.0));
                    stack.push_var(dest, new_name);
                    *dest = new_name;
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
                    stack.push_var(dest, new_name);
                    *dest = new_name;
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
        // eprintln!("updating successors");
        let contents: &mut Block = &mut successor.contents.borrow_mut();
        let block = &mut contents.0;
        for instr in block.iter_mut() {
            if let Instr::Value { op: op @ Op::Phi, args, dest, labels, ..} = instr {
                let mut index:usize = 0;
                let mut found = false;
                for (idx, arg) in args.iter().enumerate() {
                    if stack.contains(arg) {
                        index = idx;
                        found = true;
                        break
                    }
                }
                if !found {
                    // eprintln!("adding arg to phi node for {}", dest);
                    args.push(stack.get_top(dest).unwrap());
                    labels.push(node.label())
                    // panic!("No arg to rename? {:?} {} {}", args, dest, successor.label())
                } else {
                    // eprintln!("rewriting phi node");
                    // eprintln!("args len {}, idx {}", args.len(), index);
                    let var = args.get_mut(index).unwrap();

                    if let Some(renamed) = stack.get_top(var){
                        *var = renamed;
                        let label = labels.get_mut(index).unwrap();
                        *label = node.label();
                    } else {
                        // The phi node is not valid along this path. Remove
                        *op = Op::Nop;
                        *args = vec! [];
                        *labels = vec! [];
                    }


                }


            }
        }
    }

    for mut child in dom_tree.get_children(&node.label()) {
        rename(&mut child, dom_tree, stack, headers);
    }

    let contents: &mut Block = &mut node.contents.borrow_mut();
    let block = &mut contents.0;
    // TODO: Figure out how to get rid of this
    for instr in block.iter_mut() {
        if let Instr::Value {op: op @ Op::Phi, args, labels, ..} = instr{
            // eprintln!("{:?}", args);
            while !args.is_empty() && stack.contains(args.last().unwrap()) {
                args.pop();
                labels.pop();
            }
            if args.is_empty() {
                *op = Op::Nop;
            }
        }
    }

    block.retain(|x| if let Instr::Value { op:Op::Nop, ..} = x {
        false
    } else{
        true
    });

    stack.decrease_layer();

}

pub fn to_ssa(nodes: &mut Vec<Rc<Node>>, headers: &[FnHeaders]) {
    for node in nodes.iter() {
        node.normalize()
    }
    let (dom_tree, mut def_map) = insert_phi_nodes(&mut nodes[..], headers);

    let mut stack = RenameStack::new(def_map.keys(), headers);
    let header_vars: Vec<Var> = headers.iter().map(|x|x.name).collect();

    rename(&mut nodes[0], &dom_tree, &mut stack, &header_vars[..])
}

pub fn from_ssa(nodes: &mut Vec<Rc<Node>>) {
    let mut label_map: HashMap<Label, Rc<Node>> = HashMap::new();

    for node in nodes.iter() {
        node.clear_predecessors();
        label_map.insert(node.label(), node.clone());
    }

    let mut new_nodes: Vec<Rc<Node>> = Vec::new();
    let mut fix_list: Vec<(Label, Rc<Node>)> = Vec::new();

    let mut new_nodes_map: HashMap<(Label, Label), Rc<Node>> = HashMap::new();

    for node in nodes.iter_mut() {
        let block = &mut *node.contents.borrow_mut();
        // let mut correction_list: Vec<> = vec! [];

        for instr in block.0.iter_mut() {
            if let Instr::Value {op: Op::Phi, args, labels, dest, r_type,..} = instr {
                for (var, label) in args.iter().zip(labels.iter()) {

                    let new_node = new_nodes_map.entry((node.label(), *label)).or_insert(Node::empty_block());
                    new_node.insert_id(*var, *dest, r_type.clone());
                    // if *label != node.label() {
                    //     label_map.get_mut(label).unwrap().replace_link(node.label(), Rc::downgrade(&new_node), new_node.label());
                    //     new_nodes.push(new_node);
                    // } else {
                    //     fix_list.push((*label, new_node));
                    // }
                }
            }
        }
    }

    for ((to, from), new_node) in new_nodes_map.drain() {
        {
            let target = &label_map[&to];
            new_node.add_jump(Rc::downgrade(target), target.label());
        }
        label_map[&from].replace_link(to,
                            Rc::downgrade(&new_node),
                                    new_node.label());
        nodes.push(new_node);
    }

    // for (label, new_node) in fix_list {
    //     // eprintln!("{} {}", label, new_node.label());
    //     label_map.get_mut(&label).unwrap().replace_link(label, Rc::downgrade(&new_node), new_node.label());
    //     new_nodes.push(new_node);
    // }

    nodes.append(&mut new_nodes);

    let len = nodes.len();

    for (idx,node) in nodes.iter_mut().enumerate() {
        node.contents.borrow_mut().0.retain(|x| {
            if let Instr::Value {op:Op::Phi, ..} = x {
                false
            } else {
                true
            }
        });

        if let Some(Link::Exit) = &*node.out.borrow() {
            if idx != len-1{
                node.contents.borrow_mut().0.push(Instr::Effect {
                    op: Op::Ret,
                    args: Vec::new(),
                    funcs: Vec::new(),
                    labels: Vec::new(),
                });
            }

        }

    }

}
