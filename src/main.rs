use fog::ApplicationError;
use fog::app::compiler::compiler::{CompilerConfig, CompilerState};
use fog::app::cli_parser::cli_parser::{CliCommand, parse_args};
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

    let (command, mut arg) = parse_args(command, argument);

    match command {
        CliCommand::Compile => {
            println!("Reading file...");
            // Check for source file
            let source_file = fs::read_to_string(&arg).map_err(ApplicationError::FileError)?;

            // Pop last item
            arg.pop();

            arg.push("config.toml");

            // Read config file
            let config_file = fs::read_to_string(&arg).map_err(ApplicationError::FileError)?;

            let parsed_config = toml::from_str::<CompilerConfig>(&config_file)
                .map_err(ApplicationError::ConfigError)?;

            let path_to_out = PathBuf::from(args.next().unwrap_or_default());

            arg.pop();

            arg.push(format!("output/{}.ll", parsed_config.name.clone()));

            let compiler_state = CompilerState::new(parsed_config);

            compiler_state.compilation_process(
                source_file,
                arg.clone(),
                args.next().unwrap_or_default() == "release"
                    || args.next().unwrap_or_default() == "r",
            )?;

            println!(
                "Compilation finished, output file is located at: {:?}",
                fs::canonicalize(arg).unwrap_or_default()
            );
        }
        CliCommand::Help => display_help_prompt(),
        CliCommand::Version => println!("Build version: {}", env!("CARGO_PKG_VERSION")),
        CliCommand::New => {
            let mut path_to_folder = arg;

            fs::create_dir_all(&path_to_folder).map_err(ApplicationError::FileError)?;

            path_to_folder.push("main.f");

            fs::write(&path_to_folder, include_str!("../defaults/default_code.f"))
                .map_err(ApplicationError::FileError)?;

            path_to_folder.pop();

            path_to_folder.push("config.toml");

            fs::write(
                &path_to_folder,
                toml::to_string(&CompilerConfig::default())?,
            )
            .map_err(ApplicationError::FileError)?;

            path_to_folder.pop();

            path_to_folder.push("output");

            fs::create_dir_all(dbg!(path_to_folder))?;
        }
    }

    Ok(())
}
