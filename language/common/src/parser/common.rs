use anyhow::Result;
use strum_macros::Display;

use crate::{
    codegen::{DerefMode, FunctionArgumentIdentifier, If, Order},
    error::{SpanInfo, parser::ParserError, syntax::SyntaxError},
    parser::{
        function::FunctionSignature,
        value::MathematicalSymbol,
        variable::{ControlFlowType, UniqueId, VariableReference},
    },
    tokenizer::Token,
    ty::{OrdMap, Type, Value},
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

    pub fn peek(&self, nth: isize) -> Option<&T>
    {
        self.idx
            .checked_add_signed(nth)
            .map(|idx| self.buffer.get(idx))
            .flatten()
    }

    /// This does not remove the token from the list, therefor it is O(1).
    /// The function only increments an internal index.
    pub fn consume(&mut self) -> Option<&T>
    {
        let query = self.buffer.get(self.idx);
        self.idx += 1;
        return query;
    }

    /// This does not remove the token from the list, therefor it is O(1).
    /// The function only increments an internal index.
    /// If the tokenstream does not have any more items left, this function will return the provided error.
    pub fn try_consume_match<E: Clone, D>(&mut self, error: E, discriminant: &D) -> Result<&T, E>
    where
        T: PartialEq<D>,
    {
        let query = self.buffer.get(self.idx).ok_or(error.clone())?;

        if query != discriminant {
            return Err(error);
        }

        self.idx += 1;
        return Ok(query);
    }

    /// This does not remove the token from the list, therefor it is O(1).
    /// The function only increments an internal index.
    pub fn consume_bulk(&mut self, nth: usize) -> Option<&[T]>
    {
        let query = self.buffer.get(self.idx..self.idx + nth);
        self.idx += nth;
        return query;
    }

    /// Decrement the cursor by `num`. If `num > self.idx` the internal index is zeroed.
    pub fn decrement_cursor(&mut self, num: usize)
    {
        self.idx = self.idx.checked_sub(num).unwrap_or(0);
    }

    pub fn get_last_consumed(&self) -> Option<&T>
    {
        self.buffer.get(self.idx - 1)
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

/// Pass in 0 for the `open_braces_count` if you're searching for the very next closing token on the same nestedness.
/// The index this will return will point to the closing `}`.
pub fn find_closing_braces(braces_start_slice: &[Token], open_braces_count: usize)
-> Result<usize>
{
    let mut braces_layer_counter = 1;
    let iter = braces_start_slice.iter().enumerate();

    for (idx, token) in iter {
        match token {
            Token::OpenBraces => braces_layer_counter += 1,
            Token::CloseBraces => {
                braces_layer_counter -= 1;
                if braces_layer_counter == open_braces_count {
                    return Ok(idx);
                }
            },
            _ => continue,
        }
    }

    Err(ParserError::SyntaxError(SyntaxError::LeftOpenBraces).into())
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
