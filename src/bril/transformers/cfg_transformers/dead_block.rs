use super::super::cfg::{Node};
use crate::bril::serde_structs::*;
use std::rc::Rc;

pub fn remove_inaccessible_block(blocks: Vec<Node>) -> Vec<Node> {
    let _root = blocks[0].reference();
    blocks.into_iter().filter(|Node(x)| {Rc::weak_count(x) > 0}).collect()
}

pub fn remove_inaccessible_blocks(mut input: CFGFunction) -> CFGFunction {
    input.instrs = remove_inaccessible_block(input.instrs);
    input

}
