mod cli;

use crate::cli::CliCommand;
use clap::Parser;
use common::{
    anyhow, clap,
    compiler::ProjectConfig,
    dependency_manager::{DependencyUpload, DependencyUploadReply, write_folder_items},
    error::{
        application::ApplicationError, codegen::CodeGenError,
        linker::LinkerError,
    },
    flate2::{Compression, write::ZlibEncoder},
    linker::BuildManifest,
    reqwest::{StatusCode, blocking::Client},
    rmp_serde, serde_json, toml,
    ty::OrdSet,
    zip::{ZipWriter, write::SimpleFileOptions},
};
use compiler::CompilerState;
use linker::link;
use std::{
    env, fs,
    io::{Cursor, Write},
    path::PathBuf,
};

#[derive(Parser, Debug)]
#[command(version, about, long_about)]
pub struct CompilerArgs
{
    #[command(subcommand)]
    command: CliCommand,
}

fn main() -> common::anyhow::Result<()>
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

            let project_cfg = ProjectConfig::new_from_name(
                path.file_name().unwrap().to_string_lossy().to_string(),
            );

            fs::write(
                format!("{}/config.toml", path_s),
                toml::to_string(&project_cfg)?,
            )
            .map_err(ApplicationError::FileError)?;

            println!("Successfully created project `{}`", project_cfg.name)
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
        CliCommand::Publish {
            url,
            author,
            secret,
            path,
        } => {
            let path = if let Some(path) = path.clone() {
                path
            }
            else {
                current_working_dir
            };

            // Read config file
            let config_file = fs::read_to_string(format!("{}/config.toml", path.display()))
                .map_err(|_| ApplicationError::ConfigNotFound(path.clone()))?;

            let compiler_config = toml::from_str::<ProjectConfig>(&config_file)
                .map_err(ApplicationError::ConfigError)?;

            println!("Contacting `{url}`...");

            let http_client = Client::new();
            let reply = http_client.get(&url).send()?.status();

            println!("Remote `{url}` responded with: `{}`", reply);

            let mut source_files = Cursor::new(Vec::new());

            let mut zip = ZipWriter::new(&mut source_files);

            let read_dir = fs::read_dir(&path)?;

            write_folder_items(
                &mut zip,
                read_dir,
                PathBuf::new(),
                SimpleFileOptions::default(),
                Some(compiler_config.build_path),
            )?;

            let readable = zip.finish_into_readable()?;

            if let Some(secret_key) = secret {
            }
            else if reply == StatusCode::OK {
                println!("Uploading dependency...");

                let dependency_instance = DependencyUpload::new(
                    compiler_config.name.clone(),
                    compiler_config.version.clone(),
                    author,
                    source_files.into_inner(),
                );

                let serialized_dep_upload = rmp_serde::to_vec(&dependency_instance)?;

                let mut encoder = ZlibEncoder::new(Vec::new(), Compression::fast());

                encoder.write_all(&serialized_dep_upload)?;

                let compressed_body = encoder.finish()?;

                println!("Sending dependency...");

                let publish_reply = http_client
                    .post(format!("{url}/publish_dependency"))
                    .header("Content-Type", "application/octet-stream")
                    .body(compressed_body)
                    .send()?;

                println!(
                    "Received response `{}` from server.",
                    publish_reply.status()
                );

                let reply = publish_reply.text()?;

                let dep_reply = serde_json::from_str::<DependencyUploadReply>(&reply)?;

                println!(
                    "Dependency `{}({})` has been successfully created. This secret token `{}` can be used to update this dependency later.",
                    compiler_config.name, compiler_config.version, dep_reply.secret_to_dep
                );
            }

            println!("Abandoning connection...");
        },
    }

    Ok(())
}
