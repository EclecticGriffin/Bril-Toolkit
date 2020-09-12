use super::cfg::{Node};
use std::rc::Rc;

pub fn remove_inaccessible_blocks(blocks: Vec<Node>) -> Vec<Node> {
    let _root = blocks[0].reference();
    (blocks).into_iter().filter(|Node(x)| {Rc::weak_count(x) > 0}).collect()
}
