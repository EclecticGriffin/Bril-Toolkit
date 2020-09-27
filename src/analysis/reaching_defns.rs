use super::prelude::*;
use super::dehydrated::set_union;
use std::collections::HashSet;
use std::cell::RefCell;
use super::super::transformers::cfg::Link;

type Data = HashSet<VarDef>;

#[derive(Hash, Clone, Eq, PartialEq, Debug)]
pub struct VarDef(Var, usize);

impl Display for VarDef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}_{}", self.0, self.1)
    }
}



fn transfer(input: &Data, instrs: &Block, idx: usize) -> Data {
    let mut out = Data::new();

    for instr in instrs.0.iter() {
        match instr {
            Instr::Const { dest, .. } | Instr::Value { dest, .. } => {
                let new = VarDef(*dest, idx);
                out.retain(|x| {x.0 != *dest});
                out.insert(new);
            }
            _ => {}
        }
    }
    input.union(&out).cloned().collect()
}

pub fn reaching_definitions(nodes: &[Rc<Node>], initial: &[FnHeaders] ) -> Vec<AnalysisNode<Data>> {


    let fake_node = Node {
        contents: RefCell::new(Block(initial.iter().map(|x| x.generate_dummy_instrs()).collect())),
        out: RefCell::new(Some(Link::Fallthrough(Rc::downgrade(&nodes[0])))),
        predecessors: RefCell::new(Vec::new()),
        idx: RefCell::new(None),
    };

    let mut input_nodes = Vec::<Rc<Node>>::new();

    input_nodes.push(Rc::new(fake_node));
    for node in nodes {
        input_nodes.push(node.clone())
    }


    worklist_solver(&input_nodes, Data::new(), transfer, set_union, Direction::Forward)
}
