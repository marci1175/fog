pub mod application;
pub mod cliparser;
pub mod codegen;
pub mod dependency;
pub mod dependency_manager;
pub mod linker;
pub mod parser;
pub mod syntax;

#[derive(Clone, Debug)]
pub struct ErrorWrapper<T>
{
    pub error: T,
    pub debug_information: SpanInfo,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Copy)]

pub struct SpanInfo
{
    pub char_start: CharPosition,
    // The char end position is inclusive.
    pub char_end: CharPosition,
}

impl SpanInfo
{
    pub fn new(char_start: CharPosition, char_end: CharPosition) -> Self
    {
        Self {
            char_start,
            char_end,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, PartialOrd, Copy)]
pub struct CharPosition
{
    pub line: usize,
    pub column: usize,
}

impl Ord for CharPosition
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering
    {
        self.line
            .cmp(&other.line)
            .then(self.column.cmp(&other.column))
    }
}

impl CharPosition
{
    pub fn new(line: usize, column: usize) -> Self
    {
        Self { line, column }
    }
}
