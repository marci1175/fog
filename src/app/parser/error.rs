use std::path::PathBuf;

use indexmap::IndexMap;
use thiserror::Error;

use crate::app::type_system::type_system::TypeDiscriminant;

use super::types::{FunctionSignature, Token};

#[derive(Debug, Error)]
pub enum ParserError {
    #[error("A function or import signature is invalid.")]
    InvalidSignatureDefinition,
    #[error("The function is called with the wrong types of arguments.")]
    InvalidFunctionCallArguments,
    #[error("Function has been called with the wrong amount of arguments.")]
    InvalidFunctionArgumentCount,
    #[error("Type `{0}` mismatches type `{1}`.")]
    TypeError(TypeDiscriminant, TypeDiscriminant),
    #[error("Source code contains a Syntax Error: {0}")]
    SyntaxError(SyntaxError),
    #[error("Variable `{0}` with type `{1}` mismatches `{2}`.")]
    VariableTypeMismatch(String, TypeDiscriminant, TypeDiscriminant),
    #[error("The variable named `{0}` has not been found in the current scope.")]
    VariableNotFound(String),
    #[error("The following argument was not found in the argument list: `{0}`.")]
    ArgumentError(String),
    #[error(
        "[INTERNAL ERROR] A variable was not found in the scope when it should've been. This is not the same as `VariableNotFound`!"
    )]
    InternalVariableError,
    #[error(
        "[INTERNAL ERROR] Tried to parse an incompatible `Token` into `MathematicalExpression`."
    )]
    InternalMathParsingError,
    #[error(
        "[INTERNAL ERROR] A value could not be parsed because a desired type discriminant wasn't set, required for type checking something with known type."
    )]
    InternalDesiredTypeMissing,
    #[error("A function with this name/signature has been imported already.")]
    DuplicateSignatureImports,
    #[error("The linked source file at `{0}` is inaccesible or is not a vaild Fog source file.")]
    LinkedSourceFileMissing(PathBuf),
    #[error(r#"Type `{1}` cannot be constructed from '{0}'."#)]
    InvalidTypeCast(String, TypeDiscriminant),
    #[error("The type of literal `{0}` could not be guessed.")]
    ValueTypeUnknown(String),
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
    #[error("A valid struct has not been found with the name of `{0}`")]
    InvalidStructName(String),
    #[error("Invalid Struct Definition.")]
    InvalidStructDefinition,
    #[error("Invalid Dot placement, variable cannot have fields.")]
    InvalidDotPlacement,
    #[error("Struct field `{0}` was not found in Struct `{0}`.")]
    StructFieldNotFound(String, (String, IndexMap<String, TypeDiscriminant>)),
    #[error("Invalid Struct field definition.")]
    InvalidStructFieldDefinition,
    #[error("Missing/Invalid Struct body definition.")]
    MissingStructBody,
    #[error("Invalid Struct field reference.")]
    InvalidStructFieldReference,
    #[error("Struct Extensions should be only placed on the top-most layer of code.")]
    InvalidStructExtensionPlacement,
    #[error("Token `{0}` cannot be used to comapre values.")]
    InvalidTokenComparisonUsage(Token),
    #[error(r#"The condition should be surrounded by parentheses. ie: `if (x > 3) {{}}`"#)]
    InvalidIfConditionDefinition,
}
