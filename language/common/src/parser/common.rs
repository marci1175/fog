use anyhow::Result;
use strum_macros::Display;

use crate::{
    codegen::{CustomItem, DerefMode, FunctionArgumentIdentifier, If, Order},
    error::{SpanInfo, Spanned, parser::ParserError, syntax::SyntaxError},
    parser::{
        function::{
            CompilerInstruction, FunctionArguments, FunctionDefinition, FunctionSignature, PathMap,
        },
        value::MathematicalSymbol,
        variable::{ControlFlowType, UniqueId, VariableReference},
    },
    tokenizer::Token,
    ty::{OrdMap, OrdSet, Type, Value},
};

#[derive(Debug, Clone, Eq, Hash)]
/// A ParsedTokenInstance is ParsedToken with additional information. DebugInformation will not affect comparisons. (Check PartialEq trait implementation)
pub struct ParsedTokenInstance
{
    pub inner: ParsedToken,
    pub debug_information: SpanInfo,
}

impl PartialEq for ParsedTokenInstance
{
    fn eq(&self, other: &Self) -> bool
    {
        self.inner == other.inner
    }
}

impl PartialEq<ParsedToken> for ParsedTokenInstance
{
    fn eq(&self, other: &ParsedToken) -> bool
    {
        &self.inner == other
    }
}

/// Helper trait for types lookingto implement a buffer-like stream.
pub trait Streamable<T>
{
    /// Peeks the nth next token from the stream.
    /// Since nth is an [`isize`] it can peek both backwards and forwards.
    fn peek(&self, nth: isize) -> Option<&T>;
    fn peek_next(&self) -> Option<&T>;

    /// Returns the next item from the stream.
    fn consume(&mut self) -> Option<&T>;

    /// This function only returns the item, if it equals the discriminant. If it does not it returns the error provided.
    fn try_consume_match<E: Clone, D>(&mut self, error: E, discriminant: &D) -> Result<&T, E>
    where
        T: PartialEq<D>;

    /// The fetching should be non-inclusive.
    /// The function should return the `nth` next tokens.
    fn consume_bulk(&mut self, nth: usize) -> Option<&[T]>;

    fn decrement_cursor(&mut self, num: usize);

    /// Peeks the rest of the stream.
    fn peek_remainder(&self) -> Option<&[T]>;

    /// Returns the last consumed item of the stream.
    fn get_last_consumed(&self) -> Option<&T>;
}

/// Stores the index of the cursor in the time this checkpoint was captured.
pub struct StreamCheckpoint
{
    idx: usize,
}

#[derive(Debug, Clone, Default)]
pub struct TokenStream<T>
{
    buffer: Vec<T>,
    idx: usize,
}

impl<T> TokenStream<T>
{
    pub fn new(tokens: Vec<T>) -> Self
    {
        Self {
            buffer: tokens,
            idx: 0,
        }
    }

    /// Create a child iterator, which has its own internal index and holds a reference for their owner's index.
    /// When incrementing the child's index it also increments the parent's index. However, the child only holds the amount of tokens it was provided with.
    pub fn child_iterator_bulk<'child>(
        &'child mut self,
        nth: usize,
    ) -> Option<StreamChild<'child, T>>
    {
        self.buffer.get(self.idx..self.idx + nth).map(|buffer| {
            StreamChild {
                buffer,
                idx: 0,
                owner_idx_ref: &mut self.idx,
            }
        })
    }

    pub fn create_checkpoint(&self) -> StreamCheckpoint
    {
        StreamCheckpoint { idx: self.idx }
    }

    pub fn load_checkpoint(&mut self, checkpoint: StreamCheckpoint)
    {
        self.idx = checkpoint.idx;
    }

    pub fn idx_mut(&mut self) -> &mut usize
    {
        &mut self.idx
    }

    pub fn tokens_mut(&mut self) -> &mut Vec<T>
    {
        &mut self.buffer
    }

    pub const fn len(&self) -> usize
    {
        self.buffer.len()
    }

    pub fn idx(&self) -> usize
    {
        self.idx
    }

    pub const fn is_empty(&self) -> bool
    {
        self.buffer.is_empty()
    }
}

impl<T> Streamable<T> for TokenStream<T>
{
    fn peek(&self, nth: isize) -> Option<&T>
    {
        self.idx
            .checked_add_signed(nth)
            .and_then(|idx| self.buffer.get(idx))
    }

    fn peek_next(&self) -> Option<&T>
    {
        self.buffer.get(self.idx)
    }

    /// This does not remove the token from the list, therefor it is O(1).
    /// The function only increments an internal index.
    fn consume(&mut self) -> Option<&T>
    {
        let query = self.buffer.get(self.idx)?;
        self.idx += 1;
        Some(query)
    }

