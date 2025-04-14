use anyhow::Error;
use thiserror::Error;

pub fn parse_args(arg1: String, arg2: String) -> anyhow::Result<(CliCommand, String)> {
    Ok((
        CliCommand::try_from(arg1.chars().nth(0).unwrap()).unwrap_or_else(|_| CliCommand::Help),
        arg2,
    ))
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("An invalid argument `{0}` has been provided.")]
    InvalidArg(char),
}

#[derive(Debug)]
pub enum CliCommand {
    Compile,
    Help,
}

impl TryFrom<char> for CliCommand {
    type Error = anyhow::Error;

    fn try_from(value: char) -> Result<CliCommand, Error> {
        match value {
            'c' => Ok(Self::Compile),
            'h' => Ok(Self::Help),

            _ => Err(ParseError::InvalidArg(value).into()),
        }
    }
}
