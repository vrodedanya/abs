mod abs;

use abs::prelude::*;

use clap::{Command, arg};

fn cli() -> Command {
    Command::new("abs")
        .infer_subcommands(true)
        .about("Another build system for C++. Under development")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .subcommand(
            Command::new("files")
                .about("Shows collected files ")
        )
        .subcommand(
            Command::new("check")
                .about("Checks that the current section can be built")
        )
        .subcommand(
            Command::new("build")
                .about("Builds the current section")
                .arg(
                    arg!(-p --profile <PROFILE> "Sets profile for building")
                    .required(false)
                )
        )
        .subcommand(
            Command::new("run")
                .about("Builds and runs")
                .arg(
                    arg!(-p --profile <PROFILE> "Sets profile for running")
                    .required(false)
                )
        )
}

fn main() {
    let matches = cli().get_matches();
    let tank = Tank::new("abs.toml").unwrap();
    let mut result = false;
    match matches.subcommand() {
        Some(("files", _)) => {
            tank.print_sections();
        },
        Some(("check", _)) => {
            result = tank.check();
        },
        Some(("build", matches)) => {
            let mut profile = String::from("debug");
            if let Some(input) = matches.get_one::<String>("profile") {
                profile = input.clone();
            }
            result = tank.build(&profile);
        },
        Some(("run", matches)) => {
            let mut profile = String::from("debug");
            if let Some(input) = matches.get_one::<String>("profile") {
                profile = input.clone();
            }
            result = tank.run(&profile);
        },
        None => {
            println!("Unexpected command")
        },
        _ => unreachable!()
    }

    if result {
        std::process::exit(0);
    } else {
        std::process::exit(1);
    }
}