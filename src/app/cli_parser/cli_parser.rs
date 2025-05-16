use anyhow::Error;

use super::error::CliParseError;

pub fn parse_args(arg1: String, arg2: String) -> (CliCommand, String) {
    (CliCommand::try_from(arg1).unwrap_or(CliCommand::Help), arg2)
}

#[derive(Debug, strum_macros::VariantArray, strum_macros::Display, strum_macros::EnumMessage)]
pub enum CliCommand {
    #[strum(message = "`c <path-to-file> <path-to-out>` - Compile a file")]
    Compile,
    #[strum(message = "`h` - Display this help screen")]
    Help,
    #[strum(message = "`v` - Display the version of this build.")]
    Version,
}

impl TryFrom<String> for CliCommand {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<CliCommand, Error> {
        match value.as_str() {
            "c" => Ok(Self::Compile),
            "h" => Ok(Self::Help),
            "v" => Ok(Self::Version),

            _ => Err(CliParseError::InvalidArg(value).into()),
        }
    }
}
