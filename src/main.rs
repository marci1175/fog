use std::{fs, path::PathBuf};

use app::{cli_parser::cli_parser::parse_args, compiler};

pub mod app;

fn main() -> anyhow::Result<()> {
    let mut args = std::env::args();

    let command = args.next().unwrap();

    let argument = args.next().unwrap();

    let (command, arg) = parse_args(command, argument)?;

    // Check if the file is valid
    fs::metadata(&arg)?;

    match command {
        app::cli_parser::cli_parser::CliCommand::Compile => {
            compiler::compiler::compilation_process(PathBuf::from(arg))?;
        }
        app::cli_parser::cli_parser::CliCommand::Help => println!(""),
    }

    Ok(())
}
