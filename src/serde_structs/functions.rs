use std::fmt::{self, Display};
use serde::{self, Deserialize, Serialize};
use super::names::{FnName, namer, Var};
use super::basic_types::{Type, Literal};
use super::instructions::Instr;
use super::operations::Op;
use super::super::transformers::cfg::Node;
use super::super::transformers::cfg::{connect_basic_blocks, construct_basic_blocks, construct_cfg_nodes};
use super::super::transformers::orphan::remove_inaccessible_blocks;
use super::super::transformers::dce::{trivial_global_dce,local_dce};
use super::super::transformers::lvn::run_lvn;
use std::rc::Rc;
use crate::analysis;
use crate::analysis::reaching_defns::VarDef;

use std::mem::replace;
#[derive(Serialize, Deserialize, Debug)]
pub struct FnHeaders {
    pub name: Var,
    #[serde(rename = "type")]
    r_type: Type,
}

impl FnHeaders {
    // Only should be used for dataflow analysis
    pub fn generate_dummy_instrs(&self) -> Instr {
        Instr::Const { op: Op::Const, dest: self.name, r_type: self.r_type.clone(),
                value: match self.r_type {
                    Type::Int => {Literal::Int(0)}
                    Type::Bool => {Literal::Bool(false)}
                    Type::Ptr(_) => {todo!()}
                    Type::Float => {Literal::Float(0.0)}
                }}
    }
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

    pub fn make_cfg(self) -> CFGFunction {

        let mut blocks = construct_cfg_nodes(construct_basic_blocks(self.instrs));

        connect_basic_blocks(&mut blocks);

        CFGFunction {
            name: self.name,
            args: self.args,
            r_type: self.r_type,
            blocks
        }
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
    pub fn apply_lvn(&mut self) {
        for blk in self.blocks.iter() {
            let node = blk.as_ref();
            let contents = &mut node.contents.borrow_mut().0;
            run_lvn(contents)
        }
    }

    pub fn reaching_defns(&self) {
        let analysis_nodes = analysis::reaching_definitions(&self.blocks, &self.args);

        println!("\n\nRunning reaching definitions analysis on {}\n", self.name);
        for (index, node) in analysis_nodes.into_iter().enumerate() {
            let mut out_vars = node.out_data_as_vec();
            out_vars.sort_by(|x: &VarDef, y: &VarDef|  {
                (x.0, x.1).cmp(&(y.0, y.1))
            });

            let mut in_vars = node.in_data_as_vec();
            in_vars.sort_by(|x: &VarDef, y: &VarDef|  {
                (x.0, x.1).cmp(&(y.0, y.1))
            });
            if index == 0 {
                println!("Function start:");
                print!(" Input:");

                for var in out_vars  {
                    print!(" {}", var);
                }
                println!("\n")
            } else {
                println!("Block {} [{}]", index, self.blocks[index - 1].contents.borrow());
                print!(" Input:");
                for var in in_vars {
                    print!(" {}", var);
                }
                print!("\n Output:");
                for var in out_vars {
                    print!(" {}", var);
                }
                println!("\n")
            }


        }
    }

    pub fn live_vars(&self) {
        let analysis_nodes = analysis::live_variables(&self.blocks);

        println!("\n\nRunning live variable analysis on {}\n", self.name);
        for (index, node) in analysis_nodes.into_iter().enumerate() {

            let mut out_vars = node.out_data_as_vec();
            out_vars.sort_by(|x: &Var, y: &Var|  {
                x.cmp(&y)
            });

            let mut in_vars = node.in_data_as_vec();
            in_vars.sort_by(|x: &Var, y: &Var|  {
                x.cmp(&y)
            });
            println!("Block {} [{}]", index, self.blocks[index].contents.borrow());
            print!(" Input:");
            for var in in_vars {
                print!(" {}", var);
            }
            print!("\n Output:");
            for var in out_vars {
                print!(" {}", var);
            }
            println!("\n")
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
