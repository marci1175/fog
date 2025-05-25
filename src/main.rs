use fog::ApplicationError;
use fog::app::cli_parser::cli_parser::{CliCommand, parse_args};
use fog::app::codegen::error::CodeGenError;
use fog::app::compiler::compiler::{CompilerConfig, CompilerState};
pub use fog_lib;
use std::env;
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

    let current_working_dir = env::current_dir()?;

    let command = args.next().unwrap_or_default();

    let argument = args.next().unwrap_or_default();

    let (command, arg) = parse_args(command, argument);

    match command {
        CliCommand::Compile => {
            println!("Reading Path...");

            // Check for the main source file
            println!("Reading File...");
            let source_file =
                fs::read_to_string(format!("{}/src/main.f", current_working_dir.display()))
                    .map_err(|_| ApplicationError::CodeGenError(CodeGenError::NoMain.into()))?;

            // Read config file
            let config_file =
                fs::read_to_string(format!("{}/config.toml", current_working_dir.display()))
                    .map_err(ApplicationError::FileError)?;

            let parsed_config = toml::from_str::<CompilerConfig>(&config_file)
                .map_err(ApplicationError::ConfigError)?;

            let compiler_state = CompilerState::new(parsed_config.clone());

            let target_path = PathBuf::from(format!(
                "{}/output/{}.ll",
                current_working_dir.display(),
                parsed_config.name.clone()
            ));

            let release_flag = arg.display().to_string();

            compiler_state.compilation_process(
                source_file,
                target_path.clone(),
                release_flag == "release" || release_flag == "r",
            )?;

            println!(
                "Compilation finished, output file is located at: {:?}",
                fs::canonicalize(target_path).unwrap_or_default()
            );
        }
        CliCommand::Help => display_help_prompt(),
        CliCommand::Version => println!("Build version: {}", env!("CARGO_PKG_VERSION")),
        CliCommand::New => {
            let working_folder = format!("{}/{}", current_working_dir.display(), arg.display());

            fs::create_dir_all(&working_folder).map_err(ApplicationError::FileError)?;
            fs::create_dir_all(format!("{working_folder}/src"))?;

            fs::write(
                format!("{}/src/main.f", working_folder),
                include_str!("../defaults/default_code.f"),
            )
            .map_err(ApplicationError::FileError)?;

            fs::write(
                format!("{}/config.toml", working_folder),
                toml::to_string(&CompilerConfig::new(
                    arg.file_name().unwrap().to_string_lossy().to_string(),
                    false,
                ))?,
            )
            .map_err(ApplicationError::FileError)?;

            fs::create_dir_all(format!("{}/output", working_folder))?;
        }
        CliCommand::Init => {
            let get_folder_name = current_working_dir
                .file_name()
                .unwrap_or_default()
                .to_string_lossy();

            fs::create_dir(format!("{}/output", current_working_dir.display()))
                .map_err(ApplicationError::FileError)?;
            fs::create_dir(format!("{}/src", current_working_dir.display()))
                .map_err(ApplicationError::FileError)?;

            fs::write(
                format!("{}/src/main.f", current_working_dir.display()),
                include_str!("../defaults/default_code.f"),
            )?;

            fs::write(
                &current_working_dir,
                toml::to_string(&CompilerConfig::new(get_folder_name.to_string(), false))?,
            )
            .map_err(ApplicationError::FileError)?;
        }
    }

    Ok(())
}
