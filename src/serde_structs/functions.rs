use std::fmt::{self, Display};
use serde::{self, Deserialize, Serialize};
use super::names::{FnName, namer, Var};
use super::basic_types::Type;
use super::instructions::Instr;
use super::super::transformers::cfg::Node;
use super::super::transformers::cfg::{connect_basic_blocks, construct_basic_blocks, construct_cfg_nodes};
use super::super::transformers::orphan::remove_inaccessible_blocks;
use super::super::transformers::dce::{trivial_global_dce,local_dce};
use crate::config::ConfigOptions;
use std::rc::Rc;

use std::mem::replace;
#[derive(Serialize, Deserialize, Debug)]
pub struct FnHeaders {
    name: Var,
    #[serde(rename = "type")]
    r_type: Type,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Function {
    name: FnName,
    #[serde(default = "Vec::new")]
    args: Vec<FnHeaders>,

    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    r_type: Option<Type>,

    instrs: Vec<Instr>,
}

impl Function {

    pub fn g_tcde(&mut self) {
        trivial_global_dce(&mut self.instrs)
    }

    pub fn make_cfg(mut self, confs: &ConfigOptions) -> CFGFunction {
        if confs.g_tdce {
            self.g_tcde();
        }
        let mut blocks = construct_cfg_nodes(construct_basic_blocks(self.instrs));

        connect_basic_blocks(&mut blocks);

        let mut cfg = CFGFunction {
            name: self.name,
            args: self.args,
            r_type: self.r_type,
            blocks
        };

        if confs.l_tdce {
            cfg.apply_basic_dce();
        }

        if confs.orphan_block {
            cfg.drop_orphan_blocks();
        }

        cfg
    }
}

#[derive(Debug)]
pub struct CFGFunction {
    name: FnName,
    args: Vec<FnHeaders>,

    r_type: Option<Type>,

    blocks: Vec<Rc<Node>>,
}

impl CFGFunction {
    pub fn make_serializeable(self) -> Function {
        Function {
            name: self.name,
            args: self.args,
            r_type: self.r_type,
            instrs: self.blocks.into_iter().map(|x| Rc::try_unwrap(x).unwrap().make_serializeable()).flatten().collect()
        }
    }

    pub fn drop_orphan_blocks(&mut self) {
        let tmp = replace(&mut self.blocks, Vec::new());

        self.blocks = remove_inaccessible_blocks(tmp);
    }

    pub fn apply_basic_dce(&mut self) {
        for block in self.blocks.iter_mut() {
            local_dce(block);
        }
    }
}

impl Display for CFGFunction {
    fn fmt(&self, f: & mut fmt::Formatter<'_>) -> fmt::Result {
        let namer = namer();
        writeln!(f, "== Function: {} ==", namer.get_string(&self.name.0))?;
        writeln!(f, "args: {:?}", &self.args)?;
        if let Some(x) = &self.r_type {
            writeln!(f, "returns {}", x)?;
        }
        for node in self.blocks.iter() {
            writeln!(f, "\n{}", node)?;
        }
        Ok(())
    }
}
