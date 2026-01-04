mod cli;

use crate::cli::CliCommand;
use clap::Parser;
use common::{
    anyhow, clap,
    compiler::ProjectConfig,
    compression::{compress_bytes, zip_folder},
    dependency_manager::{DependencyUpload, DependencyUploadReply},
    error::{application::ApplicationError, codegen::CodeGenError, linker::LinkerError},
    linker::BuildManifest,
    reqwest::{self, StatusCode},
    rmp_serde, serde_json, tokio, toml,
    tracing::{info, warn},
    tracing_subscriber,
    ty::OrdSet,
};
use compiler::CompilerState;
use linker::link;
use tracing::Level;
use std::{env, fs, path::PathBuf};

#[derive(Parser, Debug)]
#[command(version, about, long_about)]
pub struct CompilerArgs
{
    #[command(subcommand)]
    command: CliCommand,
}

#[tokio::main]
async fn main() -> common::anyhow::Result<()>
{
    tracing_subscriber::fmt().with_max_level(Level::DEBUG).init();

    let current_working_dir = env::current_dir()?;

    let compiler_args = CompilerArgs::parse();
    let compiler_command = compiler_args.command;

    match compiler_command.clone() {
        CliCommand::Link { path } => {
            info!("Reading file on: `{}`", path.display());

            let manifest_string = fs::read_to_string(&path)?;

            let manifest = toml::from_str::<BuildManifest>(&manifest_string)?;

            let link_res = link(&manifest)?;

            if !link_res.status.success() {
                return Err(LinkerError::Other(String::from_utf8(link_res.stderr)?.into()).into());
            }

            info!(
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
            let root_path = if let Some(path) = compile_path.clone() {
                path
            }
            else {
                current_working_dir
            };

            // Check for the main source file
            info!("Reading Files...");

            let source_file = fs::read_to_string(format!("{}/src/main.f", root_path.display()))
                .map_err(|_| ApplicationError::CodeGenError(CodeGenError::NoMain.into()))?;

            let compiler_state = CompilerState::new(root_path.clone(), OrdSet::new())?;

            let compiler_config = compiler_state.config.clone();

            if !compiler_config.is_library && compiler_config.features.is_some() {
                warn!(
                    "WARNING: Project `{}({})` is not a library, but has features. Features {:?} will be ignored.",
                    compiler_config.name,
                    compiler_config.version,
                    compiler_config.features.clone().unwrap()
                );
            }

            fs::create_dir_all(compiler_config.build_path)?;

            let build_artifact_name = format!(
                "{}\\{}\\{}",
                root_path.display(),
                compiler_state.config.build_path,
                compiler_config.name.clone()
            );

            let target_ir_path = PathBuf::from(format!("{build_artifact_name}.ll"));

            let target_o_path = PathBuf::from(format!("{build_artifact_name}.obj"));

            let build_path = PathBuf::from(format!("{build_artifact_name}.exe"));

            let build_manifest_path = PathBuf::from(format!("{build_artifact_name}.manifest"));
            let build_path_clone = build_path.clone();
            let compiler_startup_instant = std::time::Instant::now();
            let root_path_clone = root_path.clone();

            let build_manifest = tokio::task::spawn_blocking(move || {
                compiler_state.compilation_process(
                    &source_file,
                    target_ir_path.clone(),
                    target_o_path.clone(),
                    build_path_clone.clone(),
                    is_release,
                    compiler_config.is_library,
                    &format!("{}\\src", root_path.display()),
                    &llvm_flags,
                    target_triple,
                    cpu_name,
                    cpu_features,
                )
            })
            .await??;

            // Write build manifest to disc
            fs::write(build_manifest_path, toml::to_string(&build_manifest)?)?;

            info!("All build artifacts have been saved.");

            // Link automaticly
            let link_res = link(&build_manifest).map_err(anyhow::Error::from)?;

            if !link_res.status.success() {
                return Err(LinkerError::Other(String::from_utf8(link_res.stderr)?.into()).into());
            }

            info!(
                "Linking finished successfully! Binary output is available at: {}",
                build_path.display()
            );

            info!(
                "Building finished in {:.2?}.",
                compiler_startup_instant.elapsed()
            );

            if matches!(compiler_command.clone(), CliCommand::Run { .. }) {
                let args: Vec<String> = Vec::new();

                info!(
                    "Running `{} {}`",
                    build_path.display(),
                    /* Pass in the arguments inherited (TODO) */ args.join(" ")
                );

                let exit_status = build_manifest.run_build_output(root_path_clone, args)?;

                if !exit_status.success() {
                    if let Some(exit_code) = exit_status.code() {
                        info!("Process failed with exit code: {exit_code}")
                    }
                    else {
                        info!("Process was interrupted")
                    }
                }
            }
        },
        CliCommand::Version => info!("Build version: {}", env!("CARGO_PKG_VERSION")),
        CliCommand::New { path } => {
            info!("Creating project folders...");
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

            info!("Successfully created project `{}`", project_cfg.name)
        },
        CliCommand::Init { path: _ } => {
            info!("Getting folder name...");

            let get_folder_name = current_working_dir
                .file_name()
                .unwrap_or_default()
                .to_string_lossy();

            info!("Creating project folders...");
            fs::create_dir(format!("{}/output", current_working_dir.display()))
                .map_err(ApplicationError::FileError)?;
            fs::create_dir(format!("{}/deps", current_working_dir.display()))
                .map_err(ApplicationError::FileError)?;
            fs::create_dir(format!("{}/src", current_working_dir.display()))
                .map_err(ApplicationError::FileError)?;

            info!("Creating main source file...");
            fs::write(
                format!("{}/src/main.f", current_working_dir.display()),
                include_str!("../../../defaults/default_code.f"),
            )?;

            info!("Creating config file...");
            fs::write(
                format!("{}/config.toml", current_working_dir.display()),
                toml::to_string(&ProjectConfig::new_from_name(get_folder_name.to_string()))?,
            )
            .map_err(ApplicationError::FileError)?;

            info!(
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

            info!("Resolving `{url}`...");

            let http_client = reqwest::Client::new();
            let request_reply = http_client.get(&url).send().await?;

            let response_code = request_reply.status();

            info!("Remote `{url}` responded with: `{}`", response_code);

            let zip = zip_folder(fs::read_dir(path)?, Some(compiler_config.build_path))?;

            let zipped_folder = zip.finish_into_readable()?;

            if response_code == StatusCode::OK {
                info!("Uploading dependency...");

                if let Some(_secret_key) = secret {}

                let dependency_instance = DependencyUpload::new(
                    compiler_config.name.clone(),
                    compiler_config.version.clone(),
                    author,
                    zipped_folder.into_inner().into_inner(),
                );

                let serialized_dep_upload = rmp_serde::to_vec(&dependency_instance)?;

                let compressed_body = compress_bytes(&serialized_dep_upload)?;

                info!("Sending dependency...");

                let publish_response_code = http_client
                    .post(format!("{url}/publish_dependency"))
                    .header("Content-Type", "application/octet-stream")
                    .body(compressed_body)
                    .send()
                    .await?;

                let response_code = publish_response_code.status();

                if response_code == StatusCode::INTERNAL_SERVER_ERROR {
                    let request_body = publish_response_code.text().await?;
                    info!("Received response `{response_code}` from server: {request_body}.");
                }
                else {
                    let reply = request_reply.text().await?;

                    let dep_reply = serde_json::from_str::<DependencyUploadReply>(&reply)?;

                    info!(
                        "Dependency `{}({})` has been successfully created. This secret token `{}` can be used to update this dependency later.",
                        compiler_config.name, compiler_config.version, dep_reply.secret_to_dep
                    );
                }
            }

            info!("Abandoning connection...");
        },
    }

    Ok(())
}
