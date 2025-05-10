use std::path::PathBuf;

use thiserror::Error;

use crate::app::type_system::type_system::TypeDiscriminants;

#[derive(Debug, Error)]
pub enum CodeGenError {
    #[error("Output path `{0}` is unavailable.")]
    InvalidOutPath(PathBuf),
    #[error("[INTERNAL ERROR] Function was not found in the module at codegen.")]
    InternalFunctionNotFound(String),
    #[error("[INTERNAL ERROR] Function did not return anything when the returned type was {0}.")]
    InternalFunctionReturnedVoid(TypeDiscriminants),
}
