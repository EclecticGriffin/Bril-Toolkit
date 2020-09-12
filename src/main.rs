mod serde_structs;
mod transformers;

use std::io::{self, Read};
use std::process::exit;
use serde_structs::structs::Program;


fn main() {
    let stdin = io::stdin();
    let mut buffer = String::new();
    let mut handle = stdin.lock();

    match handle.read_to_string(&mut buffer) {
        Ok(_) => {}
        Err(error) => {
            eprint!("Encountered error {}", error);
            exit(1)
        }
    }

    let v: Program = serde_json::from_str(&buffer).unwrap();
    let mut v = v.determine_cfg();

    // for func in v.functions.iter() {
    //     print!("{}\n\n\n", func);
    // }

    // println!("\n\n making modifications \n\n");



    for _ in v.functions.iter_mut().map(|f| f.drop_orphan_blocks()) {}

    // for func in v.functions.iter() {
    //     print!("{}\n\n\n", func);
    // }

    let v = v.make_serializeable();
    println!("{}", serde_json::to_string(&v).ok().unwrap_or_default());
    // println!("{}", serde_json::to_string(&v).unwrap());
}
