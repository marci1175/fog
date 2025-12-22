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

pub mod compression;

/// Custom language types and type wrappers
pub mod ty;

pub mod dependency_manager;

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

/// Used for making HTTP requests to contact web servers when fetching or publishing dependencies
pub use reqwest;

/// Used for thread safe types
pub use parking_lot;

/// Reading environment variables from `.env` files
pub use dotenvy;

/// Basic time management types.
pub use chrono;

/// Used for uploading/handling dependencies.
pub use zip;

pub use crossbeam;
pub use dashmap;
pub use flate2;
pub use futures;
pub use rmp_serde;
pub use serde_json;
pub use tracing;
pub use tracing_subscriber;

#[cfg(feature = "dependency_manager")]
pub use axum;
#[cfg(feature = "dependency_manager")]
pub use base64;
#[cfg(feature = "dependency_manager")]
pub use diesel;
#[cfg(feature = "dependency_manager")]
pub use r2d2;
#[cfg(feature = "dependency_manager")]
pub use rand;

/// This macro can be used to check if two struct's definitons matches. This will not check field name match, only Type.
/// Types are only checked shallow, if a field uses a type from a different path this will raise an error.
#[macro_export]
macro_rules! assert_same_fields {
    ($A:ty, $B:ty, { $($field:ident),* $(,)? }) => {
        const _: fn() = || {
            $(
                {
                    // Type inference trick: if types differ, this can't unify.
                    trait SameType {}
                    impl<T> SameType for (T, T) {}

                    // If A.$field and B.$field differ, this impl breaks → compile_error!
                    fn _assert(a: &$A, b: &$B) {
                        let _: &dyn SameType = &(a.$field.clone(), b.$field.clone());
                    }
                }
            )*
        };
    };
}
