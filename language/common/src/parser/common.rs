use anyhow::Result;
use strum_macros::Display;

use crate::{
    codegen::{FunctionArgumentIdentifier, If, Order},
    error::{DbgInfo, parser::ParserError, syntax::SyntaxError},
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
    pub debug_information: DbgInfo,
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

    DerefPointer(Box<ParsedTokenInstance>),
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

/// Pass in 0 for the `open_braces_count` if you're searching for the very next closing token on the same level.
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

    Err(ParserError::SyntaxError(SyntaxError::LeftOpenParentheses).into())
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
