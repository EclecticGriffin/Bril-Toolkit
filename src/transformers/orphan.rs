use super::cfg::{Node};
use std::rc::Rc;

pub fn remove_inaccessible_blocks(blocks: Vec<Rc<Node>>) -> Vec<Rc<Node>> {
    let _root = Rc::downgrade(&blocks[0]);
    (blocks).into_iter().filter(|x| {Rc::weak_count(x) > x.successor_count()}).collect()
}
