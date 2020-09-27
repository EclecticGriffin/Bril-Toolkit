mod serde_structs;
mod transformers;
mod analysis;

use std::io::{self, Read};
use std::process::exit;
use serde_structs::structs::{Program, CFGFunction};
use clap::{Arg, App, SubCommand};
use transformers::config::ConfigOptions;



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

fn apply_transformations(mut prog: Program, conf: ConfigOptions) -> Program {

    if conf.g_tdce {
        // eprintln!("Applying trivial global dce");
        for fun in prog.functions.iter_mut() {
            fun.g_tcde()
        }
    }

    if conf.lvn.run_lvn() || conf.l_tdce || conf.orphan_block {
        let mut cfg = prog.determine_cfg();
        // for fun in cfg.functions.iter() {
        //     eprintln!("{:?}", fun)
        // }
        // eprintln!("\n\n\n\n\n");
        if conf.orphan_block {
        // eprintln!("Applying orphan block removal");
            for fun in cfg.functions.iter_mut() {
                fun.drop_orphan_blocks()
            }
        }

        if conf.l_tdce {
        // eprintln!("Applying trivial local dce");
            for fun in cfg.functions.iter_mut() {
                fun.apply_basic_dce()
            }
        }

        if conf.lvn.run_solo() {
        // eprintln!("Applying lvn solo");
            for fun in cfg.functions.iter_mut() {
                fun.apply_lvn()
            }
        }

        if conf.lvn.run_normal() {
        // eprintln!("Applying lvn normal");
            for fun in cfg.functions.iter_mut() {
                fun.apply_lvn();
                fun.apply_basic_dce()
            }
        }

        prog = cfg.make_serializeable()
    }

    if conf.lvn.run_normal() {
        for fun in prog.functions.iter_mut() {
            fun.g_tcde()
        }
    }

    prog
}

fn main() {
    let matches = App::new("Bril Toolkit").version("0.1")
                    .author("Griffin Berlstein <griffin@berlste.in>")
                    .about("A toolkit for bril transformations")
                    .subcommand(
                    SubCommand::with_name("transform")
                                .version("0.1")
                                .author("Griffin Berlstein <griffin@berlste.in>")
                                .about("Apply transformations to a bril program")
                                .arg(Arg::with_name("optimizations")
                                .short("o")
                                .long("optimizations")
                                .multiple(true)
                                .takes_value(true)
                                .possible_values(&transformers::config::ALLOWED_VALUES)
                    ))
                    .subcommand(
                        SubCommand::with_name("analyze")
                                .version("0.1")
                                .author("Griffin Berlstein <griffin@berlste.in>")
                                .about("Perform dataflow analyses")
                                .arg(Arg::with_name("analysis")
                                     .short("a")
                                     .long("analysis")
                                     .index(1)
                                     .takes_value(true)
                                     .possible_values(&analysis::ALLOWED_VALUES)
                                     .required(true)
                    ))
                    .get_matches();



    let buffer = get_stdin();

    match matches.subcommand() {
        ("transform", Some(sub_m)) => {
            let optimizations = sub_m.values_of("optimizations");

            // If there are no optimizations just return what was given
            if let None = optimizations {
                println!("{}", buffer);
                exit(0)
            }

            let confs = ConfigOptions::new(optimizations.unwrap());

            let mut prog: Program = serde_json::from_str(&buffer).unwrap();

            prog = apply_transformations(prog, confs);
            println!("{}", serde_json::to_string_pretty(&prog).ok().unwrap_or_default());
            }
        ("analyze", Some(sub_m)) => {
            let func = match sub_m.value_of("analysis") {
                Some("reaching_defns") => {
                    CFGFunction::reaching_defns
                }
                _ => {exit(1)}
            };

            let prog: Program = serde_json::from_str(&buffer).unwrap();

            let cfg = prog.determine_cfg();


            for cfg_fun in cfg.functions.iter(){
                func(cfg_fun)
            }

        }
        _ => {}
    }




}
