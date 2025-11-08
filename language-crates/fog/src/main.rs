mod cli;

use crate::cli::CliCommand;
use clap::Parser;
use fog_common::{
    anyhow, clap,
    compiler::ProjectConfig,
    error::{application::ApplicationError, codegen::CodeGenError, linker::LinkerError},
    linker::BuildManifest,
    toml,
    ty::OrdSet,
};
use fog_compiler::CompilerState;
use fog_linker::link;
use std::{env, fs, path::PathBuf};
use strum::{VariantArray, VariantNames};

#[derive(Parser, Debug)]
#[command(version, about, long_about)]
pub struct CompilerArgs
{
    #[command(subcommand)]
    command: CliCommand,
}

fn main() -> fog_common::anyhow::Result<()>
{
    let args = std::env::args();

    let current_working_dir = env::current_dir()?;

    let compiler_args = CompilerArgs::parse();
    let compiler_command = compiler_args.command;

    match compiler_command.clone() {
        CliCommand::Link { path } => {
            println!("Reading file on: `{}`", path.display());

            let manifest_string = fs::read_to_string(&path)?;

            let manifest = toml::from_str::<BuildManifest>(&manifest_string)?;

            let link_res = link(&manifest)?;

            if !link_res.status.success() {
                return Err(LinkerError::Other(String::from_utf8(link_res.stderr)?.into()).into());
            }

            println!(
                "Linking finished successfully! Binary output is available at: {}",
                manifest.output_path.display()
            );
        },
        CliCommand::Compile {
            path: compile_path,
            release: is_release,
            target_triple,
            llvm_flags,
            cpu_name,
            cpu_features,
        }
        | CliCommand::Run {
            path: compile_path,
            release: is_release,
            target_triple,
            llvm_flags,
            cpu_name,
            cpu_features,
        } => {
            let path = if let Some(path) = compile_path.clone() {
                path
            }
            else {
                current_working_dir
            };

            // Check for the main source file
            println!("Reading Files...");

            // Read config file
            let config_file = fs::read_to_string(format!("{}/config.toml", path.display()))
                .map_err(|_| ApplicationError::ConfigNotFound(path.clone()))?;

            let compiler_config = toml::from_str::<ProjectConfig>(&config_file)
                .map_err(ApplicationError::ConfigError)?;

            if !compiler_config.is_library && compiler_config.features.is_some() {
                println!(
                    "WARNING: Project `{}({})` is not a library, but has features. Features {:?} will be ignored.",
                    compiler_config.name,
                    compiler_config.version,
                    compiler_config.features.clone().unwrap()
                );
            }

            let source_file = fs::read_to_string(format!("{}/src/main.f", path.display()))
                .map_err(|_| ApplicationError::CodeGenError(CodeGenError::NoMain.into()))?;

            let compiler_state =
                CompilerState::new(compiler_config.clone(), path.clone(), OrdSet::new());

            fs::create_dir_all(compiler_config.build_path)?;

            let target_ir_path = PathBuf::from(format!(
                "{}\\{}\\{}.ll",
                path.display(),
                compiler_state.config.build_path,
                compiler_config.name.clone()
            ));

            let target_o_path = PathBuf::from(format!(
                "{}\\{}\\{}.obj",
                path.display(),
                compiler_state.config.build_path,
                compiler_config.name.clone()
            ));

            // Make this not so specific later
            let build_path = PathBuf::from(format!(
                "{}\\{}\\{}.exe",
                path.display(),
                compiler_state.config.build_path,
                compiler_config.name.clone()
            ));

            let build_manifest_path = PathBuf::from(format!(
                "{}\\{}\\{}.manifest",
                path.display(),
                compiler_state.config.build_path,
                compiler_config.name.clone()
            ));

            let compiler_startup_instant = std::time::Instant::now();

            let build_manifest = compiler_state.compilation_process(
                &source_file,
                target_ir_path.clone(),
                target_o_path.clone(),
                build_path.clone(),
                is_release,
                compiler_config.is_library,
                &format!("{}\\src", path.display()),
                &llvm_flags,
                target_triple,
                cpu_name,
                cpu_features,
            )?;

            // Write build manifest to disc
            fs::write(build_manifest_path, toml::to_string(&build_manifest)?)?;

            println!("All build artifacts have been saved.");

            // Link automaticly
            let link_res = link(&build_manifest).map_err(anyhow::Error::from)?;

            if !link_res.status.success() {
                return Err(LinkerError::Other(String::from_utf8(link_res.stderr)?.into()).into());
            }

            println!(
                "Linking finished successfully! Binary output is available at: {}",
                build_path.display()
            );

            println!(
                "Building finished in {:.2?}.",
                compiler_startup_instant.elapsed()
            );

            if matches!(compiler_command.clone(), CliCommand::Run { .. }) {
                let args: Vec<String> = Vec::new();

                println!(
                    "Running `{} {}`",
                    build_path.display(),
                    /* Pass in the arguments inherited (TODO) */ args.join(" ")
                );

                let exit_status = build_manifest.run_build_output(path.clone(), args)?;

                if !exit_status.success() {
                    if let Some(exit_code) = exit_status.code() {
                        println!("Process failed with exit code: {exit_code}")
                    }
                    else {
                        println!("Process was interrupted")
                    }
                }
            }
        },
        CliCommand::Version => println!("Build version: {}", env!("CARGO_PKG_VERSION")),
        CliCommand::New { path } => {
            println!("Creating project folders...");
            let path_s = path.display();

            fs::create_dir_all(path_s.to_string()).map_err(ApplicationError::FileError)?;
            fs::create_dir(format!("{path_s}/out"))?;
            fs::create_dir(format!("{path_s}/deps"))?;
            fs::create_dir(format!("{path_s}/src"))?;

            fs::write(
                format!("{}/src/main.f", path_s),
                include_str!("../../../defaults/default_code.f"),
            )
            .map_err(ApplicationError::FileError)?;

            fs::write(
                format!("{}/config.toml", path_s),
                toml::to_string(&ProjectConfig::new_from_name(
                    path.file_name().unwrap().to_string_lossy().to_string(),
                ))?,
            )
            .map_err(ApplicationError::FileError)?;
        },
        CliCommand::Init { path } => {
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
                toml::to_string(&ProjectConfig::new_from_name(get_folder_name.to_string()))?,
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
        println!("{}. {}", idx + 1, command)
    }
}
