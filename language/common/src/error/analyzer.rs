use strum::IntoStaticStr;
use thiserror::Error;

use crate::{
    codegen::ty::{Type, Value},
    parser::StatementVariant,
};

#[derive(Debug, Error, IntoStaticStr)]
pub enum AnalyzerError
{
    #[error(
        "[INTERNAL ERROR] A function `{0}`'s definition has been found but its signature is not available during analysis."
    )]
    InternalFunctionSignatureNotFoundDefinitionFound(String),
    #[error("Different code paths return different types.")]
    HeterogeneousReturnType,
    #[error("Function returns a value of type `{0}`, signature requires `{1}`.")]
    MismatchingReturnType(Box<Type>, Box<Type>),
    #[error(
        "Different items of an array have different types. First item's type is `{0}` but the this item's type is `{1}`"
    )]
    HeterogeneousArrayItemType(Box<Type>, Box<Type>),
    #[error("Not all conditional branches return a value when needed.")]
    InsufficientReturnPathCoverage,
    #[error(
        "A function with a return type other than `void` requires a value returned. A return statement is expected."
    )]
    FunctionRequiresReturn,
    #[error(
        "The function has {0} arguments. This argument is the {1}. passed in the function call."
    )]
    FunctionArgumentCountInvalid(usize, usize),
    #[error(
        "The argument entered in the function call is invalid. The function's signature does not contain such named argument."
    )]
    UnknownFunctionNamedArgument,
    #[error("[INTERNAL ERROR] Identifier `{0}` was not resolved, the analyzer cannot continue.")]
    InternalIdentUnresolved(StatementVariant),
    #[error("The module(s) containing the imported item was not found.")]
    PathModuleNotFound,
    #[error(
        "Identifier referenced could not be resolved in current context. Check spelling of the identifier."
    )]
    IdentifierNotFound,
    #[error("The value `{0}` cannot be fit into the destination type `{1}`.")]
    UnsupportedValueByType(Box<Value>, Box<Type>),
    #[error(
        "[INTERNAL ERROR] Value with type `{0}` is not numerical. Check where the automatic type"
    )]
    InternalValueNotNumerical(Box<Type>),
    #[error("Item is not a function and cannot be called as one.")]
    ItemNotCallableAsFunction,
}
