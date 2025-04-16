use thiserror::Error;

use crate::app::type_system::TypeDiscriminants;

use super::types::Tokens;

#[derive(Debug, Error)]
pub enum ParserError {
    #[error("The function definition / signature is invalid.")]
    InvalidFunctionDefinition,
    #[error("The function is called with the wrong types of arguments.")]
    InvalidFunctionCallArguments,
    #[error("Type {0} and {1} cannot be called with {2}")]
    TypeError(TypeDiscriminants, TypeDiscriminants, Tokens),
    #[error("Source code contains a Syntax Error.")]
    SyntaxError,
}
