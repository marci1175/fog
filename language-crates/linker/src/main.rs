use std::{env, path::PathBuf};

use fog_linker::link_from_manifest;

pub fn main() -> Result<(), Box<dyn std::error::Error>>
{
    let args: Vec<String> = env::args().collect();

    match args.get(1) {
        Some(build_manifest_path) => {
            link_from_manifest(PathBuf::from(build_manifest_path.clone()))?;
        },
        None => {
            println!("Build manifest path must be passed in.");
        },
    }

    Ok(())
}
