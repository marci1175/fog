use std::{fmt::Display, fs, path::PathBuf};

use anyhow::Error;

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

#[derive(Debug, Clone, PartialEq)]
pub struct Spanned<T>
{
    inner: T,
    span: SpanInfo,
}

impl<T> Spanned<T>
{
    pub fn new(inner: T, span: SpanInfo) -> Self
    {
        Self { inner, span }
    }

    pub fn get_span(&self) -> &SpanInfo {
        &self.span
    }
    
    pub fn inner(&self) -> &T {
        &self.inner
    }

    pub fn raise_error<E>(&self, file: PathBuf, error: E) -> SpannedError<E> {
        return SpannedError { error, file, span: dbg!(self.span) };
    }
}

#[derive(Debug, Clone)]
pub struct SpannedError<E> {
    error: E,
    file: PathBuf,
    span: SpanInfo,
}

impl<E: ToString> Into<anyhow::Error> for SpannedError<E> {
    fn into(self) -> anyhow::Error {
        Error::msg(self.to_string())
    }
}

impl<E: ToString> Display for SpannedError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Lets display the whole error first
        let mut message = self.error.to_string();

        // Separate the error location from the error
        message.push('\n');

        // Read the file
        if let Ok(content) = fs::read_to_string(&self.file) {
            let mut lines = content.lines();

            // Lets print out the whole line(s) of the file where the error occured
            if self.span.char_end.line == self.span.char_start.line {
                // Fetch the relevant line
                let relevant_line = lines.nth(self.span.char_start.line - 1).unwrap_or_default();
                
                // Store the relevant line
                message.push_str(relevant_line);
                message.push('\n');

                // Push the appropriate amount of spaces before the `^`s.
                for _ in 0..self.span.char_start.column {
                    message.push(' ');
                }

                // Push the error indicators
                for _ in self.span.char_start.column..self.span.char_end.column - 1 {
                    message.push('^');
                }

                message.push('\n');
            }
            // Print out all the lines where the error was with the indicators in every line
            else {
                // These are the lines which need to be printed fully.
                let iterator_len = self.span.char_end.line - self.span.char_start.line;
                let mut relevant_lines = lines.skip(self.span.char_start.line - 1).take(iterator_len + 1);

                // The first line should be printed from the beginning of the span's column
                let first_line = relevant_lines.next().unwrap_or_default();

                // Store the first line
                message.push_str(first_line);
                message.push('\n');

                // Push the appropriate amount of spaces before the `^`s.
                for _ in 0..self.span.char_start.column {
                    message.push(' ');
                }

                // Push the error indicators
                for _ in self.span.char_start.column..first_line.len() {
                    message.push('^');
                }
                
                message.push('\n');

                // Iterate over all the lines pushing the error indicators too, but leave out the last line
                for (idx, line) in relevant_lines.enumerate() {
                    // Check if this is the last line 
                    if idx + 1 == iterator_len {
                        message.push_str(line);
                
                        // Push the error indicators
                        for _ in 0..self.span.char_end.column - 1 {
                            message.push('^');
                        }
    
                        message.push('\n');

                        continue;
                    }

                    message.push_str(line);
                    message.push('\n');
                    
                    for _ in 0..line.len() {
                        message.push('^');
                    }

                    message.push('\n');
                }
            }
        }
        else {
            message.push_str("Failed to access file.");
        }

        f.write_str(&message)
    }
}