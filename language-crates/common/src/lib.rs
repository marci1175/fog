#![feature(f16)]
pub const DEFAULT_COMPILER_ADDRESS_SPACE_SIZE: u16 = 0;

pub mod codegen;
pub mod compiler;
pub mod dependency;
pub mod error;
pub mod imports;
pub mod linker;
pub mod parser;
pub mod tokenizer;
pub mod ty;
pub mod distributed_compiler;

pub use anyhow;
pub use clap;
pub use indexmap;
pub use inkwell;
pub use serde;
pub use strum;
pub use strum_macros;
pub use toml;
