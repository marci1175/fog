use std::{collections::HashMap, fmt::Display, rc::Rc};

use crate::{
    anyhow::{self, Result},
    codegen::{CustomItem, FunctionArgumentIdentifier},
    error::{DbgInfo, parser::ParserError, syntax::SyntaxError},
    indexmap::IndexMap,
    parser::{
        common::{ParsedToken, ParsedTokenInstance, find_closing_comma, find_closing_paren},
        dbg::fetch_and_merge_debug_information,
        value::parse_value,
        variable::{UniqueId, VARIABLE_ID_SOURCE, VariableReference},
    },
    tokenizer::Token,
    ty::{OrdMap, OrdSet, Type, ty_from_token},
};

#[derive(Clone, Debug, Default, PartialEq, Hash)]
pub struct UnparsedFunctionDefinition
{
    pub signature: FunctionSignature,
    pub inner: Vec<Token>,

    /// This is used to offset the index when fetching [`DebugInformation`] about [`ParsedToken`]s inside the function.
    pub token_offset: usize,
}

#[derive(Debug, Clone, PartialEq, Hash, Default, Eq)]
pub struct FunctionDefinition
{
    pub signature: FunctionSignature,
    pub inner: Vec<ParsedTokenInstance>,
    /// This is used to offset the index when fetching [`DebugInformation`] about [`ParsedToken`]s inside the function.
    pub token_offset: usize,
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
    /// Even though [`UniqueId`]s are truly unique, we still dont want to use them (for now) as a key because strings are quniue in this context.
    pub arguments: OrdMap<String, (Type, UniqueId)>,
    pub ellipsis_present: bool,
    /// This is true if the function references the struct its implemented for ie. using the this keyword.
    /// Obviously this shouldnt be true for an ordinary function since the `this` keyword cannot be used there.
    pub receiver_referenced: bool,
}

impl FunctionArguments
{
    /// We need to implement a Custom eq check on [`FunctionArguments`]s because the use of [`UniqueId`].
    pub fn check_arg_eq(&self, rhs: &Self) -> bool
    {
        self.arguments
            .iter()
            .map(|(name, (ty, _))| (name, ty))
            .collect::<Vec<_>>()
            == rhs
                .arguments
                .iter()
                .map(|(name, (ty, _))| (name, ty))
                .collect::<Vec<_>>()
    }
}

