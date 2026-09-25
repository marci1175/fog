use thiserror::Error;

use crate::codegen::ty::{Type, Value};

#[derive(Debug, Error)]
pub enum AnalyzerError
{
    #[error("Value `{0}` cannot be automatically casted to type `{1}` without data loss. Consider casting with the `as` keyword.")]
    InvalidAutomaticTypeSizeResolve(Value, Type),
    #[error("[INTERNAL ERROR] Value with type `{0}` is not numerical. Check where the automatic type")]
    InternalValueNotNumerical(Type),
}
