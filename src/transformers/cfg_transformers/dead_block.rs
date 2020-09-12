use super::super::cfg::{Node};
use crate::serde_structs::structs::CFGFunction;
use std::rc::Rc;

pub fn remove_inaccessible_blocks(blocks: Vec<Node>) -> Vec<Node> {
    let _root = blocks[0].reference();
    (blocks).into_iter().filter(|Node(x)| {Rc::weak_count(x) > 0}).collect()
}
