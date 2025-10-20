mod cli;
use fog_common::{
    anyhow,
    compiler::ProjectConfig,
    error::{application::ApplicationError, cliparser::CliParseError, codegen::CodeGenError},
    toml,
};
use fog_compiler::CompilerState;
use fog_linker::link;
use std::{collections::HashMap, env, fs, path::PathBuf};
use strum::{EnumMessage, VariantArray};

use crate::cli::{CliCommand, parse_args};

fn main() -> fog_common::anyhow::Result<()>
{
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

            // Read config file
            let config_file =
                fs::read_to_string(format!("{}/config.toml", current_working_dir.display()))
                    .map_err(|_| ApplicationError::ConfigNotFound(current_working_dir.clone()))?;

            let source_file =
                fs::read_to_string(format!("{}/src/main.f", current_working_dir.display()))
                    .map_err(|_| ApplicationError::CodeGenError(CodeGenError::NoMain.into()))?;

            let compiler_config = toml::from_str::<ProjectConfig>(&config_file)
                .map_err(ApplicationError::ConfigError)?;

            let compiler_state =
                CompilerState::new(compiler_config.clone(), current_working_dir.clone());

            let target_ir_path = PathBuf::from(format!(
                "{}\\output\\{}.ll",
                current_working_dir.display(),
                compiler_config.name.clone()
            ));

            let target_o_path = PathBuf::from(format!(
                "{}\\output\\{}.obj",
                current_working_dir.display(),
                compiler_config.name.clone()
            ));

            // Make this not so specific later
            let build_path = PathBuf::from(format!(
                "{}\\output\\{}.exe",
                current_working_dir.display(),
                compiler_config.name.clone()
            ));

            let build_manifest_path = PathBuf::from(format!(
                "{}\\output\\{}.manifest",
                current_working_dir.display(),
                compiler_config.name.clone()
            ));

            let release_flag = arg.display().to_string();

            let build_manifest = compiler_state.compilation_process(
                &source_file,
                target_ir_path.clone(),
                target_o_path.clone(),
                build_path.clone(),
                release_flag == "release" || release_flag == "r",
                compiler_config.is_library,
                &format!("{}\\src", current_working_dir.display(),),
            )?;

            // Write build manifest to disc
            fs::write(build_manifest_path, toml::to_string(&build_manifest)?)?;

            println!("All build artifacts have been saved.");

            // Link automaticly
            link(&build_manifest).map_err(anyhow::Error::from)?;

            println!(
                "Linking finished successfully! Binary output is available at: {}",
                build_path.display()
            );
        },
        CliCommand::Help => display_help_prompt(),
        CliCommand::Version => println!("Build version: {}", env!("CARGO_PKG_VERSION")),
        CliCommand::New => {
            let working_folder = format!("{}/{}", current_working_dir.display(), arg.display());

            println!("Creating project folders...");

            fs::create_dir_all(&working_folder).map_err(ApplicationError::FileError)?;
            fs::create_dir(format!("{working_folder}/output"))?;
            fs::create_dir(format!("{working_folder}/deps"))?;
            fs::create_dir(format!("{working_folder}/src"))?;

            fs::write(
                format!("{}/src/main.f", working_folder),
                (|| {
                    if let Some(argument) = args.next() {
                        if argument == "demo" {
                            return Ok(include_str!("../../../defaults/default_code.f"));
                        }
                        else {
                            return Err(ApplicationError::CliParseError(
                                CliParseError::InvalidArg(argument),
                            ));
                        }
                    }

                    Ok("")
                })()?,
            )
            .map_err(ApplicationError::FileError)?;

            fs::write(
                format!("{}/config.toml", working_folder),
                toml::to_string(&ProjectConfig::new(
                    arg.file_name().unwrap().to_string_lossy().to_string(),
                    false,
                    "0.0.1".to_string(),
                    HashMap::new(),
                ))?,
            )
            .map_err(ApplicationError::FileError)?;

            fs::create_dir_all(format!("{}/output", working_folder))?;
        },
        CliCommand::Init => {
            println!("Getting folder name...");

            let get_folder_name = current_working_dir
                .file_name()
                .unwrap_or_default()
                .to_string_lossy();

            println!("Creating project folders...");
            fs::create_dir(format!("{}/output", current_working_dir.display()))
                .map_err(ApplicationError::FileError)?;
            fs::create_dir(format!("{}/deps", current_working_dir.display()))
                .map_err(ApplicationError::FileError)?;
            fs::create_dir(format!("{}/src", current_working_dir.display()))
                .map_err(ApplicationError::FileError)?;

            println!("Creating main source file...");
            fs::write(
                format!("{}/src/main.f", current_working_dir.display()),
                include_str!("../../../defaults/default_code.f"),
            )?;

            println!("Creating config file...");
            fs::write(
                format!("{}/config.toml", current_working_dir.display()),
                toml::to_string(&ProjectConfig::new(
                    get_folder_name.to_string(),
                    false,
                    "0.0.1".to_string(),
                    HashMap::new(),
                ))?,
            )
            .map_err(ApplicationError::FileError)?;

            println!(
                "Successfully initalized a project at: {}",
                current_working_dir.display()
            );
        },
    }

    Ok(())
}

fn display_help_prompt()
{
    println!("Commands available to use:");

    for (idx, command) in CliCommand::VARIANTS.iter().enumerate() {
        println!("{}. {}", idx + 1, command.get_message().unwrap())
    }
}
