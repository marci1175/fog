#![feature(iterator_try_collect)]

use fog_common::{error::linker::LinkerError, linker::BuildManifest, toml};
use std::{
    env, fs,
    path::PathBuf,
    process::{Command, Output},
};

pub fn link_from_manifest(build_manifest_path: PathBuf) -> Result<(), LinkerError>
{
    println!("Fog linker [Build version: {}]", env!("CARGO_PKG_VERSION"));

    match Command::new("clang").arg("--version").output() {
        Ok(output) => {
            println!("[Clang found]");
            println!("{}", String::from_utf8_lossy(&output.stdout));
        },
        Err(_e) => {
            return Err(LinkerError::ClangNotFound);
        },
    }

    println!("[Build manifest path]: {}", build_manifest_path.display());

    // Check if the file exists and read into a string
    let build_manifest =
        fs::read_to_string(build_manifest_path).map_err(|err| LinkerError::Other(Box::new(err)))?;

    let build_manifest = toml::from_str::<BuildManifest>(&build_manifest)
        .map_err(|_| LinkerError::InvalidManifestFormat)?;

    link(&build_manifest)?;

    println!(
        "Linking finished output located at: {}",
        build_manifest.output_path.display()
    );

    Ok(())
}

pub fn link(build_manifest: &BuildManifest) -> Result<Output, LinkerError>
{
    let mut args: Vec<String> = Vec::new();

    args.extend(
        build_manifest
            .build_output_paths
            .iter()
            .map(|p| p.display().to_string()),
    );

    args.extend(
        build_manifest
            .additional_linking_material
            .iter()
            .map(|p| {
                if let Ok(path) = fs::canonicalize(p) {
                    return Ok(path.display().to_string());
                }
                else {
                    return Err(LinkerError::AdditionalLinkingMaterialNotFound(p.clone()));
                }
            })
            .try_collect::<Vec<String>>()?,
    );

    args.push("-o".to_string());
    args.push(build_manifest.output_path.display().to_string());

    println!("Linking...");

    let clang_out = Command::new("clang")
        .args(args.iter())
        .output()
        .map_err(|err| LinkerError::ClangError(Box::new(err)))?;

    Ok(clang_out)
}
