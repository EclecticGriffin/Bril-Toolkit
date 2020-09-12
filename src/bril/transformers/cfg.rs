use super::super::serde_structs::*;
use crate::utils::name_mapper;
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::{Rc, Weak};
use std::{cell::RefCell, fmt, fmt::Display};
// fn gen_cfg(prog: Program) -> PLACEHOLDER {

type Block = Vec<Instr>;
type LinkTarget = Weak<CFGNode>;
type LabelMap = HashMap<Label, Node>;
#[derive(Debug)]
enum Link {
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
pub struct CFGNode {
    contents: Block,
    label: Option<Label>,
    out: RefCell<Option<Link>>,
}

#[derive(Debug)]
pub struct Node(pub Rc<CFGNode>);

impl Node {
    pub fn reference(&self) -> Weak<CFGNode> {
        Rc::downgrade(&self.0)
    }

    pub fn new(mut input: Vec<Instr>) -> Self {
        let inner = CFGNode::new(input);
        Node(Rc::new(inner))
    }
    pub fn clone(&self) -> Node {
        Node(Rc::clone(&self.0))
    }
}

// impl Deref for Node {
//     type Target = Rc<CFGNode>;

//     fn deref(&self) -> &Self::Target {

//     }
// }

impl CFGNode {
    pub fn new(mut input: Vec<Instr>) -> CFGNode {
        if input.is_empty() {
            panic!("Tried to create an empty block????\n")
        }

        if input[0].is_label() {

            let label = input.remove(0).extract_label().unwrap();

            CFGNode {
                label: Some(label),
                contents: input,
                out: RefCell::new(None),
            }
        } else {
            CFGNode {
                label: None,
                contents: input,
                out: RefCell::new(None),
            }
        }
    }

    pub fn apply_label(&mut self, label: Label) {
        if self.label.is_none() {
            self.label = Some(label)
        }
    }

    pub fn is_labeled(&self) -> bool {
        self.label.is_some()
    }

