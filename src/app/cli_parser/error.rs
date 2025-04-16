use thiserror::Error;

#[derive(Debug, Error)]
pub enum CliParseError {
    #[error("An invalid argument `{0}` has been provided.")]
    InvalidArg(String),
}
