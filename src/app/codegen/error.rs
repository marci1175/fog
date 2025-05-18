use std::path::PathBuf;

use thiserror::Error;

use crate::app::type_system::type_system::TypeDiscriminants;

#[derive(Debug, Error)]
pub enum CodeGenError {
    #[error("Output path `{0}` is unavailable.")]
    InvalidOutPath(PathBuf),
    #[error("[INTERNAL ERROR] Function `{0}` was not found in the module at codegen.")]
    InternalFunctionNotFound(String),
    #[error("[INTERNAL ERROR] Function did not return anything when the returned type was {0}.")]
    InternalFunctionReturnedVoid(TypeDiscriminants),
    #[error("[INTERNAL ERROR] Variable `{0}` was not found in Variable map.")]
    InternalVariableNotFound(String),
    #[error("[INTERNAL ERROR] Variable {0} mismatches variable `{1}`'s type.")]
    InternalVariableTypeMismatch(String, String),
    #[error("[INTERNAL ERROR] The automatic optimiser has failed after the code generation.")]
    InternalOptimisationPassFailed,
    #[error("[INTERNAL ERROR] Failed to get TargetTriple for host.")]
    FaliedToAcquireTargetTriple,
}
