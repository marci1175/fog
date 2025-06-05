use std::path::PathBuf;
use thiserror::Error;

use crate::app::type_system::type_system::TypeDiscriminant;

#[derive(Debug, Error)]
pub enum CodeGenError {
    #[error("Output path `{0}` is unavailable.")]
    InvalidOutPath(PathBuf),
    #[error("[INTERNAL ERROR] Function `{0}` was not found in the module at codegen.")]
    InternalFunctionNotFound(String),
    #[error("[INTERNAL ERROR] Function did not return anything when the returned type was {0}.")]
    InternalFunctionReturnedVoid(TypeDiscriminant),
    #[error("[INTERNAL ERROR] Variable `{0}` was not found in Variable map.")]
    InternalVariableNotFound(String),
    #[error("Type `{0}` mismatches type `{1}`.")]
    InternalVariableTypeMismatch(TypeDiscriminant, TypeDiscriminant),
    #[error("[INTERNAL ERROR] The automatic optimizer has failed after the code generation.")]
    InternalOptimisationPassFailed,
    #[error("[INTERNAL ERROR] Failed to get TargetTriple for host.")]
    FaliedToAcquireTargetTriple,
    #[error(
        "The main entrypoint to the binary is not found. If you want to create a library, configure `config.toml` accordingly."
    )]
    NoMain,
    #[error(
        "The main entrypoint to the binary is found, but the signature is invalid. No arguments should be taken and `I32` is returned."
    )]
    InvalidMain,
    #[error("[INTERNAL ERROR] A struct's field was not found at codegen.")]
    InternalStructFieldNotFound,
    #[error("[INTERNAL ERROR] A variable type mismatch has occurred.")]
    InternalTypeMismatch,
    #[error("[INTERNAL ERROR] A reference to an inexistent struct has been provided.")]
    InternalStructReference,
    #[error("Comparsions are not implemented for type `{0}`.")]
    ComparisonIncompatibility(TypeDiscriminant),
    #[error("Type `{0}` cannot be casted to type `{1}`.")]
    InvalidTypeCast(TypeDiscriminant, TypeDiscriminant),
}
