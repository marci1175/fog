use thiserror::Error;

use crate::{
    parser::FunctionSignature,
    tokenizer::Token,
    ty::{OrdMap, TypeDiscriminant},
};

#[derive(Debug, Error)]
pub enum SyntaxError {
    #[error(r#"An open '{{' has been left in the code."#)]
    LeftOpenBraces,
    #[error("An open '(' has been left in the code.")]
    LeftOpenParentheses,
    #[error("An open '<' has been left in the code.")]
    LeftOpenAngledBrackets,
    #[error("An open '[' has been left in the code.")]
    LeftOpenSquareBrackets,
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
    #[error("Casting to a type requires a `TypeDefinition` after the `As` keyword.")]
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
    StructFieldNotFound(String, (String, OrdMap<String, TypeDiscriminant>)),
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
    #[error("Loop bodies are defined via brackets surrounding the code we would like to repeat.")]
    InvalidLoopBody,
    #[error("Imported function must have their return type defined.")]
    ImportUnspecifiedReturnType,
    #[error("A comma has been left out when defining an array.")]
    MissingCommaAtArrayDef,
    #[error("The type `{0}` cannot be indexed with.")]
    InvalidIndex(TypeDiscriminant),
    #[error("Unparsable expression: `{0}`")]
    UnparsableExpression(String),
}
