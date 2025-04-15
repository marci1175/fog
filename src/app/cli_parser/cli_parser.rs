use anyhow::Error;
use thiserror::Error;

pub fn parse_args(arg1: String, arg2: String) -> anyhow::Result<(CliCommand, String)> {
    Ok((
        CliCommand::try_from(arg1).unwrap_or_else(|_| CliCommand::Help),
        arg2,
    ))
}

#[derive(Debug, Error)]
pub enum CliParseError {
    #[error("An invalid argument `{0}` has been provided.")]
    InvalidArg(String),
}

#[derive(Debug, strum_macros::VariantArray, strum_macros::Display, strum_macros::EnumMessage)]
pub enum CliCommand {
    #[strum(message = "`c <path-to-file>` - Compile a file")]
    Compile,
    #[strum(message = "`h` - Display this help screen")]
    Help,
}

impl TryFrom<String> for CliCommand {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<CliCommand, Error> {
        match value.as_str() {
            "c" => Ok(Self::Compile),
            "h" => Ok(Self::Help),

            _ => Err(CliParseError::InvalidArg(value).into()),
        }
    }
}
