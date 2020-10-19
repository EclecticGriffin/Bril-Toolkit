use fnv::FnvHashMap as HashMap;

use super::{Label, Instr};


pub struct Graph {
    map: HashMap<Label, Block>,
    root: Label,
}

// This struct exists to retain whether or not a given block label was
// originally present or generated for the block. Used for serializing the
// blocks to avoid emitting temp labels on blocks that already existed
#[derive(Debug)]
struct BlockLabel {
    label: Label,
    is_real: bool
}

impl BlockLabel {
    pub fn real(label: Label) -> Self {
        Self {
            label,
            is_real: true
        }
    }

    pub fn gen_temp() -> Self {
        Self {
            label: Label::default(),
            is_real:false
        }
    }

    pub fn gen_real() -> Self {
        Self {
            label: Label::default(),
            is_real: true
        }
    }

    pub fn as_label(&self) -> Label {
        self.label
    }
}

#[derive(Debug)]
pub struct Block {
    contents: Vec<Instr>,
    label: BlockLabel
}

impl Block {
    pub fn new(mut input: Vec<Instr>) -> Self {
        let label: BlockLabel = match input.first() {
            Some(x) => {
                if let Instr::Label {label} = x {
                    let label = input.remove(0);
                    if let Instr::Label {label} = label {
                        BlockLabel::real(label)
                    } else {
                        // this will never run
                        BlockLabel::gen_temp()
                    }
                } else {
                    BlockLabel::gen_temp()
                }
            }
            None => {
                BlockLabel::gen_temp()
            }
        };

        Block {
            contents: input,
            label
        }
    }

    pub fn is_empty(&self) -> bool {
        self.contents.is_empty()
    }

    pub fn label(&self) -> Label {
        self.label.as_label()
    }

    pub fn last(&self) -> Option<&Instr> {
        self.contents.last()
    }

    pub fn len(&self) -> usize {
        self.contents.len()
    }

    fn make_serializeable(mut self) -> Vec<Instr> {
        if self.label.is_real {
            self.contents.insert(0, self.label.as_label().as_instr());
            self.contents
        } else {
            self.contents
        }
    }

    fn iter(&self) -> std::slice::Iter<Instr> {
        self.contents.iter()
    }

    fn iter_mut(&mut self) -> std::slice::IterMut<Instr> {
        self.contents.iter_mut()
    }

}

impl Default for Block {
    fn default() -> Self {
        Block {
            contents: Vec::new(),
            label: BlockLabel::gen_temp()
        }
    }
}
