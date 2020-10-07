use super::super::serde_structs::structs::{Label, Instr, Op};
use super::super::serde_structs::namer;
use std::collections::HashMap;
use std::rc::{Rc, Weak};
use std::{cell::RefCell, fmt, fmt::Display};
use std::iter::Iterator;



type LinkTarget = Weak<Node>;
type LabelMap = HashMap<Label, Rc<Node>>;

#[derive(Debug)]
pub struct Block(pub Vec<Instr>);

impl Block {
    pub fn new(input: Vec<Instr>) -> Self {
        Block(input)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn label(&self) -> Option<Label> {
        self.0[0].extract_label()
    }

    pub fn last(&self) -> &Instr {
        self.0.last().unwrap()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    fn make_serializeable(self) -> Vec<Instr> {
        self.0
    }

    fn iter(&self) -> std::slice::Iter<Instr> {
        self.0.iter()
    }

    fn iter_mut(&mut self) -> std::slice::IterMut<Instr> {
        self.0.iter_mut()
    }

}

impl Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.label() {
            Some(l) => {
                write!(f, "{}", l)
            }
            None => {
                write!(f, "unlabeled")
            }
        }
    }
}

impl IntoIterator for Block {
    type Item = Instr;

    type IntoIter = std::vec::IntoIter<Instr>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[derive(Debug)]
pub enum Link {
    Ret,
    Exit,
    Fallthrough(LinkTarget),
    Jump(LinkTarget),
    Branch {
        true_branch: LinkTarget,
        false_branch: LinkTarget,
    },
}

#[derive(Debug)]
pub struct Node {
    pub contents: RefCell<Block>,
    pub out: RefCell<Option<Link>>,
    pub predecessors: RefCell<Vec<Weak<Node>>>,
    pub idx: RefCell<Option<usize>>, // this is bad and I am bad
    label: Label
}

impl Node {
    pub fn from_block(input: Block) -> Node {
        if input.is_empty() {
            panic!("Tried to create an empty block????\n")
        }

        let label = input.label().unwrap_or(namer().fresh_label());
            Node {
                contents: RefCell::new(input),
                out: RefCell::new(None),
                predecessors: RefCell::new(Vec::new()),
                idx: RefCell::new(None),
                label
            }
    }

    pub fn new(input: Vec<Instr>) -> Node{
        Node::from_block(Block::new(input))
    }

    pub fn dummy_block(
        contents: RefCell<Block>,
        out: RefCell<Option<Link>>,
        predecessors: RefCell<Vec<Weak<Node>>>,
        idx: RefCell<Option<usize>>) -> Self {

        Node {
            contents,
            out,
            predecessors,
            idx,
            label: namer().fresh_label()
        }
    }

    pub fn label(&self) -> Label{
        self.label
    }

    pub fn predecessor_labels(&self) -> Vec<Label> {
        self.predecessors.borrow().iter().map(|x| Weak::upgrade(x).unwrap().label()).collect()
    }

    pub fn block_label(&self) -> Option<Label>{
        self.contents.borrow().label()
    }

    pub fn is_labeled(&self) -> bool {
        self.contents.borrow().label().is_some()
    }

    pub fn make_serializeable(self) -> Vec<Instr> {
        self.contents.into_inner().make_serializeable()
    }

    pub fn successor_count(&self) -> usize {
        let out: &Option<Link> = &self.out.borrow();
        match out {
            Some(x) => {
                match x {
                    Link::Branch {..} => 2,
                    Link::Exit => 0,
                    Link::Jump {..} => 1,
                    Link::Fallthrough {..} => 1,
                    Link::Ret => 0,
                }
            }
            None => 0
        }
    }

