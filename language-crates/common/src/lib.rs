#![feature(f16)]
pub const DEFAULT_COMPILER_ADDRESS_SPACE_SIZE: u16 = 0;

/// LLVM-IR generation code with inkwell
pub mod codegen;

/// Main fog compiler frontend
pub mod compiler;

/// Types for handling project dependencies
pub mod dependency;

/// Types and helper functions for the FDCN tool
pub mod distributed_compiler;

/// Error type definitions which can occure in the compiler
pub mod error;

/// Used for hanlidng imports from dependencies and external files
pub mod imports;

/// Linking interface with clang (Can be compiled as a standalone tool)
pub mod linker;

/// Parsing the tokens produced by the tokenizer
pub mod parser;

/// Tokenizer
pub mod tokenizer;

/// Custom language types and type wrappers
pub mod ty;

/// Used for handling errors in the fog toolset
pub use anyhow;

/// Used for Cli parsing
pub use clap;

/// Used for storing data in a HashMap with order
pub use indexmap;

/// Used for LLVM-IR generation
pub use inkwell;

/// Used for Serializing and Deserializing config files
pub use serde;

/// This is used to handle enums
pub use strum;
pub use strum_macros;

/// Config file parsing / handling
pub use toml;

/// Used for communicating with the dependency manager server and to create the FDCN
pub use tokio;