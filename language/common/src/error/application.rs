use std::{io::Error, path::PathBuf};
use thiserror::Error;

use crate::error::cliparser::CliParseError;

#[derive(Debug, Error)]
pub enum ApplicationError
{
    #[error("File could not be accessed: {0}")]
    FileError(Error),

    #[error("Configuration file could not be found at `{0}`.")]
    ConfigNotFound(PathBuf),

    #[error("The following error occured while parsing: {0}")]
    ParsingError(anyhow::Error),

    #[error("Could not parse cli: {0}")]
    CliParseError(CliParseError),

    #[error("Error occured while generating LLVM-IR: {0}")]
    CodeGenError(anyhow::Error),

    #[error("Invalid Config syntax: {0}")]
    ConfigError(toml::de::Error),

    #[error("An error occured while parsing the project's dependencies: {0}")]
    DependencyError(anyhow::Error),
}