    pub fn successor_refs(&self) -> Vec<Rc<Node>> {
        let successor: &Option<Link> = &self.out.borrow();

        match successor {
            Some(link) => {
                match link {
                    Link::Ret => Vec::new(),
                    Link::Exit => Vec::new(),
                    Link::Fallthrough(weak) => Weak::upgrade(weak).map_or(Vec::new(), |s| { vec! [s] }),
                    Link::Jump(weak) => Weak::upgrade(weak).map_or(Vec::new(), |s| { vec! [s] }),
                    Link::Branch { true_branch, false_branch } => {
                        let mut true_br = Weak::upgrade(true_branch).map_or(Vec::new(), |s| { vec! [s] });
                        let mut false_br  = Weak::upgrade(false_branch).map_or(Vec::new(), |s| { vec! [s] });
                        true_br.append(&mut false_br);
                        true_br
                    }
                }
            }
            None => Vec::new()
        }
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.label() == other.label()
    }
}


pub fn construct_cfg_nodes(instrs: Vec<Block>) -> Vec<Rc<Node>> {
    instrs.into_iter().map(|x|{Rc::new(
        Node::from_block(x)
    )}).collect()
}

pub fn construct_basic_blocks(instrs: Vec<Instr>) -> Vec<Block> {
    let mut output = Vec::<Block>::new();
    let mut cur_block = Vec::<Instr>::new();
    for instr in instrs.into_iter() {
        if instr.is_label() {
            if !cur_block.is_empty() {
                output.push(Block::new(cur_block));
            }
            cur_block = vec![instr];
        } else if instr.is_terminator() {
            cur_block.push(instr);
            output.push(Block::new(cur_block));
            cur_block = Vec::<Instr>::new();
        } else {
            cur_block.push(instr);
        }
    }
    if !cur_block.is_empty() {
        output.push(Block::new(cur_block));
    }
    output
}

fn construct_label_lookup(blocks: &[Rc<Node>]) -> LabelMap {
    let mut map = LabelMap::new();
    for node in blocks.iter() {
        if node.is_labeled() {
            map.insert(node.block_label().unwrap(), Rc::clone(node));
        }
    }
    map
}

fn connect_block(current: &Rc<Node>, node: &Rc<Node>, map: &LabelMap) {
    if current.out.borrow().is_some() {
        return;
    }

    let instrs = &current.contents.borrow();
    let last = instrs.last();
    let (op, labels) = match last {
        Instr::Value { op, labels, .. } | Instr::Effect { op, labels, .. } => (op, labels),
        _ => {
            current
                .out
                .replace(Some(Link::Fallthrough(Rc::downgrade(node))));
            let mut vec_cell = node.predecessors.borrow_mut();
            vec_cell.push(Rc::downgrade(current));

            return;
        }
    };
    match op {
        Op::Jmp => {
            let target = &labels[0];
            let target_ref = map.get(&target).unwrap_or_else(|| {
                panic!(
                    "Unable to locate label {}",
                    namer().get_string(&(target.0))
                )
            });
            current.out.replace(Some(Link::Jump(Rc::downgrade(target_ref))));
            let mut vec_cell = target_ref.predecessors.borrow_mut();
            vec_cell.push(Rc::downgrade(current));
        }
        Op::Br => {
            let true_label = &labels[0];
            let false_label = &labels[1];

            let true_target = map.get(&true_label).unwrap_or_else(|| {
                panic!(
                    "Unable to locate label {}",
                    namer().get_string(&(true_label.0))
                )
            });

            let false_target = map.get(&false_label).unwrap_or_else(|| {
                panic!(
                    "Unable to locate label {}",
                    namer().get_string(&(false_label.0))
                )
            });

            current.out.replace(Some(Link::Branch {
                true_branch: Rc::downgrade(true_target),
                false_branch: Rc::downgrade(false_target),
            }));
            let mut vec_cell = true_target.predecessors.borrow_mut();
            vec_cell.push(Rc::downgrade(current));

            let mut vec_cell = false_target.predecessors.borrow_mut();
            vec_cell.push(Rc::downgrade(current));
        }
        Op::Ret => {
            current.out.replace(Some(Link::Ret));
        }
        _ => {
            current
                .out
                .replace(Some(Link::Fallthrough(Rc::downgrade(node))));
            let mut vec_cell = node.predecessors.borrow_mut();
            vec_cell.push(Rc::downgrade(current));
        }
    }
}

fn connect_terminal_block( last_block: &Rc<Node>, map: &LabelMap) {
    let instrs = &last_block.contents.borrow();
    let last = instrs.last();

    match last {
        Instr::Value { op, labels, .. } | Instr::Effect { op, labels, .. } => match op {
            Op::Jmp => {
                let target = &labels[0];
                let target_ref = map.get(&target).unwrap_or_else(|| {
                    panic!(
                        "Unable to locate label {}",
                        namer().get_string(&(target.0))
                    )
                });
                last_block
                    .out
                    .replace(Some(Link::Jump(Rc::downgrade(target_ref))));
                    let mut vec_cell = target_ref.predecessors.borrow_mut();
                    vec_cell.push(Rc::downgrade(last_block));
                return;
            }
            Op::Br => {
                let true_label = &labels[0];
                let false_label = &labels[1];

                let true_target = map.get(&true_label).unwrap_or_else(|| {
                    panic!(
                        "Unable to locate label {}",
                        namer().get_string(&(true_label.0))
                    )
                });

                let false_target = map.get(&false_label).unwrap_or_else(|| {
                    panic!(
                        "Unable to locate label {}",
                        namer().get_string(&(false_label.0))
                    )
                });

                last_block.out.replace(Some(Link::Branch {
                    true_branch: Rc::downgrade(true_target),
                    false_branch: Rc::downgrade(false_target),
                }));
                let mut vec_cell = true_target.predecessors.borrow_mut();
                vec_cell.push(Rc::downgrade(last_block));

                let mut vec_cell = false_target.predecessors.borrow_mut();
                vec_cell.push(Rc::downgrade(last_block));
                return;
            }
            Op::Ret => {
                last_block.out.replace(Some(Link::Ret));
                return;
            }
            _ => {}
        },
        _ => {}
    };

    last_block.out.replace(Some(Link::Exit));
}

pub fn connect_basic_blocks(blocks: &mut Vec<Rc<Node>>) {
    let map = construct_label_lookup(blocks);
    let mut second_iter = blocks.iter();
    second_iter.next();

    for (current, node) in blocks.iter().zip(second_iter) {
        connect_block(current, node, &map)
    }

    connect_terminal_block(blocks.last().unwrap(), &map)
}



impl Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(label) = &self.block_label() {
            writeln!(f, "Block {}:", namer().get_string(&label.0))?;
        } else {
            writeln!(f, "Block (unlabeled):")?;
        }
        for line in self.contents.borrow_mut().iter() {
            writeln!(f,"     {}", line)?;
        }
        if let Some(x) = self.out.borrow().as_ref() {
            write!(f, " Connected to: {}", x)
        } else {
            write!(f, "  not connected?")
        }
    }
}



