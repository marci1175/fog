use thiserror::Error;

use crate::codegen::ty::{Type, Value};

#[derive(Debug, Error)]
pub enum AnalyzerError
{
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
