use anyhow::Result;
use std::{
    collections::{BTreeSet, HashSet},
    fmt::Display,
};
use strum_macros::Display;

use crate::{
    codegen::{FunctionArgumentIdentifier, If, Order},
    error::{parser::ParserError, syntax::SyntaxError},
    tokenizer::Token,
    ty::{OrdMap, Type, TypeDiscriminant},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MathematicalSymbol
{
    Addition,
    Subtraction,
    Division,
    Multiplication,
    Modulo,
}

impl TryInto<MathematicalSymbol> for Token
{
    type Error = ParserError;

    fn try_into(self) -> Result<MathematicalSymbol, Self::Error>
    {
        let expr = match self {
            Self::Addition => MathematicalSymbol::Addition,
            Self::Subtraction => MathematicalSymbol::Subtraction,
            Self::Division => MathematicalSymbol::Division,
            Self::Multiplication => MathematicalSymbol::Multiplication,
            Self::Modulo => MathematicalSymbol::Modulo,

            _ => return Err(ParserError::InternalVariableError),
        };

        Ok(expr)
    }
}

#[derive(Debug, Clone, Display, strum_macros::EnumTryAs, PartialEq, Eq, Hash)]
pub enum ParsedToken
{
    NewVariable(String, TypeDiscriminant, Box<ParsedToken>),

    /// This is the token for referencing a variable. This is the lowest layer of referencing a variable.
    /// Other tokens might wrap it like an `ArrayIndexing`. This is the last token which points to the variable.
    VariableReference(VariableReference),

    Literal(Type),

    TypeCast(Box<ParsedToken>, TypeDiscriminant),

    MathematicalExpression(Box<ParsedToken>, MathematicalSymbol, Box<ParsedToken>),

    Brackets(Vec<ParsedToken>, TypeDiscriminant),

    FunctionCall(
        (FunctionSignature, String),
        OrdMap<FunctionArgumentIdentifier<String, usize>, (ParsedToken, TypeDiscriminant)>,
    ),

    /// The first ParsedToken is the parsedtoken referencing some kind of variable reference (Does not need to be a `VariableReference`), basicly anything.
    /// The second is the value we are setting this variable.
    SetValue(Box<ParsedToken>, Box<ParsedToken>),

    MathematicalBlock(Box<ParsedToken>),

    ReturnValue(Box<ParsedToken>),

    Comparison(Box<ParsedToken>, Order, Box<ParsedToken>, TypeDiscriminant),

    If(If),

    InitializeStruct(
        OrdMap<String, TypeDiscriminant>,
        OrdMap<String, Box<ParsedToken>>,
    ),

    CodeBlock(Vec<ParsedToken>),

    Loop(Vec<ParsedToken>),

    ControlFlow(ControlFlowType),

    /// The first ParsedToken is the parsedtoken referencing some kind of variable reference (Does not need to be a `VariableReference`), basicly anything.
    /// The second argument is the index we are referencing at.
    ArrayIndexing(Box<ParsedToken>, Box<ParsedToken>),

    ArrayInitialization(Vec<ParsedToken>, TypeDiscriminant),
}

#[derive(Debug, Clone, Display, PartialEq, Eq, Hash)]
pub enum ControlFlowType
{
    Break,
    Continue,
}

#[derive(Debug, Clone, Display, PartialEq, Eq, Hash)]
/// VariableReferences are the lowest layer of referencing a variable. This is enum wrapped in a ParsedToken, consult the documentation of that enum variant for more information.ű
/// VariableReferences should not contain themselves as they are only for referencing a variable, there is not much more to it.
pub enum VariableReference
{
    /// Variable name, (struct_name, struct_type)
    StructFieldReference(
        StructFieldReference,
        (String, OrdMap<String, TypeDiscriminant>),
    ),
    /// Variable name
    BasicReference(String),
    /// Variable name, array index
    ArrayReference(String, Box<ParsedToken>),
}

/// The first item of the StructFieldReference is used to look up the name of the variable which stores the Struct.
/// The functions which take the iterator of the `field_stack` field should not be passed the first item of the iterator, since the first item is used to look up the name of the variable which stores the struct.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StructFieldReference
{
    /// The name of the fields which get referenced
    pub field_stack: Vec<String>,
}

impl Default for StructFieldReference
{
    fn default() -> Self
    {
        Self::new()
    }
}

impl StructFieldReference
{
    /// Creates an instnace from a single entry
    pub fn from_single_entry(field_name: String) -> Self
    {
        Self {
            field_stack: vec![field_name],
        }
    }

    /// Initializes an instance from a list of field entries
    pub fn from_stack(field_stack: Vec<String>) -> Self
    {
        Self { field_stack }
    }

    /// Creates an instnace from an empty list
    pub fn new() -> Self
    {
        Self {
            field_stack: vec![],
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct UnparsedFunctionDefinition
{
    pub function_sig: FunctionSignature,
    pub inner: Vec<Token>,
}

#[derive(Debug, Clone)]
pub struct FunctionDefinition
{
    pub function_sig: FunctionSignature,
    pub inner: Vec<ParsedToken>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub enum FunctionVisibility
{
    #[default]
    Private,
    Public,
    PublicLibrary,
    /// Branches are parsed like function, and this type is supposed to indicate that the function is actually a branch.
    /// A branch does not have any visibility.
    Branch,
}

impl TryFrom<Token> for FunctionVisibility
{
    type Error = ParserError;

    fn try_from(value: Token) -> Result<Self, Self::Error>
    {
        Ok(match value {
            Token::Public => Self::Public,
            Token::PublicLibrary => Self::PublicLibrary,
            Token::Private => Self::Private,
            _ => return Err(ParserError::InvalidSignatureDefinition),
        })
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct FunctionSignature
{
    pub args: FunctionArguments,
    pub return_type: TypeDiscriminant,
    pub debug_attributes: Option<String>,
    pub module_path: Vec<String>,
    pub visibility: FunctionVisibility,
    pub compiler_hints: Vec<CompilerHint>,
}

impl Display for FunctionSignature
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.write_str(&format!(
            "Arguments: {:?}, Return type: {}, Debug Attributes: {:?}",
            self.args, self.return_type, self.debug_attributes
        ))
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct FunctionArguments
{
    pub arguments_list: OrdMap<String, TypeDiscriminant>,
    pub ellipsis_present: bool,
}

impl FunctionArguments
{
    pub fn new() -> Self
    {
        Self {
            arguments_list: OrdMap::new(),
            ellipsis_present: false,
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

pub fn parse_signature_args(token_list: &[Token]) -> Result<FunctionArguments>
{
    // Create a list of args which the function will take, we will return this later
    let mut args: FunctionArguments = FunctionArguments::new();

    // Create an index which will iterate through the tokens
    let mut args_idx = 0;

    // Iter until we find a CloseBracket: ")"
    // This will be the end of the function's arguments
    while args_idx < token_list.len() {
        // Match the signature of an argument
        // Get the variable's name
        // If the token is an identifier then we know that this is a variable name
        // If the token is a colon then we know that this is a type definition
        let current_token = token_list[args_idx].clone();
        if let Token::Identifier(var_name) = current_token {
            // Match the colon from the signature, to ensure correct signaure
            if token_list[args_idx + 1] == Token::Colon {
                // Get the type of the argument
                if let Token::TypeDefinition(var_type) = &token_list[args_idx + 2] {
                    // Store the argument in the HashMap
                    args.arguments_list.insert(var_name, var_type.clone());

                    // Increment the idx based on the next token
                    if let Some(Token::Comma) = token_list.get(args_idx + 3) {
                        args_idx += 4;
                    }
                    else {
                        args_idx += 3;
                    }

                    // Countinue the loop
                    continue;
                }
            }
        }
        // If an ellipsis is found, that means that there can be an indefinite amount of arguments, this however can only be used at the end of the arguments when importing an external function
        else if Token::Ellipsis == current_token {
            // Check if this is the last argument, and return an error if it isn't
            if args_idx != token_list.len() - 1 {
                return Err(ParserError::InvalidEllipsisLocation.into());
            }

            // Store the ellipsis
            args.ellipsis_present = true;

            args_idx += 1;

            // Countinue the loop
            continue;
        }

        // If the pattern didnt match the tokens return an error
        return Err(ParserError::InvalidSignatureDefinition.into());
    }

    Ok(args)
}

pub fn parse_signature_argument_tokens(tokens: &[Token]) -> Result<(usize, FunctionArguments)>
{
    let bracket_closing_idx =
        find_closing_paren(tokens, 0).map_err(|_| ParserError::InvalidSignatureDefinition)?;

    let mut args = FunctionArguments::new();

    if bracket_closing_idx != 0 {
        args = parse_signature_args(&tokens[..bracket_closing_idx])?;
    }

    Ok((bracket_closing_idx, args))
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

#[derive(Debug, Clone, PartialEq, strum_macros::Display, Eq, Hash)]
pub enum CompilerHint
{
    /// See llvm function attributes
    Cold,
    /// See llvm function attributes
    NoFree,
    /// See llvm function attributes
    Inline,
    /// See llvm function attributes
    NoUnWind,

    Feature(String),
}
