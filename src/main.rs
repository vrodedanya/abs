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
                .about("Check that the current section can be built")
        )
        .subcommand(
            Command::new("build")
                .about("Build the current section")
        )
}

fn main() {
    let matches = cli().get_matches();
    let tank = Tank::new("abs.toml").unwrap();
    match matches.subcommand() {
        Some(("files", _)) => {
            tank.print_sections();
        },
        Some(("check", _)) => {
            if tank.check() {
                std::process::exit(0)
            } else {
                std::process::exit(1)
            }
        },
        Some(("build", _)) => {
            if tank.build() {
                std::process::exit(0)
            } else {
                std::process::exit(1)
            }
        },
        None => {
            println!("Unexpected command")
        },
        _ => unreachable!()
    }
}