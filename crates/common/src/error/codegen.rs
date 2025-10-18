use std::path::PathBuf;
use thiserror::Error;

use crate::{parser::ParsedToken, ty::TypeDiscriminant};

#[derive(Debug, Error)]
pub enum CodeGenError
{
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
    #[error("A type mismatch has occurred at codegen. Type `{0}` mismatches type `{1}`.")]
    CodegenTypeMismatch(TypeDiscriminant, TypeDiscriminant),
    #[error("[INTERNAL ERROR] A reference to an inexistent struct has been provided.")]
    InternalInvalidStructReference,
    #[error("Comparsions are not implemented for type `{0}`.")]
    ComparisonIncompatibility(TypeDiscriminant),
    #[error("Type `{0}` cannot be casted to type `{1}`.")]
    InvalidTypeCast(TypeDiscriminant, TypeDiscriminant),
    #[error(
        "The if statement contains an invalid condition. The condition has to return a boolean value."
    )]
    InvalidIfCondition,
    #[error("Codegen has encountered a parsing error.")]
    InternalParsingError,
    #[error("The codegen encountered a missing or an invalid PreAllocation in `allocation_map`.")]
    InvalidPreAllocation,
    #[error("A `null` value is used in the mathematical expression.")]
    InvalidMathematicalValue,
    #[error(
        "A value or argument of type `Void` is invalid. `Void` is solely for defining function return types."
    )]
    InvalidVoidValue,
    #[error(
        "Control flow keyword used in a non-iteration environment. Flow control keywords can only be used in iterator bodies."
    )]
    InvalidControlFlowUsage,

    /// The first value is the length of the original array, the second is the length of the array it was initalized with.
    #[error("An array of length `{0}` was initalized with an array with the length of `{1}`.")]
    ArrayLengthMismatch(usize, usize),

    #[error("Cannot index into a list with type `{0}`.")]
    NonIndexType(TypeDiscriminant),
    #[error("Value `{0}` cannot be indexed with.")]
    InvalidIndexValue(ParsedToken),
    #[error("ParsedToken `{0}` is not a valid variable reference.")]
    InvalidVariableReference(ParsedToken),

    /// This error can only be returned when an error occured thorugh LLVM-SYS itself.
    #[error("An error has occured while generating LLVM-IR: `{0}`.")]
    LibraryLLVMMessage(String),
}
