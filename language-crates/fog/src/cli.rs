use std::path::PathBuf;

use fog_common::{anyhow, error::cliparser::CliParseError, strum, strum_macros};

pub fn parse_args(arg1: String, arg2: String) -> (CliCommand, PathBuf)
{
    (
        CliCommand::try_from(arg1).unwrap_or(CliCommand::Help),
        PathBuf::from(arg2),
    )
}

#[derive(Debug, strum::VariantArray, strum_macros::Display, strum_macros::EnumMessage)]
pub enum CliCommand
{
    #[strum(message = "`h` - Display this help screen.")]
    Help,
    #[strum(message = "`v` - Display the version of this build.")]
    Version,
    #[strum(message = "`l <path-to-manifest>` - Links an executable based on a `.manifest` file.")]
    Link,
    #[strum(message = "`c |release/r|` - Compile a file.")]
    Compile,
    #[strum(message = "`i` - Initialize a new Fog project.")]
    Init,
    #[strum(message = "`n <path-to-folder>` - Create a new Fog project.")]
    New,
}

impl TryFrom<String> for CliCommand
{
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<CliCommand, Self::Error>
    {
        match value.as_str() {
            "c" | "compile" => Ok(Self::Compile),
            "h" | "help" => Ok(Self::Help),
            "v" | "version" => Ok(Self::Version),
            "n" | "new" => Ok(Self::New),
            "i" | "init" => Ok(Self::Init),
            "l" | "link" => Ok(Self::Link),

            _ => {
                println!("Invalid Argument: `{value}`");
                Err(CliParseError::InvalidArg(value).into())
            },
        }
    }
}
