use crate::transformers::cfg::{Node, Block, Link};
use std::rc::Rc;

pub struct AnalysisNode<D> {
    pub in_data: D,
    pub out_data: D,
    pub program_node: Rc<Node>,
    predecessors: Vec<usize>,
    successors: Vec<usize>
}

pub enum Direction {
    Forward,
    Backward
}

pub fn worklist_solver<D, T, M>(nodes: &[Rc<Node>], initial_value: D, transfer_fn: T,
                            merge_fn: M, direction: Direction) -> Vec<AnalysisNode<D>>
    where T:Fn(&D, &Block, usize) -> D, M:Fn(Vec<&D>) -> D, D: Clone + PartialEq {
        let mut analysis_nodes = Vec::<AnalysisNode<D>>::new();

        for (idx, node) in nodes.iter().enumerate() {
            node.idx.replace(Some(idx));
        }

        for (idx, node) in nodes.iter().enumerate() {
            analysis_nodes.push(AnalysisNode {
                in_data: initial_value.clone(),
                out_data: initial_value.clone(),
                program_node: Rc::clone(node),
                predecessors: Vec::new(),
                successors: Vec::new()
            });
            let pred_ref = node.predecessors.borrow();
            let preds: Vec<usize> = pred_ref.iter().map(|x| -> usize {
                let upgraded = x.upgrade().unwrap();
                let num = upgraded.idx.borrow();
                num.unwrap()
            }).collect();

            analysis_nodes[idx].predecessors = preds;

            let out = node.out.borrow();
            if let Some(x) = out.as_ref() {
                let successors: Vec<usize> = match x{
                    Link::Ret => vec! [],
                    Link::Exit => vec! [],
                    Link::Fallthrough(x) | Link::Jump(x)  => {
                        let upgraded = x.upgrade().unwrap();
                        let borrowed: &Option<usize> = &upgraded.idx.borrow();
                        vec! [borrowed.unwrap()]
                    }
                    Link::Branch { true_branch, false_branch } => {
                        let upgraded_true = true_branch.upgrade().unwrap();
                        let borrowed_true: &Option<usize> = &upgraded_true.idx.borrow();
                        let upgraded_false = false_branch.upgrade().unwrap();
                        let borrowed_false: &Option<usize> = &upgraded_false.idx.borrow();
                        vec! [borrowed_true.unwrap(), borrowed_false.unwrap()]
                    }
                };
                analysis_nodes[idx].successors = successors;
            }
        }

        // Now the analysis nodes are fully set up and we no longer need to refer
        // to the program blocks for the graph

        let mut worklist: Vec<usize> = (0..analysis_nodes.len()).collect();

        while let Some(block_idx) = worklist.pop() {
            let old: D = if let Direction::Forward = direction {
                std::mem::replace(&mut analysis_nodes[block_idx].out_data, initial_value.clone())
            } else {
                std::mem::replace(&mut analysis_nodes[block_idx].in_data, initial_value.clone())
            };

            let merge_list: Vec<&D> = if let Direction::Forward = direction {
                analysis_nodes[block_idx].predecessors.iter().map(|x| -> &D {
                    &analysis_nodes[*x].out_data
                }).collect()
            } else {
                analysis_nodes[block_idx].successors.iter().map(|x| -> &D {
                    &analysis_nodes[*x].in_data
                }).collect()
            };

            if let Direction::Forward = direction {
                analysis_nodes[block_idx].in_data = merge_fn(merge_list)
            } else {
                analysis_nodes[block_idx].out_data = merge_fn(merge_list)
            }

            {
            // going directly to the
            let block:&Block = &nodes[block_idx].contents.borrow();
            if let Direction::Forward = direction {
                analysis_nodes[block_idx].out_data = transfer_fn(&analysis_nodes[block_idx].in_data, block, block_idx);
            } else {
                analysis_nodes[block_idx].in_data = transfer_fn(&analysis_nodes[block_idx].out_data, block, block_idx);
            }
            }

            let updates_required = if let Direction::Forward = direction {
                analysis_nodes[block_idx].out_data != old
            } else {
                analysis_nodes[block_idx].in_data != old
            };

            if updates_required {
                if let Direction::Forward = direction {
                    for index in analysis_nodes[block_idx].successors.iter() {
                        worklist.push(*index)
                    }
                } else {
                    for index in analysis_nodes[block_idx].predecessors.iter() {
                        worklist.push(*index)
                    }
                }
            }
        }

        // invalidate the index info
        for node in nodes.iter() {
            node.idx.replace(None);
        }

        analysis_nodes
    }
