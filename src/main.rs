mod abs;

use abs::prelude::*;

use clap::Command;

fn cli() -> Command {
    Command::new("abs")
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
        )
        .subcommand(
            Command::new("run")
                .about("Builds and runs")
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
        Some(("build", _)) => {
            result = tank.build();
        },
        Some(("run", _)) => {
            result = tank.run();
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