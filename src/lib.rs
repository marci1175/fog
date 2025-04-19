pub mod app;

use app::{cli_parser::error::CliParseError, parser::error::ParserError};
use std::io::Error;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CompilerError {
    #[error("File could not be accessed: `{0}`")]
    FileError(Error),

    #[error("The following error occured while parsing: `{0}`")]
    ParsingError(ParserError),

    #[error("Could not parse cli: `{0}`")]
    CliParseError(CliParseError),
}
