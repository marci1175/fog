use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParserError {
    #[error("The function definition / signature is invalid.")]
    InvalidFunctionDefinition,
    #[error("The function is called with the wrong types of arguments.")]
    InvalidFunctionCallArguments,
}
