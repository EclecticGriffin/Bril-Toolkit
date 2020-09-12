mod bril;
mod utils;

use std::io::{self, Read};
use std::process::exit;

use serde_json;
use bril::transformers::cfg_transformers::dead_block::remove_inaccessible_blocks;

fn main() {
    let stdin = io::stdin();
    let mut buffer = String::new();
    let mut handle = stdin.lock();

    match handle.read_to_string(&mut buffer) {
        Ok(n) => {}
        Err(error) => {
            eprint!("Encountered error {}", error);
            exit(1)
        }
    }

    let v: bril::Program = serde_json::from_str(&buffer).unwrap();
    let mut funcs = bril::transformers::cfg::construct_cfg(v);
    for func in funcs.iter() {
        print!("{}\n\n\n", func);
    }

    println!("\n\n making modifications \n\n");

    funcs = funcs.into_iter().map(|x| remove_inaccessible_blocks(x)).collect();

    for func in funcs {
        print!("{}\n\n\n", func);
    }

    // println!("{}", serde_json::to_string(&v).unwrap());
}
