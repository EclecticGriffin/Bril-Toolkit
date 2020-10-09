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

    while let Some(current) = process_queue.last() {
        // eprintln!("process_queue {:?}", process_queue.len());
        let successors = current.successor_refs();

        if successors.is_empty() {
            // eprintln!("no successors");
            exit_queue.push(process_queue.pop().unwrap());
        } else {
            // eprintln!("successors!");
            let not_processed: Vec<&Rc<Node>> = successors.iter().filter(|x| !exit_queue.contains(x) && *x != current).collect();
            if not_processed.is_empty() {
                // eprintln!("all successors processed!");
                exit_queue.push(process_queue.pop().unwrap());
            } else if not_processed.iter().all(|x| process_queue.contains(x)) {
                    exit_queue.push(process_queue.pop().unwrap());
                } else {
                    for item in not_processed.into_iter() {
                    if !process_queue.contains(item) {
                        // eprintln!("pushing {}", item.label());
                        process_queue.push(item.clone());
                    }
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
    // assert!(ordering.len() == nodes.len(), "ordering: {}, nodes: {}", ordering.len(), nodes.len());
    {
        let mut dom_set = HashSet::<Label>::new();
        for node in ordering.iter() {
            dom_set.insert(node.label());
        }
        for node in ordering.iter(){
            // eprintln!("inserted {}", node.label());
            label_map.insert(node.label(), dom_set.clone());
        }
    }

    let mut changed = true;

    while changed {
        changed = false;
        for node in ordering.iter() {
            let preds = node.predecessor_labels();
            let sets: Vec<&HashSet<Label>> = preds.into_iter().map(|x| {&label_map[&x]}).collect();

            let intersect = set_intersection(sets);
            let mut current = HashSet::<Label>::with_capacity(1);
            current.insert(node.label());
            let new_value = set_union(vec! [&intersect, &current ]);

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
    dominated_map: HashMap<Label, HashSet<Label>>,
    dom_tree: HashMap<Label, Vec<Label>>,
    root_label: Label
}

impl DominanceTree {
    pub fn new(nodes: &[Rc<Node>]) -> Self {
        let mut node_map = HashMap::<Label, Rc<Node>>::new();
        for node in nodes {
            node_map.insert(node.label(), node.clone());
        }

        let (dom_tree, dominated_map) = construct_dominance_tree(nodes);

        DominanceTree {
            nodes: node_map,
            dom_tree,
            dominated_map,
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
        let mut processing_queue: Vec<Label> = self.lookup_node(target_label).successor_labels();
        let mut frontier = Vec::<Label>::new();
        let mut processed: HashSet<Label> = HashSet::new();
        processed.insert(*target_label);

        while let Some(current) = processing_queue.pop() {
            if self.dominated_map[&current].contains(target_label) {
                processed.insert(current);
                for successor_label in self.lookup_node(&current).successor_labels() {
                    if !processed.contains(&successor_label) {
                        processing_queue.push(successor_label);
                    } else if successor_label == *target_label && !frontier.contains(target_label) {
                        frontier.push(successor_label);
                    }
                }
            } else {
                frontier.push(current);
            }
    }
    // frontier.push(*target_label);

    frontier
    }
}

fn construct_dominance_tree(nodes: &[Rc<Node>]) -> (HashMap<Label, Vec<Label>>, HashMap<Label, HashSet<Label>>){
    let dominance_map = determine_dominators(nodes);
    let mut label_map = HashMap::<Label, Rc<Node>>::new();
    let mut immediate_dominance_map = HashMap::<Label, Vec<Label>>::new();

    for node in nodes {
        label_map.insert(node.label(), node.clone());
        immediate_dominance_map.insert(node.label(), Vec::new());
    }


    for node in nodes {
        for successor_label in node.successor_refs().iter().map(|x| x.label()) {
            if dominance_map[&successor_label].contains(&node.label()) && node.label() != successor_label {
                immediate_dominance_map.get_mut(&node.label()).unwrap().push(successor_label);
            }
        }
    }

//     for (label, doms) in dominance_map.iter() {
//         eprint!("{} is dominated by [", label);
//         for label in doms.iter() {
//             eprint!(" {}", label)
//         }
//         eprintln!("]");
//    }

    // for (label, doms) in immediate_dominance_map.iter() {
    //      eprintln!("{} dominates {:?}", label, doms);
    // }

    (immediate_dominance_map, dominance_map)
}
