use std::path::PathBuf;
use thiserror::Error;

use crate::{
    error::syntax::SyntaxError,
    parser::{common::ParsedToken, variable::VariableReference},
    tokenizer::Token,
    ty::Type,
};

#[derive(Debug, Error)]
pub enum ParserError
{
    #[error(
        "[INTERNAL ERROR] Function with a receiver (`this`) argument was not added to the function signature."
    )]
    InternalFunctionReceiverArgMissing,
    #[error(
        "Function `{0}` has been defined multiple times. Function overloading is not supported."
    )]
    FunctionRedefinition(String),
    #[error("Implementation bodies can only contain function implementations.")]
    InvalidImplItem,
    #[error("Trait definition bodies can only contain function signatures.")]
    InvalidTraitItem,
    #[error(
        "Functions [{0:?}] of trait `{1}` either wasn't defined or weren't correctly implemented."
    )]
    InvalidTraitImplementation(Vec<String>, String),
    #[error("Custom item `{0}` was not found in the current scope. (Check typos)")]
    CustomItemNotFound(String),
    #[error(
        "External function cannot return type interfaces, only a concrete type. (ie. int or a struct)"
    )]
    ExternalFunctionsReturnConcreteTypes,
    #[error("Trait `{0}` cannot be converted into a value.")]
    TraitNotObject(String),
    #[error("A function or import signature is invalid.")]
    InvalidSignatureDefinition,
    #[error(
        "The function is called with the wrong type of arguments. Functions with no arguments still need to be called with `()`. ie.: `foo();`"
    )]
    InvalidFunctionCallArguments,
    #[error(
        "When importing a function, the ellpisis can only be used at the last place of the arguments."
    )]
    InvalidEllipsisPosition,
    #[error(
        "When using a receiver as an argument (The `this` keyword) it must only be used at the very first position of the arguments."
    )]
    InvalidReceiverPosition,
    #[error(
        "When using a receiver as an argument (The `this` keyword) it must only be used when implementing a function for a struct."
    )]
    InvalidReceiverUsage,
    #[error("Function has been called with the wrong amount of arguments.")]
    InvalidFunctionArgumentCount,
    #[error(
        "The function defined by fog must have a definate amount of arguments. The ellpisis can only be used when importing foreign functions."
    )]
    DeterminiateArgumentsFunction,
    #[error("Type `{0}` mismatches type `{1}` or does not implement appropriate  trait.")]
    TypeMismatch(Type, Type),
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
    #[error(
        "Type `{0}` does not contain any fields and may not be accessed via any field. (Only structs have fields)"
    )]
    TypeWithoutFields(Type),
    #[error("Parser has encountered invalid END OF FILE.")]
    EOF,
    #[error("Variable `{0}` must have a default value of type `{1}`.")]
    MissingVariableValue(String, Type),
    #[error(
        "Function name cannot start with `__internal` as it is reserved for internal language functions."
    )]
    FunctionNameReserved,
}
