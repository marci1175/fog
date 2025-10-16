use std::path::PathBuf;

use anyhow::Error;

use super::error::CliParseError;

pub fn parse_args(arg1: String, arg2: String) -> (CliCommand, PathBuf) {
    (
        CliCommand::try_from(arg1).unwrap_or(CliCommand::Help),
        PathBuf::from(arg2),
    )
}

#[derive(Debug, strum_macros::VariantArray, strum_macros::Display, strum_macros::EnumMessage)]
pub enum CliCommand {
    #[strum(message = "`c |release/r|` - Compile a file.")]
    Compile,
    #[strum(message = "`h` - Display this help screen.")]
    Help,
    #[strum(message = "`v` - Display the version of this build.")]
    Version,
    #[strum(message = "`n <path-to-folder>` - Create a new Fog project.")]
    New,
    #[strum(message = "`init` - Initialize a new Fog project.")]
    Init,
}

impl TryFrom<String> for CliCommand {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<CliCommand, Error> {
        match value.as_str() {
            "c" => Ok(Self::Compile),
            "h" => Ok(Self::Help),
            "v" => Ok(Self::Version),
            "n" => Ok(Self::New),
            "init" => Ok(Self::Init),

            _ => {
                println!("Invalid Argument: `{value}`");
                Err(CliParseError::InvalidArg(value).into())
            }
        }
    }
}
