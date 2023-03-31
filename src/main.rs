mod abs;

use std::io::Write;

use abs::prelude::*;

use clap::{arg, Command, ArgMatches};

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
                .arg(arg!(<tank_name> "name of tank").required(true)),
        )
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

fn get_tank(profile_name: &str) -> Tank {
    Tank::new("abs.toml", profile_name).unwrap_or_else(|err| {
        match err {
            TankError::ConfigFileDoesntExist(_) => {
                println!("Can't find configuration file. Check it")
            }
            TankError::WrongFormatOfToml(message) => {
                println!("TOML format is wrong. Can't parse: {message}")
            }
            TankError::MandatoryLack(message) => println!(
                "Can't find mandatory field in configuration file: {message}"
                
            ),
            TankError::WrongTypeOfField(message) => {
                println!("Type of field is wrong: {message}")
            }
            TankError::SectionError(message) => {
                println!("Got error from section: {message}")
            }
        }
        std::process::exit(1)
    })
}

fn command_new(matches: &ArgMatches) -> Result<(), &str>{
    let tank_name = matches.get_one::<String>("tank_name");
    if tank_name.is_none() {
        return Err("Failed to get name of the tank");
    }
    let tank_name = tank_name.unwrap();
    if std::path::Path::new(tank_name).exists() {
        return Err("Directory already exists");
    }
    if let Err(err) = std::fs::create_dir_all(format!("{tank_name}/.abs/")) {
        println!("{err}");
        return Err("Failed to create project directory directory");
    }
    let file = std::fs::File::create(format!("{tank_name}/abs.toml"));
    if file.is_err() {
        return Err("Failed to create configuration file");
    }
    let file = file.unwrap();
    let mut writer = std::io::BufWriter::new(file);
    if let Err(err) = writer.write(format!("[tank]\nname = \"{tank_name}\"\nversion = \"0.1.0\"").as_bytes()) {
        println!("{err}");
        return Err("Failed to write basic configuration");
    }
    return Ok(());
}

fn main() {
    let matches = cli().get_matches();
    let mut result = false;
    match matches.subcommand() {
        Some(("new", matches)) => {
            if let Err(err) = command_new(&matches) {
                println!("Error: {err}");
            }
        }
        Some(("check", matches)) => {
            let mut profile = String::from("debug");
            if let Some(input) = matches.get_one::<String>("profile") {
                profile = input.clone();
            }
            let tank = get_tank(&profile);
            result = tank.check();
        }
        Some(("build", matches)) => {
            let mut profile = String::from("debug");
            if let Some(input) = matches.get_one::<String>("profile") {
                profile = input.clone();
            }
            let tank = get_tank(&profile);
            result = tank.build();
        }
        Some(("run", matches)) => {
            let mut profile = String::from("debug");
            if let Some(input) = matches.get_one::<String>("profile") {
                profile = input.clone();
            }
            let tank = get_tank(&profile);
            result = tank.run();
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
