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


pub fn compute_dominance_tree(nodes: &mut [Rc<Node>]) -> HashMap<Label, HashSet<Label>> {
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
