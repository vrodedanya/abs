mod abs;

use std::io::Write;

use abs::prelude::*;

use clap::{arg, Command};

fn cli() -> Command {
    Command::new("abs")
        .infer_subcommands(true)
        .about("Another build system for C++. Under development")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .subcommand(
            Command::new("new")
                .about("Creates new tank directory")
                .arg(arg!(<NAME> "name of tank").required(true)),
        )
        .subcommand(Command::new("files").about("Shows collected files"))
        .subcommand(
            Command::new("check")
                .about("Checks that the current section can be built")
                .arg(arg!(-p --profile <PROFILE> "Sets profile for checking").required(false)),
        )
        .subcommand(
            Command::new("build")
                .about("Builds the current section")
                .arg(arg!(-p --profile <PROFILE> "Sets profile for building").required(false)),
        )
        .subcommand(
            Command::new("run")
                .about("Builds and runs")
                .arg(arg!(-p --profile <PROFILE> "Sets profile for running").required(false)),
        )
}

fn get_tank() -> Tank {
    Tank::new("abs.toml").unwrap_or_else(|err| {
        match err {
            TankError::ConfigFileDoesntExist(_) => {
                println!("Can't find configuration file. Check it")
            }
            TankError::WrongFormatOfToml(message) => {
                println!("TOML format is wrong. Can't parse: {}", message)
            }
            TankError::MandatoryLack(message) => println!(
                "Can't find mandatory field in configuration file: {}",
                message
            ),
            TankError::WrongTypeOfField(message) => {
                println!("Type of field is wrong: {}", message)
            }
            TankError::SectionError(message) => {
                println!("Got error from section: {}", message)
            }
        }
        std::process::exit(1)
    })
}

fn main() {
    let matches = cli().get_matches();
    let mut result = false;
    match matches.subcommand() {
        Some(("new", matches)) => {
            if let Some(tank_name) = matches.get_one::<String>("name") {
                if let Err(err) = std::fs::create_dir_all(format!("{}/.abs/", tank_name)) {
                    println!("Failed to create directory: {}", err);
                    std::process::exit(1);
                }
                if let Ok(file) = std::fs::File::create(format!("{}/abs.toml", tank_name)) {
                    let mut writer = std::io::BufWriter::new(file);
                    if let Err(err) = writer.write(
                        format!("[tank]\nname = {}\nversion = \"0.1.0\"", tank_name).as_bytes(),
                    ) {
                        println!("Failed to write to the configuration file: {}", err);
                    }
                } else {
                    println!("Failed to create configuration file");
                    std::process::exit(1);
                }
            } else {
                println!("Failed to get name of the tank");
            }
        }
        Some(("files", _)) => {
            let tank = get_tank();
            tank.print_sections();
        }
        Some(("check", matches)) => {
            let tank = Tank::new("abs.toml").unwrap();
            let mut profile = String::from("debug");
            if let Some(input) = matches.get_one::<String>("profile") {
                profile = input.clone();
            }
            result = tank.check(&profile);
        }
        Some(("build", matches)) => {
            let tank = get_tank();
            let mut profile = String::from("debug");
            if let Some(input) = matches.get_one::<String>("profile") {
                profile = input.clone();
            }
            result = tank.build(&profile);
        }
        Some(("run", matches)) => {
            let tank = get_tank();
            let mut profile = String::from("debug");
            if let Some(input) = matches.get_one::<String>("profile") {
                profile = input.clone();
            }
            result = tank.run(&profile);
        }
        None => {
            println!("Unexpected command")
        }
        _ => unreachable!(),
    }

    if result {
        std::process::exit(0);
    } else {
        std::process::exit(1);
    }
}