    pub fn has_label(&self, target: &Label) -> bool {
        match &self.label {
            Some(l) => *l == *target,
            None => false,
        }
    }
}

pub fn construct_basic_block(instrs: Vec<Instr>) -> Vec<Node> {
    let mut output = Vec::<Node>::new();
    let mut cur_block = Vec::<Instr>::new();
    for instr in instrs.into_iter() {
        if instr.is_label() {
            if !cur_block.is_empty() {
                output.push(Node::new(cur_block));
            }
            cur_block = vec![instr];
        } else if instr.is_terminator() {
            cur_block.push(instr);
            output.push(Node::new(cur_block));
            cur_block = Vec::<Instr>::new();
        } else {
            cur_block.push(instr);
        }
    }
    if !cur_block.is_empty() {
        output.push(Node::new(cur_block));
    }
    output
}

fn construct_label_lookup(blocks: &[Node]) -> LabelMap {
    let mut map = LabelMap::new();
    for outer in blocks.iter() {
        if outer.0.is_labeled() {
            map.insert(outer.0.label.as_ref().unwrap().clone(), outer.clone());
        }
    }
    map
}

fn connect_block(current: &Node, node: &Node, map: &LabelMap) {
    let &Node(ref block1) = current;
    if block1.out.borrow().is_some() {
        return;
    }

    let instrs = &block1.contents;
    let last = instrs.last().unwrap();
    let (op, labels) = match last {
        Instr::Value { op, labels, .. } | Instr::Effect { op, labels, .. } => (op, labels),
        _ => {
            block1
                .out
                .replace(Some(Link::Fallthrough(node.reference())));
            return;
        }
    };
    match op {
        Op::Jmp => {
            let target = &labels[0];
            let target_ref = map.get(&target).unwrap_or_else(|| {
                panic!(
                    "Unable to locate label {}",
                    name_mapper::namer().get_string(&(target.0))
                )
            });
            block1.out.replace(Some(Link::Jump(target_ref.reference())));
        }
        Op::Br => {
            let true_label = &labels[0];
            let false_label = &labels[1];

            let true_target = map.get(&true_label).unwrap_or_else(|| {
                panic!(
                    "Unable to locate label {}",
                    name_mapper::namer().get_string(&(true_label.0))
                )
            });

            let false_target = map.get(&false_label).unwrap_or_else(|| {
                panic!(
                    "Unable to locate label {}",
                    name_mapper::namer().get_string(&(false_label.0))
                )
            });

            block1.out.replace(Some(Link::Branch {
                true_branch: true_target.reference(),
                false_branch: false_target.reference(),
            }));
        }
        Op::Ret => {
            block1.out.replace(Some(Link::Ret));
        }
        _ => {
            block1
                .out
                .replace(Some(Link::Fallthrough(node.reference())));
        }
    }
}

fn connect_terminal_block(&Node(ref last_block): &Node, map: &LabelMap) {
    let instrs = &last_block.contents;
    let last = instrs.last().unwrap();

    match last {
        Instr::Value { op, labels, .. } | Instr::Effect { op, labels, .. } => match op {
            Op::Jmp => {
                let target = &labels[0];
                let target_ref = map.get(&target).unwrap_or_else(|| {
                    panic!(
                        "Unable to locate label {}",
                        name_mapper::namer().get_string(&(target.0))
                    )
                });
                last_block
                    .out
                    .replace(Some(Link::Jump(target_ref.reference())));
                return;
            }
            Op::Br => {
                let true_label = &labels[0];
                let false_label = &labels[1];

                let true_target = map.get(&true_label).unwrap_or_else(|| {
                    panic!(
                        "Unable to locate label {}",
                        name_mapper::namer().get_string(&(true_label.0))
                    )
                });

                let false_target = map.get(&false_label).unwrap_or_else(|| {
                    panic!(
                        "Unable to locate label {}",
                        name_mapper::namer().get_string(&(false_label.0))
                    )
                });

                last_block.out.replace(Some(Link::Branch {
                    true_branch: true_target.reference(),
                    false_branch: false_target.reference(),
                }));
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

pub fn connect_basic_blocks(blocks: &mut Vec<Node>) {
    let map = construct_label_lookup(blocks);
    let mut second_iter = blocks.iter();
    second_iter.next();

    for (current, node) in blocks.iter().zip(second_iter) {
        connect_block(current, node, &map)
    }

    connect_terminal_block(blocks.last().unwrap(), &map)
}

pub fn construct_cfg(Program { functions: funs }: Program) -> Vec<CFGFunction> {
    funs.into_iter().map(|x| x.form_function_blocks()).collect()
}

impl Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Display for CFGNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(label) = &self.label {
            write!(f, "Block {}:\n",name_mapper::namer().get_string(&label.0));
        } else {
            write!(f, "Block (unlabeled):\n");
        }
        for line in self.contents.iter() {
            write!(f,"     {}\n", line);
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
        let namer = name_mapper::namer();
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
                    match &val.label {
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
                    if let  Some(label) = &val.label {
                        write!(f, "<JUMP: .{}>", namer.get_string(&label.0));
                    }
                    write!(f,"")

                } else {
                    write!(f, "?? LOST CONNECTION ??")
                }
            }
            Link::Branch { true_branch, false_branch } => {
                let val = true_branch.upgrade();
                if let Some(val) = val {
                    if let Some(label) = &val.label{
                        write!(f, "<BR TRUE: .{}>", namer.get_string(&label.0));
                    }
                } else {
                    write!(f, "<BR TRUE:?? LOST CONNECTION ??>");
                }

                write!(f, " ");
                let val = false_branch.upgrade();
                if let Some(val) = val {
                    if let Some(label) = &val.label{
                        write!(f, "<BR FALSE: .{}>", namer.get_string(&label.0));
                    }
                    write!(f,"")
                } else {
                    write!(f, "<BR FALSE:?? LOST CONNECTION ??>")
                }

            }
        }
    }
}
