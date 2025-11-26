use std::ops::Range;

pub mod application;
pub mod cliparser;
pub mod codegen;
pub mod dependency;
pub mod linker;
pub mod parser;
pub mod syntax;

#[derive(Clone, Debug)]
pub struct ErrorWrapper<T>
{
    pub error: T,
    pub debug_information: DebugInformation,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]

pub struct DebugInformation
{
    pub char_range: Vec<Range<usize>>,
    pub lines: Range<usize>,
}
