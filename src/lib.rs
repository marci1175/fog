mod app;

use std::io::Error;
use app::cli_parser::cli_parser::CliParseError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CompilerError {
    #[error("File could not be accessed: `{0}`")]
    FileError(Error),

    // ParsingError(),
    
    #[error("Could not parse cli: `{0}`")]
    CliParseError(CliParseError),
}