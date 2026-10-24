use crate::tokenizer::Token;
use std::{
    fmt::{Debug, Display},
    fs,
    ops::Deref,
    path::PathBuf,
};

pub mod analyzer;
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

impl Display for SpanInfo
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.write_str(&format!(
            "From: {}:{}, To: {}:{}",
            self.char_start.line, self.char_start.column, self.char_end.line, self.char_end.column
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, PartialOrd, Copy)]
pub struct CharPosition
{
    pub line: u32,
    pub column: u32,
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
    pub fn new(line: u32, column: u32) -> Self
    {
        Self { line, column }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Spanned<T>
{
    pub inner: T,
    pub span: SpanInfo,
}

impl PartialEq<Token> for Spanned<Token>
{
    fn eq(&self, other: &Token) -> bool
    {
        &self.inner == other
    }
}

impl<T> Spanned<T>
{
    pub fn new(inner: T, span: SpanInfo) -> Self
    {
        Self { inner, span }
    }

    pub fn get_span(&self) -> &SpanInfo
    {
        &self.span
    }

    pub fn rewrap_value<T2>(&self, inner: T2) -> Spanned<T2>
    {
        Spanned {
            inner,
            span: self.span,
        }
    }

    pub fn get_inner(&self) -> &T
    {
        &self.inner
    }

    pub fn raise_error<E>(&self, file: PathBuf, error: E) -> SpannedError<E>
    {
        SpannedError {
            error,
            file,
            span: self.span,
        }
    }

    pub fn inner_owned(self) -> T
    {
        self.inner
    }
}

impl<T> Deref for Spanned<T>
{
    type Target = T;

    fn deref(&self) -> &Self::Target
    {
        self.get_inner()
    }
}

#[derive(Debug, Clone)]
pub struct SpannedError<E>
{
    pub error: E,
    pub file: PathBuf,
    pub span: SpanInfo,
}

impl<E> Display for SpannedError<E>
where
    E: ToString,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        // Create message buffer
        let mut message = String::new();

        // Display the location first
        message.push_str(&format!(
            "{}:{}\n",
            self.file.display(),
            self.span.char_start.line
        ));

        // Separate the error location from the error
        message.push('\n');

        // Read the file
        if let Ok(content) = fs::read_to_string(&self.file) {
            let mut lines = content.lines();

            // Lets print out the whole line(s) of the file where the error occured
            if self.span.char_end.line == self.span.char_start.line {
                // Fetch the relevant line
                let relevant_line = lines
                    .nth((self.span.char_start.line - 1) as usize)
                    .unwrap_or_default();

                // Store the relevant line
                message.push_str(relevant_line);
                message.push('\n');

                // Push the appropriate amount of spaces before the `^`s.
                for _ in 0..self.span.char_start.column {
                    message.push(' ');
                }

                // Push the error indicators
                for _ in self.span.char_start.column..self.span.char_end.column {
                    message.push('^');
                }

                message.push('\n');
            }
            // Print out all the lines where the error was with the indicators in every line
            else {
                // These are the lines which need to be printed fully.
                let iterator_len = self.span.char_end.line - self.span.char_start.line;
                let mut relevant_lines = lines
                    .skip((self.span.char_start.line - 1) as usize)
                    .take((iterator_len + 1) as usize);

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
                for _ in self.span.char_start.column as usize..first_line.len() {
                    message.push('^');
                }

                message.push('\n');

                // Iterate over all the lines pushing the error indicators too, but leave out the last line
                for (idx, line) in relevant_lines.enumerate() {
                    // Check if this is the last line
                    if idx + 1 == iterator_len as usize {
                        message.push_str(line);
                        message.push('\n');

                        // Push the error indicators
                        for _ in 0..self.span.char_end.column {
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

        // Separate actual error from the file content
        message.push('\n');

        message.push_str(&self.error.to_string());

        message.push('\n');

        f.write_str(&message)
    }
}

impl<E> std::error::Error for SpannedError<E> where E: ToString + Debug {}