    /// This does not remove the token from the list, therefor it is O(1).
    /// The function only increments an internal index.
    /// If the tokenstream does not have any more items left, this function will return the provided error.
    fn try_consume_match<E: Clone, D>(&mut self, error: E, discriminant: &D) -> Result<&T, E>
    where
        T: PartialEq<D>,
    {
        let query = self.buffer.get(self.idx).ok_or(error.clone())?;

        if query != discriminant {
            return Err(error);
        }

        self.idx += 1;
        Ok(query)
    }

    /// This does not remove the token from the list, therefor it is O(1).
    /// The function only increments an internal index.
    /// The fetching is non-inclusive.
    fn consume_bulk(&mut self, nth: usize) -> Option<&[T]>
    {
        let query = self.buffer.get(self.idx..self.idx + nth)?;
        self.idx += nth;
        Some(query)
    }

    /// Decrement the cursor by `num`. If `num > self.idx` the internal index is zeroed.
    fn decrement_cursor(&mut self, num: usize)
    {
        self.idx = self.idx.saturating_sub(num);
    }

    /// Peeks the rest of the [`TokenStream`].
    fn peek_remainder(&self) -> Option<&[T]>
    {
        self.buffer.get(self.idx..)
    }

    /// Returns None if none were consumed or if there arent any tokens left in the buffer.
    fn get_last_consumed(&self) -> Option<&T>
    {
        let idx = self.idx.checked_sub(1)?;
        self.buffer.get(idx)
    }
}

#[derive(Debug)]
pub struct StreamChild<'owner, T>
{
    buffer: &'owner [T],
    idx: usize,
    owner_idx_ref: &'owner mut usize,
}

impl<'owner, T> Streamable<T> for StreamChild<'owner, T>
{
    fn peek(&self, nth: isize) -> Option<&T>
    {
        self.idx
            .checked_add_signed(nth)
            .and_then(|idx| self.buffer.get(idx))
    }

    fn peek_next(&self) -> Option<&T>
    {
        self.idx
            .checked_add_signed(1)
            .and_then(|idx| self.buffer.get(idx))
    }

    /// This does not remove the token from the list, therefor it is O(1).
    /// The function only increments an internal index.
    fn consume(&mut self) -> Option<&T>
    {
        let query = self.buffer.get(self.idx)?;
        self.idx += 1;
        *self.owner_idx_ref += 1;
        Some(query)
    }

    /// This does not remove the token from the list, therefor it is O(1).
    /// The function only increments an internal index.
    /// If the tokenstream does not have any more items left, this function will return the provided error.
    fn try_consume_match<E: Clone, D>(&mut self, error: E, discriminant: &D) -> Result<&T, E>
    where
        T: PartialEq<D>,
    {
        let query = self.buffer.get(self.idx).ok_or(error.clone())?;

        if query != discriminant {
            return Err(error);
        }

        *self.owner_idx_ref += 1;
        self.idx += 1;
        Ok(query)
    }

    /// This does not remove the token from the list, therefor it is O(1).
    /// The function only increments an internal index.
    /// The fetching is non-inclusive.
    fn consume_bulk(&mut self, nth: usize) -> Option<&[T]>
    {
        let query = self.buffer.get(self.idx..self.idx + nth)?;
        *self.owner_idx_ref += nth;
        self.idx += nth;
        Some(query)
    }

    /// Decrement the cursor by `num`. If `num > self.idx` the internal index is zeroed.
    fn decrement_cursor(&mut self, num: usize)
    {
        self.idx = self.idx.saturating_sub(num);
        *self.owner_idx_ref = self.owner_idx_ref.checked_sub(num).unwrap_or(0);
    }

    /// Peeks the rest of the [`TokenStream`].
    fn peek_remainder(&self) -> Option<&[T]>
    {
        self.buffer.get(self.idx..)
    }

    /// Returns None if none were consumed or if there arent any tokens left in the buffer.
    fn get_last_consumed(&self) -> Option<&T>
    {
        let idx = self.idx.checked_sub(1)?;
        self.buffer.get(idx)
    }
}

#[derive(Debug, Clone, Display, strum_macros::EnumTryAs, PartialEq, Eq, Hash)]
pub enum ParsedToken
{
    NewVariable
    {
        variable_name: String,
        variable_type: Type,
        variable_value: Box<ParsedTokenInstance>,
        variable_id: UniqueId,
        is_mutable: bool,
    },

    /// This is the token for referencing a variable. This is the lowest layer of referencing a variable.
    /// Other tokens might wrap it like an `ArrayIndexing`. This is the last token which points to the variable.
    VariableReference(VariableReference),

    Literal(Value),

    TypeCast(Box<ParsedTokenInstance>, Type),

    MathematicalExpression(
        Box<ParsedTokenInstance>,
        MathematicalSymbol,
        Box<ParsedTokenInstance>,
    ),

    Brackets(Vec<ParsedToken>, Type),

    FunctionCall(
        (FunctionSignature, String),
        OrdMap<FunctionArgumentIdentifier<String, usize>, (ParsedTokenInstance, (Type, UniqueId))>,
    ),

