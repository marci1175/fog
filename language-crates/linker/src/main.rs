use std::{env, fs, process::Command};

use fog_common::{linker::BuildManifest, toml};

pub fn main() -> Result<(), Box<dyn std::error::Error>>
{
    let args: Vec<String> = env::args().collect();

    println!("Fog linker [Build version: {}]", env!("CARGO_PKG_VERSION"));

    match Command::new("clang").arg("--version").output() {
        Ok(output) => {
            println!("[Clang found]");
            println!("{}", String::from_utf8_lossy(&output.stdout));
        },
        Err(e) => {
            println!(
                "Clang must be added to path inorder to use this linker. Clang not found in PATH: {}",
                e
            );
        },
    }

    match args.get(1) {
        Some(arg) => {
            println!("[Build manifest path]: {}", arg);

            // Check if the file exists and read into a string
            let build_manifest = fs::read_to_string(arg)?;

            let build_manifest = toml::from_str::<BuildManifest>(&build_manifest)?;

            let mut args: Vec<String> = Vec::new();

            args.extend(
                build_manifest
                    .build_output_paths
                    .iter()
                    .map(|p| p.display().to_string()),
            );
            args.push("-o".to_string());
            args.push(build_manifest.output_path.display().to_string());

            println!("Linking...");
            Command::new("clang").args(args.iter()).output()?;

            println!(
                "Linking finished output located at: {}",
                build_manifest.output_path.display()
            );
        },
        None => {
            println!("Build manifest path must be passed in.")
        },
    }

    Ok(())
}
