use std::path::PathBuf;

use thiserror::Error;

use crate::app::type_system::type_system::TypeDiscriminants;

use super::types::{FunctionSignature, Token};

#[derive(Debug, Error)]
pub enum ParserError {
    #[error("The function definition / signature is invalid.")]
    InvalidFunctionDefinition,
    #[error("The function is called with the wrong types of arguments.")]
    InvalidFunctionCallArguments,
    #[error("Type `{0}` mismatches type `{1}`.")]
    TypeError(TypeDiscriminants, TypeDiscriminants),
    #[error("Source code contains a Syntax Error: {0}")]
    SyntaxError(SyntaxError),
    #[error("Variable `{0}` with type `{1}` mismatches `{2}`.")]
    VariableTypeMismatch(String, TypeDiscriminants, TypeDiscriminants),
    #[error("The variable named `{0}` has not been found in the current scope.")]
    VariableNotFound(String),
    #[error("The following argument was not found in the argument list: `{0}`.")]
    ArgumentError(String),
    #[error("Const definition `{0}` could not be casted to type `{1}`.")]
    ConstTypeUndetermined(String, TypeDiscriminants),
    #[error(
        "[INTERNAL ERROR] A variable was not found in the scope when it should've been. This is not the same as `VariableNotFound`!"
    )]
    InternalVariableError,
    #[error(
        "[INTERNAL ERROR] Tried to parse an incompatible `Token` into `MathematicalExpression`."
    )]
    InternalMathParsingError,
    #[error("A function with this name/signature has been imported already.")]
    DuplicateSignatureImports,
    #[error("The linked source file at `{0}` is inaccesible or is not a vaild Fog source file.")]
    LinkedSourceFileMissing(PathBuf),
}

#[derive(Debug, Error)]
pub enum SyntaxError {
    #[error(r#"An open '{{' has been left in the code."#)]
    OpenBraces,
    #[error("An open '(' has been left in the code.")]
    LeftOpenParentheses,
    #[error(r#"An open '"' has been left in the code."#)]
    OpenQuotes,
    #[error("The code contains a missing `;`.")]
    MissingLineBreak,
    #[error("The code contains generic syntax error, like an invalid signature of a statement.")]
    InvalidStatementDefinition,
    #[error("The code contains an invalid function definition.")]
    InvalidFunctionDefinition,
    #[error("An invalid mathematical expression is present in the code.")]
    InvalidMathematicalExpressionDefinition,
    #[error("An invalid `SetValue` definition is present for '{0}'.")]
    InvalidSetValueDefinition(String),
    #[error("Token `{0}` could not be interpreted as a Value.")]
    InvalidValue(Token),
    #[error("Casting to a type requires a TypeDefinition after the `As` keyword.")]
    AsRequiresTypeDef,
    #[error("Function requires a returned value.")]
    FunctionRequiresReturn,
    #[error(
        "Duplicate function definitions have been found with function `{0}`. Signature: `{1}`."
    )]
    DuplicateFunctions(String, FunctionSignature),
    #[error("The import's signature is invalid.")]
    InvalidImportDefinition,
    #[error("Invalid Function name definition.")]
    InvalidFunctionName,
    #[error("Invalid Struct name definition.")]
    InvalidStructName,
    #[error("Invalid Struct field definition.")]
    InvalidStructFieldDefinition,
    #[error("Struct Extensions should be only placed on the top-most layer of code.")]
    InvalidStructExtensionPlacement,
}