    /// The first ParsedToken is the parsedtoken referencing some kind of variable reference (Does not need to be a `VariableReference`), basicly anything.
    /// The second is the value we are setting this variable.
    SetValue(Box<ParsedTokenInstance>, Box<ParsedTokenInstance>),

    MathematicalBlock(Box<ParsedTokenInstance>),

    ReturnValue(Box<ParsedTokenInstance>),

    Comparison(
        Box<ParsedTokenInstance>,
        Order,
        Box<ParsedTokenInstance>,
        Type,
    ),

    If(If),

    CodeBlock(Vec<ParsedToken>),

    Loop(Vec<ParsedTokenInstance>),

    ControlFlow(ControlFlowType),

    ArrayInitialization(Vec<ParsedTokenInstance>, Type),

    GetPointerTo(Box<ParsedTokenInstance>),

    DerefPointer
    {
        inner_expr: Box<ParsedTokenInstance>,
        mode: DerefMode,
    },
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub enum ItemVisibility
{
    /// Not available to any scopes besides the file it was created in
    #[default]
    Private, // priv
    /// Is exposed as a function to import
    Public, // pub
    /// Can only be accessed from the same library it was created in
    PublicLibrary, // publib
    /// Branches are parsed like function, and this type is supposed to indicate that the function is actually a branch.
    /// A branch does not have any visibility, it is only for debugging.
    Branch,
}

#[derive(Clone, Debug)]
pub struct Context
{
    pub functions: PathMap<Vec<String>, String, FunctionDefinition>,
    pub items: PathMap<Vec<String>, String, CustomItem>,
    pub external_decls: PathMap<Vec<String>, String, FunctionSignature>,
    pub path: Vec<String>,
}

impl Context
{
    pub fn new(path: Vec<String>) -> Self
    {
        Self {
            functions: PathMap::new(),
            items: PathMap::new(),
            external_decls: PathMap::new(),
            path,
        }
    }

    pub fn create_function(
        &self,
        vis: ItemVisibility,
        name: String,
        arguments: FunctionArguments,
        return_type: Type,
        compiler_instructions: OrdSet<CompilerInstruction>,
        body: Vec<Spanned<ParsedToken>>,
    ) -> FunctionDefinition
    {
        FunctionDefinition {
            signature: FunctionSignature {
                name,
                args: arguments,
                return_type,
                module_path: self.path.clone(),
                visibility: vis,
                compiler_instructions,
            },
            body,
        }
    }
}

/// Pass in 0 for the `open_paren_count` if you're searching for the very next closing token on the same level.
pub fn find_closing_paren(paren_start_slice: &[Token], open_paren_count: usize) -> Result<usize>
{
    let mut paren_layer_counter = 1;
    let iter = paren_start_slice.iter().enumerate();

    for (idx, token) in iter {
        match token {
            Token::OpenParentheses => paren_layer_counter += 1,
            Token::CloseParentheses => {
                paren_layer_counter -= 1;
                if paren_layer_counter == open_paren_count {
                    return Ok(idx);
                }
            },
            _ => continue,
        }
    }

    Err(ParserError::SyntaxError(SyntaxError::LeftOpenParentheses).into())
}

/// This function will return the idx of the earliest occurence of a `|` in the provided slice.
pub fn find_next_bitor(bitor_start_slice: &[Token]) -> Result<usize>
{
    let iter = bitor_start_slice.iter().enumerate();

    for (idx, token) in iter {
        match token {
            Token::BitOr => return Ok(idx),
            _ => continue,
        }
    }

    Err(ParserError::SyntaxError(SyntaxError::LeftOpenParentheses).into())
}

pub fn find_closing_braces(tokens: &TokenStream<Spanned<Token>>) -> Option<usize>
{
    tokens.peek_remainder().and_then(|tkns| {
        let mut braces_counter: usize = 1;

        for (idx, token) in tkns.iter().enumerate() {
            if token.get_inner() == &Token::OpenBraces {
                braces_counter += 1;
            }
            else if token.get_inner() == &Token::CloseBraces {
                braces_counter -= 1;
            }

            if braces_counter == 0 {
                return Some(idx);
            }
        }

        None
    })
}

pub fn find_closing_comma(slice: &[Token]) -> Result<usize>
{
    let mut paren_level = 0;

    for (idx, item) in slice.iter().enumerate() {
        if *item == Token::OpenParentheses {
            paren_level += 1;
        }
        else if *item == Token::CloseParentheses {
            paren_level -= 1;
        }

        if *item == Token::Comma && paren_level == 0 || slice.len() - 1 == idx {
            return Ok(idx);
        }
    }

    Err(ParserError::InvalidFunctionCallArguments.into())
}

pub fn find_next_comma(slice: &[Token]) -> Result<usize>
{
    for (idx, item) in slice.iter().enumerate() {
        if *item == Token::Comma {
            return Ok(idx);
        }
    }

    Err(ParserError::SyntaxError(SyntaxError::CommaNotFound).into())
}
