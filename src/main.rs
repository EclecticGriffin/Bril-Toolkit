mod bril;

use std::io::{self, Read};
use std::process::exit;

use serde_json;


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
    print!("{}\n", serde_json::to_string(&v).unwrap());
}
