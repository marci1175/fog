use thiserror::Error;

use crate::app::type_system::TypeDiscriminants;

#[derive(Debug, Error)]
pub enum ParserError {
    #[error("The function definition / signature is invalid.")]
    InvalidFunctionDefinition,
    #[error("The function is called with the wrong types of arguments.")]
    InvalidFunctionCallArguments,
    #[error("Type `{0}` cannot be automaticly casted to type `{1}`.")]
    TypeError(TypeDiscriminants, TypeDiscriminants),
    #[error("Source code contains a Syntax Error.")]
    SyntaxError,
    #[error("Variable `{0}` with type `{1}` mismatches `{2}`.")]
    VariableTypeMismatch(String, TypeDiscriminants, TypeDiscriminants),
    #[error("The variable named `{0}` has not been found in the current scope.")]
    VariableNotFound(String),
    #[error("The following argument was not found in the argument list: `{0}`")]
    ArgumentError(String),
    #[error("Const definition `{0}` could not be casted to type `{1}`")]
    ConstTypeUndetermined(String, TypeDiscriminants),
    #[error(
        "[INTERNAL ERROR] A variable was not found in the scope when it should've been. This is not the same as `VariableNotFound`!"
    )]
    InternalVariableError,
    #[error("[INTERNAL ERROR] Tried to parse an incompatible `Token` into `MathematicalExpression`.")]
    InternalMathParsingError,
}
