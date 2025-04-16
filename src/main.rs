use std::{fs, path::PathBuf};

use app::{
    cli_parser::cli_parser::{CliCommand, parse_args},
    compiler,
};
use fog::CompilerError;
use strum::{EnumMessage, VariantArray};

pub mod app;

fn display_help_prompt() {
    println!("Help:");
    println!("Here is a list of commands you can use:");

    for (idx, command) in CliCommand::VARIANTS.iter().enumerate() {
        println!("{}. {}", idx + 1, command.get_message().unwrap())
    }
}

fn main() -> anyhow::Result<()> {
    let mut args = std::env::args();

    let path_to_file = args.next().unwrap_or_default();

    let command = args.next().unwrap_or_default();

    let argument = args.next().unwrap_or_default();

    let (command, arg) = parse_args(command, argument)?;

    match command {
        CliCommand::Compile => {
            fs::metadata(&arg).map_err(CompilerError::FileError)?;

            compiler::compiler::compilation_process(PathBuf::from(arg))?;
        }
        CliCommand::Help => display_help_prompt(),
        CliCommand::Version => println!("Build version: {}", env!("CARGO_PKG_VERSION")),
    }

    Ok(())
}
