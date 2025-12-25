use std::path::PathBuf;
use thiserror::Error;

use crate::{
    error::syntax::SyntaxError, parser::{ParsedToken, VariableReference}, tokenizer::Token, ty::{OrdMap, Type},
};

#[derive(Debug, Error)]
pub enum ParserError
{
    #[error("A function or import signature is invalid.")]
    InvalidSignatureDefinition,
    #[error("The function is called with the wrong types of arguments.")]
    InvalidFunctionCallArguments,
    #[error(
        "When importing a function, the ellpisis can only be used at the last place of the arguments."
    )]
    InvalidEllipsisLocation,
    #[error("Function has been called with the wrong amount of arguments.")]
    InvalidFunctionArgumentCount,
    #[error(
        "The function defined by fog must have a definate amount of arguments. The ellpisis can only be used when importing foreign functions."
    )]
    DeterminiateArgumentsFunction,
    #[error("Type `{0}` mismatches type `{1}`.")]
    TypeError(Type, Type),
    #[error("Source code contains a Syntax Error: {0}")]
    SyntaxError(SyntaxError),
    #[error("Variable `{0}` with type `{1}` mismatches `{2}`.")]
    VariableTypeMismatch(String, Type, Type),
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
        "[INTERNAL ERROR] A value could not be parsed because a desired type discriminant wasn't set, required for type checking something with a known type."
    )]
    InternalDesiredTypeMissing,
    #[error("[INTERNAL ERROR] Variable `{0}` has the inner type of `{1}` which is invalid.")]
    InternalTypeMismatch(VariableReference, Type),
    #[error("A function with name `{0}` has been imported already.")]
    DuplicateSignatureImports(String),
    #[error("The linked source file at `{0}` is inaccesible or is not a vaild Fog source file.")]
    LinkedSourceFileError(PathBuf),
    #[error(r#"Type `{1}` cannot be constructed from '{0}'."#)]
    InvalidTypeCast(String, Type),
    #[error("`{0:?}` cannot be parsed as a valid type.")]
    InvalidType(Vec<Token>),
    #[error("The type of literal `{0}` could not be guessed.")]
    ValueTypeUnknown(String),
    #[error("Floats cannot be created with a value of NaN.")]
    FloatIsNAN,
    #[error("Type `{0}` is non-indexable.")]
    TypeMismatchNonIndexable(Type),
    #[error("Array has type `{0:?}` as its initalizer type.")]
    InvalidArrayTypeDefinition(Vec<Token>),
    #[error(
        "A function must have its visibility explicitly set. Visibility options: `pub`, `publib`, `priv`."
    )]
    FunctionRequiresExplicitVisibility,
    #[error("Token `{0}` is not a valid compiler hint.")]
    InvalidCompilerHint(Token),
    #[error(
        "Function is only enabled when feature `{0:?}` is enabled, which is an invalid feature."
    )]
    InvalidFunctionFeature(Option<Token>),

    #[error(
        "Function requires feature `{0}` to be enabled but project only has features `{1:?}` enabled."
    )]
    InvalidFeatureRequirement(String, Vec<String>),
    #[error("Module path contains an invalid token: `{0}`.")]
    InvalidModulePathDefinition(Token),
    #[error("Imported function was not found in the dependencies: `{0:?}`.")]
    FunctionDependencyNotFound(Vec<String>),
    #[error("Literal contains a non-Utf8 compatible char.")]
    InvalidUtf8Literal,
    #[error("Number cannot be represented in 64bits. Please truncate numbers which are too large.")]
    NumberTooLarge,
    #[error("Expected literal value with type `{0:?}`, found `{1}`.")]
    InvalidValue(Option<Type>, ParsedToken),
    #[error("Enum variant `{0}` was not found in specified enum.")]
    EnumVariantNotFound(String),
}
