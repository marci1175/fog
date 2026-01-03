use std::{collections::HashMap, fmt::Display, rc::Rc};

use crate::{
    codegen::{CustomType, FunctionArgumentIdentifier},
    error::{DbgInfo, parser::ParserError},
    parser::{
        common::{ParsedTokenInstance, find_closing_comma, find_closing_paren},
        value::parse_value,
    },
    tokenizer::Token,
    ty::{OrdMap, OrdSet, Type, ty_from_token},
};
use anyhow::Result;
use indexmap::IndexMap;

#[derive(Clone, Debug, Default)]
pub struct UnparsedFunctionDefinition
{
    pub function_sig: FunctionSignature,
    pub inner: Vec<Token>,

    /// This is used to offset the index when fetching [`DebugInformation`] about [`ParsedToken`]s inside the function.
    pub token_offset: usize,
}

#[derive(Debug, Clone)]
pub struct FunctionDefinition
{
    pub signature: FunctionSignature,
    pub inner: Vec<ParsedTokenInstance>,
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
    pub name: String,
    pub args: FunctionArguments,
    pub return_type: Type,
    pub module_path: Vec<String>,
    pub visibility: FunctionVisibility,
    pub compiler_hints: OrdSet<CompilerHint>,
    pub enabling_features: OrdSet<String>,
}

impl Display for FunctionSignature
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.write_str(&format!("[Function Signature]:\n{:#?}", self))
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct FunctionArguments
{
    pub arguments: OrdMap<String, Type>,
    pub ellipsis_present: bool,
}

impl FunctionArguments
{
    pub fn new() -> Self
    {
        Self {
            arguments: OrdMap::new(),
            ellipsis_present: false,
        }
    }
}

