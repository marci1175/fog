use std::{env, path::PathBuf};

use linker::{host_information, link_from_manifest};

pub fn main() -> Result<(), Box<dyn std::error::Error>>
{   
    let args: Vec<String> = env::args().collect();

    match args.get(1) {
        Some(argument) => {
            match argument.as_str() {
                "h" | "help" => {
                    host_information()?;
                }
                _ => {
                    link_from_manifest(PathBuf::from(argument.clone()))?;
                }
            }
        },
        None => {
            println!("Build manifest path must be passed in. (`help` to see host information.)");
        },
    }

    Ok(())
}
