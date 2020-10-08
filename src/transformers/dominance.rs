use super::cfg::{Node, Block};
use crate::serde_structs::structs::{Instr, Label};
use std::rc::{Rc, Weak};
use std::collections::{HashMap, HashSet};
use crate::analysis::dehydrated::{set_intersection, set_union};

use std::collections::VecDeque;



fn reverse_post_order(root: &Rc<Node>) -> Vec<Rc<Node>> {
    let mut process_queue = Vec::<Rc<Node>>::new();
    let mut exit_queue = Vec::<Rc<Node>>::new();

    process_queue.push(root.clone());

    while let Some(x) = process_queue.last() {
        let successors = x.successor_refs();

        if successors.len() == 0 {
            exit_queue.push(x.clone());
        } else {
            let not_processed: Vec<&Rc<Node>> = successors.iter().filter(|x| exit_queue.contains(x)).collect();
            if not_processed.len() == 0 {
                exit_queue.push(x.clone());
            } else {
                for item in not_processed.into_iter() {
                    process_queue.push(item.clone());
                }
            }
        }
    }

    exit_queue.reverse();
    exit_queue
}


pub fn determine_dominators(nodes: &[Rc<Node>]) -> HashMap<Label, HashSet<Label>> {
    let mut label_map = HashMap::<Label, HashSet<Label>>::new();
    let ordering = reverse_post_order(&nodes[0]);

    {
        let mut dom_set = HashSet::<Label>::new();
        for node in ordering.iter() {
            dom_set.insert(node.label());
        }
        for node in ordering.iter(){
            label_map.insert(node.label(), dom_set.clone());
        }
    }

    let mut changed = true;

    while changed {
        changed = false;
        for node in ordering.iter() {
            let preds = node.predecessor_labels();
            let sets: Vec<&HashSet<Label>> = preds.into_iter().map(|x| &label_map[&x]).collect();
            let intersect = set_intersection(sets);
            let new_value = set_union(vec! [&intersect, &label_map[&node.label()] ]);

            if new_value != label_map[&node.label()] {
                changed = true;
                label_map.insert(node.label(), new_value);
            }
        }

    }
    label_map
}

pub struct DominanceTree {
    nodes: HashMap<Label, Rc<Node>>,
    dom_tree: HashMap<Label, Vec<Label>>,
    root_label: Label
}

impl DominanceTree {
    pub fn new(nodes: &[Rc<Node>]) -> Self {
        let mut node_map = HashMap::<Label, Rc<Node>>::new();
        for node in nodes {
            node_map.insert(node.label(), node.clone());
        }

        DominanceTree {
            nodes: node_map,
            dom_tree: construct_dominance_tree(nodes),
            root_label: nodes[0].label()
        }
    }

    pub fn lookup_node(&self, label: &Label) -> &Rc<Node> {
        &self.nodes[label]
    }

    pub fn root_node(&self) -> &Rc<Node> {
        self.lookup_node(&self.root_label)
    }

    pub fn get_children(&self, label: &Label) -> Vec<Rc<Node>> {
        self.dom_tree[label].iter()
            .map(|x| {self.lookup_node(x)})
            .cloned()
            .collect()
    }

    pub fn compute_frontier(&self, target_label: &Label) -> Vec<Label> {
        let mut processing_queue: Vec<(Label, Label)> = self.lookup_node(target_label).successor_labels().into_iter().map(|x| (*target_label, x)).collect();
        let mut frontier = Vec::<Label>::new();

        while let Some((previous, current)) = processing_queue.pop() {
            if self.dom_tree[&previous].contains(&current) {
                for successor_label in self.lookup_node(&current).successor_labels() {
                    processing_queue.push((current, successor_label));
                }
            } else {
                frontier.push(current);
            }
    }

    frontier
    }
}

fn construct_dominance_tree(nodes: &[Rc<Node>]) -> HashMap<Label, Vec<Label>>{
    let dominance_map = determine_dominators(nodes);
    let mut label_map = HashMap::<Label, Rc<Node>>::new();

    for node in nodes {
        label_map.insert(node.label(), node.clone());
    }
    let mut immediate_dominance_map = HashMap::<Label, Vec<Label>>::new();

    for node in nodes {
        for successor_label in node.successor_refs().iter().map(|x| x.label()) {
            if dominance_map[&successor_label].contains(&node.label()) && node.label() != successor_label {
                immediate_dominance_map.get_mut(&node.label()).unwrap().push(successor_label);
            }
        }
    }

    immediate_dominance_map
}
