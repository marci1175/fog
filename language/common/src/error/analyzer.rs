use thiserror::Error;

use crate::{
    codegen::ty::{Type, Value},
    parser::common::StatementVariant,
};

#[derive(Debug, Error)]
pub enum AnalyzerError
{
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
    UnsupportedValueByType(Value, Type),
    #[error(
        "[INTERNAL ERROR] Value with type `{0}` is not numerical. Check where the automatic type"
    )]
    InternalValueNotNumerical(Type),
    #[error("Item is not a function and cannot be called as one.")]
    ItemNotCallableAsFunction,
}
