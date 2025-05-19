use fog::ApplicationError;
use fog::app::{
    cli_parser::cli_parser::{CliCommand, parse_args},
    compiler,
};
pub use fog_lib;
use std::{fs, path::PathBuf};
use strum::{EnumMessage, VariantArray};

fn display_help_prompt() {
    println!("Help:");
    println!("Here is a list of commands you can use:");

    for (idx, command) in CliCommand::VARIANTS.iter().enumerate() {
        println!("{}. {}", idx + 1, command.get_message().unwrap())
    }
}

fn main() -> anyhow::Result<()> {
    let mut args = std::env::args();

    let _path_to_file = args.next().unwrap_or_default();

    let command = args.next().unwrap_or_default();

    let argument = args.next().unwrap_or_default();

    let (command, arg) = parse_args(command, argument);

    match command {
        CliCommand::Compile => {
            println!("Reading file path...");

            fs::metadata(&arg).map_err(ApplicationError::FileError)?;

            let path_to_out = PathBuf::from(args.next().unwrap_or_default());

            compiler::compiler::compilation_process(
                PathBuf::from(arg),
                path_to_out.clone(),
                args.next().unwrap_or_default() == "release"
                    || args.next().unwrap_or_default() == "r",
            )?;

            println!(
                "Compilation finished, output file is located at: {:?}",
                fs::canonicalize(path_to_out).unwrap_or_default()
            );
        }
        CliCommand::Help => display_help_prompt(),
        CliCommand::Version => println!("Build version: {}", env!("CARGO_PKG_VERSION")),
    }

    Ok(())
}