impl Display for Link {
    fn fmt(&self, f: & mut fmt::Formatter<'_>) -> fmt::Result {
        let namer = namer();
        match &self {
            Link::Ret => {
                write!(f, "<RETURN>")
            }
            Link::Exit => {
                write!(f, "<EXIT>")
            }
            Link::Fallthrough(val) => {
                let val = val.upgrade();
                if let Some(val) = val {
                    match val.block_label() {
                        Some(label) => {
                            write!(f, "<FALLTHROUGH: .{}>", namer.get_string(&label.0))
                        }
                        None => {
                            // Is this possible???
                            write!(f, "<FALLTHROUGH: Unlabeled Block>")
                        }
                    }
                } else {
                    write!(f, "?? LOST CONNECTION ??")
                }
            }
            Link::Jump(val) => {
                let val = val.upgrade();
                if let Some(val) = val {
                    if let Some(label) = val.block_label() {
                        write!(f, "<JUMP: .{}>", namer.get_string(&label.0))?
                    }
                    Ok(())
                } else {
                    write!(f, "?? LOST CONNECTION ??")
                }
            }
            Link::Branch { true_branch, false_branch } => {
                let val = true_branch.upgrade();
                if let Some(val) = val {
                    if let Some(label) = val.block_label() {
                        write!(f, "<BR TRUE: .{}>", namer.get_string(&label.0))?;
                    }
                } else {
                    write!(f, "<BR TRUE:?? LOST CONNECTION ??>")?;
                }

                write!(f, " ")?;
                let val = false_branch.upgrade();
                if let Some(val) = val {
                    if let Some(label) = val.block_label(){
                        write!(f, "<BR FALSE: .{}>", namer.get_string(&label.0))
                    } else {
                        Ok(())
                    }
                } else {
                    write!(f, "<BR FALSE:?? LOST CONNECTION ??>")
                }

            }
        }
    }
}
