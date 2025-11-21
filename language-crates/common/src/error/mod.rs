use std::ops::Range;

pub mod application;
pub mod cliparser;
pub mod codegen;
pub mod dependency;
pub mod linker;
pub mod parser;
pub mod syntax;

pub struct ErrorWrapper<T> {
    pub error: T,
    pub debug_information: DebugInformation,
}

pub struct DebugInformation {
    pub char_range: Range<usize>,
}