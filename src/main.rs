mod serde_structs;
mod transformers;
mod config;

use std::io::{self, Read};
use std::process::exit;
use std::collections::HashMap;
use serde_structs::structs::Program;
use clap::{Arg, App, SubCommand};




fn get_stdin() -> String {
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
    buffer
}

fn main() {
    let matches = App::new("Bril Toolkit").version("0.1")
                    .author("Griffin Berlstein <griffin@berlste.in>")
                    .about("A toolkit for bril transformations")
                    .arg(Arg::with_name("optimizations")
                        .short("o")
                        .long("--optimizations")
                        .multiple(true)
                        .takes_value(true)
                        .possible_values(&config::allowed_values)
                    ).get_matches();

    let optimizations = matches.values_of("optimizations");

    let buffer = get_stdin();


    // If there are no optimizations just return what was given
    if let None = optimizations {
        println!("{}", buffer);
        exit(0)
    }

    let confs = config::ConfigOptions::new(optimizations.unwrap());

    let v: Program = serde_json::from_str(&buffer).unwrap();
    let v = v.determine_cfg(&confs);

    let v = v.make_serializeable();
    println!("{}", serde_json::to_string_pretty(&v).ok().unwrap_or_default());
}