/// The slice should startwith the first token from inside the Parentheses.
/// This function quits at the ")". (Excluding function calls)
pub fn parse_function_call_args(
    tokens: &[Token],
    function_tokens_offset: usize,
    mut origin_token_idx: usize,
    debug_infos: &[DbgInfo],
    variable_scope: &mut IndexMap<String, Type>,
    mut this_function_args: FunctionArguments,
    function_signatures: Rc<IndexMap<String, UnparsedFunctionDefinition>>,
    standard_function_table: Rc<HashMap<String, FunctionSignature>>,
    custom_items: Rc<IndexMap<String, CustomType>>,
) -> anyhow::Result<(
    OrdMap<FunctionArgumentIdentifier<String, usize>, (ParsedTokenInstance, Type)>,
    usize,
)>
{
    let mut tokens_idx = 0;

    let args_list_len = tokens[tokens_idx..].len() + tokens_idx;

    // Arguments which will passed in to the function
    let mut arguments: OrdMap<
        FunctionArgumentIdentifier<String, usize>,
        (ParsedTokenInstance, Type),
    > = OrdMap::new();

    // If there are no arguments just return everything as is
    if tokens.is_empty() {
        return Ok((arguments, tokens_idx));
    }

    while tokens_idx < tokens.len() {
        let current_token = tokens[tokens_idx].clone();

        if let Token::Identifier(arg_name) = current_token.clone() {
            if let Some(Token::SetValue) = tokens.get(tokens_idx + 1) {
                let argument_type = this_function_args
                    .arguments
                    .get(&arg_name)
                    .ok_or(ParserError::ArgumentError(arg_name.clone()))?;

                tokens_idx += 2;

                let closing_idx = find_closing_comma(&tokens[tokens_idx..])? + tokens_idx;

                let (parsed_argument, jump_idx, arg_ty) = parse_value(
                    &tokens[tokens_idx..closing_idx],
                    function_tokens_offset,
                    debug_infos,
                    origin_token_idx + tokens_idx,
                    function_signatures.clone(),
                    variable_scope,
                    Some(argument_type.clone()),
                    standard_function_table.clone(),
                    custom_items.clone(),
                )?;

                tokens_idx += jump_idx;

                // Remove tha argument from the argument list so we can parse unnamed arguments easier
                this_function_args.arguments.shift_remove(&arg_name);

                arguments.insert(
                    FunctionArgumentIdentifier::Identifier(arg_name.clone()),
                    (parsed_argument, arg_ty),
                );

                continue;
            }
        }
        else if Token::CloseParentheses == current_token {
            break;
        }
        else if Token::Comma == current_token {
            tokens_idx += 1;

            continue;
        }

        let mut token_buf = Vec::new();
        let mut bracket_counter: i32 = 0;

        // Update the value of the origin_token_idx
        origin_token_idx += tokens_idx;

        // We should start by finding the comma and parsing the tokens in between the current idx and the comma
        while tokens_idx < args_list_len {
            let token = &tokens[tokens_idx];

            if *token == Token::OpenParentheses {
                bracket_counter += 1;
            }
            else if *token == Token::CloseParentheses {
                bracket_counter -= 1;
            }

            // If a comma is found parse the tokens from the slice
            if (*token == Token::Comma && bracket_counter == 0) || tokens_idx == args_list_len - 1 {
                if tokens_idx == args_list_len - 1 {
                    token_buf.push(token.clone());
                }

                let fn_argument = this_function_args.arguments.first_entry();

                if let Some(fn_argument) = fn_argument {
                    let (parsed_argument, _jump_idx, arg_ty) = parse_value(
                        &token_buf,
                        function_tokens_offset,
                        debug_infos,
                        tokens_idx,
                        function_signatures.clone(),
                        variable_scope,
                        Some(fn_argument.get().clone()),
                        standard_function_table.clone(),
                        custom_items.clone(),
                    )?;

                    tokens_idx += 1;

                    token_buf.clear();

                    arguments.insert(
                        FunctionArgumentIdentifier::Identifier(fn_argument.key().clone()),
                        (parsed_argument, arg_ty),
                    );

                    // Remove the argument from the argument list
                    fn_argument.shift_remove();
                }
                else {
                    let (parsed_argument, _jump_idx, arg_ty) = parse_value(
                        &token_buf,
                        function_tokens_offset + tokens_idx,
                        debug_infos,
                        origin_token_idx,
                        function_signatures.clone(),
                        variable_scope,
                        None,
                        standard_function_table.clone(),
                        custom_items.clone(),
                    )?;

                    tokens_idx += 1;

                    token_buf.clear();

                    let nth_argument = arguments.len();
                    arguments.insert(
                        FunctionArgumentIdentifier::Index(nth_argument),
                        (parsed_argument, arg_ty),
                    );
                }

                continue;
            }
            else {
                token_buf.push(token.clone());
            }

            tokens_idx += 1;
        }
    }

    if !this_function_args.arguments.is_empty() {
        return Err(ParserError::InvalidFunctionArgumentCount.into());
    }

    Ok((arguments, tokens_idx))
}

pub fn parse_signature_args(
    token_list: &[Token],
    custom_types: &IndexMap<String, CustomType>,
) -> Result<FunctionArguments>
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
                    args.arguments.insert(var_name, var_type.clone());

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
                else {
                    let custom_ty = ty_from_token(&token_list[args_idx + 2], custom_types)?;

                    // Store the argument in the HashMap
                    args.arguments.insert(var_name, custom_ty.clone());

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

pub fn parse_signature_argument_tokens(
    tokens: &[Token],
    custom_types: &IndexMap<String, CustomType>,
) -> Result<(usize, FunctionArguments)>
{
    let bracket_closing_idx =
        find_closing_paren(tokens, 0).map_err(|_| ParserError::InvalidSignatureDefinition)?;

    let mut args = FunctionArguments::new();

    if bracket_closing_idx != 0 {
        args = parse_signature_args(&tokens[..bracket_closing_idx], custom_types)?;
    }

    Ok((bracket_closing_idx, args))
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
    /// Feature flag to only enable compilation of the function if a certain function enabled
    Feature,
}