impl FunctionArguments
{
    pub fn new() -> Self
    {
        Self {
            arguments: OrdMap::new(),
            ellipsis_present: false,
            receiver_referenced: false,
        }
    }
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

/// The slice should startwith the first token from inside the Parentheses.
/// This function quits at the ")".
pub fn parse_function_call_args(
    tokens: &[Token],
    function_tokens_offset: usize,
    mut origin_token_idx: usize,
    debug_infos: &[DbgInfo],
    variable_scope: &mut IndexMap<String, (Type, UniqueId)>,
    mut this_function_args: FunctionArguments,
    function_signatures: Rc<IndexMap<String, UnparsedFunctionDefinition>>,
    imported_functions: Rc<HashMap<String, FunctionSignature>>,
    custom_items: Rc<IndexMap<String, CustomItem>>,
) -> anyhow::Result<(
    OrdMap<FunctionArgumentIdentifier<String, usize>, (ParsedTokenInstance, (Type, UniqueId))>,
    usize,
)>
{
    let mut tokens_idx = 0;

    let args_list_len = tokens[tokens_idx..].len() + tokens_idx;

    // Arguments which will passed in to the function
    let mut arguments: OrdMap<
        FunctionArgumentIdentifier<String, usize>,
        (ParsedTokenInstance, (Type, UniqueId)),
    > = OrdMap::new();

    // If there are no arguments just return everything as is
    if tokens.is_empty() {
        return Ok((arguments, tokens_idx));
    }

    // Check if there has been a receiver provided, if yes we should remove the `this` argument automaticly added to the function's arguments at function generation.
    // if let Some(var_ref) = receiver {
    //     if let Some((name, (this_argument, id))) = this_function_args.arguments.swap_remove_index(0)
    //     {
    //         arguments.insert(
    //             FunctionArgumentIdentifier::Index(0),
    //             (
    //                 ParsedTokenInstance {
    //                     inner: ParsedToken::VariableReference(var_ref.clone()),
    //                     debug_information: fetch_and_merge_debug_information(
    //                         debug_infos,
    //                         tokens_idx..tokens_idx + 2,
    //                         true,
    //                     )
    //                     .unwrap(),
    //                 },
    //                 (this_argument.clone(), id),
    //             ),
    //         );

    //         // Do syntax check and increment index accordingly
    //         if tokens.get(tokens_idx + 1) == Some(&Token::Comma)
    //             || this_function_args.arguments.len() == 0
    //         {
    //             tokens_idx += 2;
    //         }
    //         else {
    //             return Err(ParserError::SyntaxError(SyntaxError::MissingCommaInFnCall).into());
    //         }
    //     }
    //     // If this is a None then this is an internal error.
    //     // If the user can provide a receiver then that means that the first argument (at idx 0) MUST be the receiver (`this`)
    //     else {
    //         return Err(ParserError::InternalFunctionReceiverArgMissing.into());
    //     }
    // }

    while tokens_idx < tokens.len() {
        let current_token = tokens[tokens_idx].clone();

        if let Token::Identifier(arg_name) = current_token.clone() {
            if let Some(Token::SetValue) = tokens.get(tokens_idx + 1) {
                let (argument_type, argument_variable_id) = this_function_args
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
                    imported_functions.clone(),
                    custom_items.clone(),
                )?;

                tokens_idx += jump_idx;

                let argmuent_id = *argument_variable_id;

                // Remove tha argument from the argument list so we can parse unnamed arguments easier
                this_function_args.arguments.shift_remove(&arg_name);

                arguments.insert(
                    FunctionArgumentIdentifier::Identifier(arg_name.clone()),
                    (parsed_argument, (arg_ty, argmuent_id)),
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
                    let (arg_ty, arg_id) = fn_argument.get();
                    let (parsed_argument, _jump_idx, arg_ty) = parse_value(
                        &token_buf,
                        function_tokens_offset,
                        debug_infos,
                        tokens_idx,
                        function_signatures.clone(),
                        variable_scope,
                        Some(arg_ty.clone()),
                        imported_functions.clone(),
                        custom_items.clone(),
                    )?;

                    tokens_idx += 1;

                    token_buf.clear();

                    arguments.insert(
                        FunctionArgumentIdentifier::Identifier(fn_argument.key().clone()),
                        (parsed_argument, (arg_ty, *arg_id)),
                    );

                    // Remove the argument from the argument list
                    fn_argument.shift_remove();
                }
                // If an argument is apssed into a function which takes a variable amount of arguments, it wont be found in the fn argument list
                // We can allocate a new variable id to the argument passed in this way
                else {
                    let (parsed_argument, _jump_idx, arg_ty) = parse_value(
                        &token_buf,
                        function_tokens_offset + tokens_idx,
                        debug_infos,
                        origin_token_idx,
                        function_signatures.clone(),
                        variable_scope,
                        None,
                        imported_functions.clone(),
                        custom_items.clone(),
                    )?;

                    tokens_idx += 1;

                    token_buf.clear();

                    let nth_argument = arguments.len();

                    arguments.insert(
                        FunctionArgumentIdentifier::Index(nth_argument),
                        (
                            parsed_argument,
                            (arg_ty, VARIABLE_ID_SOURCE.get_unique_id()),
                        ),
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
    tokens: &[Token],
    custom_types: &IndexMap<String, CustomItem>,
    is_struct_implementation: bool,
) -> Result<FunctionArguments>
{
    // Create a list of args which the function will take, we will return this later
    let mut args: FunctionArguments = FunctionArguments::new();

    // Create an index which will iterate through the tokens
    let mut args_idx = 0;

    // Iter until we find a CloseBracket: ")"
    // This will be the end of the function's arguments
    while args_idx < tokens.len() {
        // Match the signature of an argument
        // Get the variable's name
        // If the token is an identifier then we know that this is a variable name
        // If the token is a colon then we know that this is a type definition
        let current_token = &tokens[args_idx];

        // Match the current token
        if let Token::Identifier(var_name) = current_token {
            // Match the colon from the signature, to ensure correct signaure
            if tokens[args_idx + 1] == Token::Colon {
                // Get the type of the argument
                if let Token::TypeDefinition(var_type) = &tokens[args_idx + 2] {
                    // Store the argument in the HashMap
                    args.arguments.insert(
                        var_name.clone(),
                        (var_type.clone(), VARIABLE_ID_SOURCE.get_unique_id()),
                    );

                    // Increment the idx based on the next token
                    if let Some(Token::Comma) = tokens.get(args_idx + 3) {
                        args_idx += 4;
                    }
                    else {
                        args_idx += 3;
                    }

                    // Countinue the loop
                    continue;
                }
                else {
                    let custom_ty = ty_from_token(&tokens[args_idx + 2], custom_types)?;

                    // Store the argument in the HashMap
                    args.arguments.insert(
                        var_name.clone(),
                        (custom_ty.clone(), VARIABLE_ID_SOURCE.get_unique_id()),
                    );

                    // Increment the idx based on the next token
                    if let Some(Token::Comma) = tokens.get(args_idx + 3) {
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
        // This token must only be at the first position of the arguments
        else if &Token::This == current_token {
            // Check the position of the `this` token
            if args_idx != 0 {
                return Err(ParserError::InvalidReceiverPosition.into());
            }

            // Check if the use of `this` is allowed
            if !is_struct_implementation {
                return Err(ParserError::InvalidReceiverUsage.into());
            }

            // Check for syntax validity
            let next_token = tokens.get(args_idx + 1);

            // Increment cursor
            args_idx += 2;

            // Set the arg
            args.receiver_referenced = true;

            // If the next token isnt a Comma even though there are tokens left, we should not continue and we should return an error
            if !(next_token.is_some()
                && next_token != Some(&Token::Comma)
                && args_idx < tokens.len())
            {
                continue;
            }
        }
        // If an ellipsis is found, that means that there can be an indefinite amount of arguments, this however can only be used at the end of the arguments when importing an external function
        else if &Token::Ellipsis == current_token {
            // Check if this is the last argument, and return an error if it isn't
            if args_idx != tokens.len() - 1 {
                return Err(ParserError::InvalidEllipsisPosition.into());
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
    custom_types: &IndexMap<String, CustomItem>,
    is_struct_implementation: bool,
) -> Result<(usize, FunctionArguments)>
{
    let bracket_closing_idx =
        find_closing_paren(tokens, 0).map_err(|_| ParserError::InvalidSignatureDefinition)?;

    let mut args = FunctionArguments::new();

    if bracket_closing_idx != 0 {
        args = parse_signature_args(
            &tokens[..bracket_closing_idx],
            custom_types,
            is_struct_implementation,
        )?;
    }

    Ok((bracket_closing_idx, args))
}
