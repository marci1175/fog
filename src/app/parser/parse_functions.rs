use anyhow::Result;
use std::{collections::HashMap, sync::Arc};

use crate::app::type_system::type_system::TypeDiscriminants;

use super::{
    error::ParserError,
    parser::{find_closing_bracket, parse_token_as_value, parse_value},
    tokens::{
        FunctionDefinition, FunctionSignature, MathematicalSymbol, ParsedToken, Token,
        UnparsedFunctionDefinition,
    },
};

pub fn create_function_table(
    tokens: Vec<Token>,
) -> Result<HashMap<String, UnparsedFunctionDefinition>> {
    let mut token_idx = 0;
    let mut function_list: HashMap<String, UnparsedFunctionDefinition> = HashMap::new();

    while token_idx < tokens.len() {
        let current_token = tokens[token_idx].clone();

        if current_token == Token::Function {
            if let Token::Identifier(function_name) = tokens[token_idx + 1].clone() {
                if tokens[token_idx + 2] == Token::OpenBracket {
                    let (bracket_close_idx, args) =
                        parse_function_argument_tokens(&tokens[token_idx + 3..])?;

                    token_idx += bracket_close_idx + 3;

                    if tokens[token_idx + 1] == Token::Colon {
                        if let Token::TypeDefinition(return_type) = tokens[token_idx + 2] {
                            if tokens[token_idx + 3] == Token::OpenBraces {
                                // Create a varable which stores the level of braces we are in
                                let mut brace_layer_counter = 1;

                                // Get the slice of the list which may contain the brackets' scope
                                let tokens_slice = &tokens[token_idx + 4..];

                                // Create an index which indexes the tokens slice
                                let mut token_braces_idx = 0;

                                // Create a list which contains all the tokens inside the two brackets
                                let mut braces_contains: Vec<Token> = vec![];

                                // Find the scope of this function
                                loop {
                                    // We have itered through the whole function and its still not found, it may be an open brace.
                                    if tokens_slice.len() == token_braces_idx {
                                        return Err(ParserError::SyntaxError(
                                            crate::app::parser::error::SyntaxError::OpenBracket,
                                        )
                                        .into());
                                    }

                                    // If a bracket is closed the layer counter should be incremented
                                    if tokens_slice[token_braces_idx] == Token::OpenBraces {
                                        brace_layer_counter += 1;
                                    }
                                    // If a bracket is closed the layer counter should be decreased
                                    else if tokens_slice[token_braces_idx] == Token::CloseBraces {
                                        brace_layer_counter -= 1;
                                    }

                                    // If we have arrived at the end of the brackets this is when we know that this is the end of the function's scope
                                    if brace_layer_counter == 0 {
                                        break;
                                    }

                                    // Store the current item in the token buffer
                                    braces_contains.push(tokens_slice[token_braces_idx].clone());

                                    // Increment the index
                                    token_braces_idx += 1;
                                }

                                let braces_contains_len = braces_contains.len();

                                // Store the function
                                function_list.insert(
                                    function_name,
                                    UnparsedFunctionDefinition {
                                        inner: braces_contains,
                                        function_sig: FunctionSignature { args, return_type },
                                    },
                                );

                                // Set the iterator index
                                token_idx += braces_contains_len + 4;

                                // Countinue with the loop
                                continue;
                            }
                        }
                    }

                    return Err(ParserError::InvalidFunctionDefinition.into());
                } else {
                    return Err(ParserError::InvalidFunctionDefinition.into());
                }
            }
        }

        token_idx += 1;
    }

    Ok(function_list)
}

fn parse_function_argument_tokens(
    tokens: &[Token],
) -> Result<(usize, HashMap<String, TypeDiscriminants>)> {
    let bracket_closing_idx =
        find_closing_bracket(tokens).map_err(|_| ParserError::InvalidFunctionDefinition)?;

    let mut args = HashMap::new();

    if bracket_closing_idx != 0 {
        args = parse_function_args(&tokens[..bracket_closing_idx])?;
    }

    Ok((bracket_closing_idx, args))
}

fn parse_function_args(token_list: &[Token]) -> Result<HashMap<String, TypeDiscriminants>> {
    // Create a list of args which the function will take, we will return this later
    let mut args: HashMap<String, TypeDiscriminants> = HashMap::new();

    // Create an index which will iterate through the tokens
    let mut args_idx = 0;

    // Iter until we find a CloseBracket: ")"
    // This will be the end of the function's arguments
    while args_idx < token_list.len() {
        // Match the signature of an argument
        // Get the variable's name
        // If the token is an identifier then we know that this is a variable name
        // If the token is a colon then we know that this is a type definition
        if let Token::Identifier(var_name) = token_list[args_idx].clone() {
            // Match the colon from the signature, to ensure correct signaure
            if token_list[args_idx + 1] == Token::Colon {
                // Get the type of the argument
                if let Token::TypeDefinition(var_type) = token_list[args_idx + 2] {
                    // Store the argument in the HashMap
                    args.insert(var_name, var_type);

                    // Increment the idx based on the next token
                    if let Some(Token::Comma) = token_list.get(args_idx + 3) {
                        args_idx += 4;
                    } else {
                        args_idx += 3;
                    }

                    // Countinue the loop
                    continue;
                }
            }
        }

        // If the pattern didnt match the tokens return an error
        return Err(ParserError::InvalidFunctionDefinition.into());
    }

    Ok(args)
}

pub fn parse_functions(
    unparsed_functions: Arc<HashMap<String, UnparsedFunctionDefinition>>,
) -> Result<HashMap<String, FunctionDefinition>> {
    let mut parsed_functions = HashMap::new();

    for (fn_name, unparsed_function) in unparsed_functions.clone().iter() {
        let function_definition = FunctionDefinition {
            function_sig: unparsed_function.function_sig.clone(),
            inner: parse_function_block(
                unparsed_function.inner.clone(),
                unparsed_functions.clone(),
                unparsed_function.function_sig.clone(),
            )?,
        };

        parsed_functions.insert(fn_name.clone(), function_definition);
    }

    Ok(parsed_functions)
}

fn parse_function_block(
    tokens: Vec<Token>,
    function_signatures: Arc<HashMap<String, UnparsedFunctionDefinition>>,
    this_function_signature: FunctionSignature,
) -> Result<Vec<ParsedToken>> {
    let mut token_idx = 0;

    let mut parsed_tokens: Vec<ParsedToken> = Vec::new();

    let mut variable_scope: HashMap<String, TypeDiscriminants> =
        this_function_signature.args.clone();

    if tokens.is_empty() {
        return Ok(vec![]);
    }

    while token_idx < tokens.len() {
        let current_token = tokens[token_idx].clone();

        if let Token::TypeDefinition(var_type) = current_token {
            if let Token::Identifier(var_name) = tokens[token_idx + 1].clone() {
                if tokens[token_idx + 2] == Token::SetValue {
                    let line_break_idx = tokens
                        .iter()
                        .skip(token_idx + 2)
                        .position(|token| *token == Token::LineBreak)
                        .ok_or({
                            ParserError::SyntaxError(
                                crate::app::parser::error::SyntaxError::MissingLineBreak,
                            )
                        })?
                        + token_idx
                        + 2;

                    let selected_tokens = &tokens[token_idx + 3..line_break_idx];

                    // Set the new idx
                    token_idx = line_break_idx;

                    let (parsed_value, _) = parse_value(
                        selected_tokens,
                        function_signatures.clone(),
                        &variable_scope,
                        var_type,
                    )?;

                    parsed_tokens.push(ParsedToken::NewVariable((
                        var_name.clone(),
                        Box::new(parsed_value.clone()),
                    )));

                    variable_scope.insert(var_name, var_type);
                } else {
                    parsed_tokens.push(ParsedToken::NewVariable((
                        var_name.clone(),
                        Box::new(ParsedToken::Literal(var_type.into())),
                    )));

                    variable_scope.insert(var_name.clone(), var_type);

                    token_idx += 2;
                }

                if *dbg!(&tokens[token_idx]) == Token::LineBreak {
                    token_idx += 1;

                    continue;
                } else {
                    return Err(ParserError::SyntaxError(
                        crate::app::parser::error::SyntaxError::MissingLineBreak,
                    )
                    .into());
                }
            } else {
                return Err(ParserError::SyntaxError(
                    crate::app::parser::error::SyntaxError::InvalidStatementDefinition,
                )
                .into());
            }
        } else if let Token::Identifier(ident_name) = current_token {
            // If the variable exists in the current scope
            if let Some(variable_type) = variable_scope.get(&ident_name) {
                // Increment the token index
                token_idx += 1;

                match &tokens[token_idx] {
                    Token::SetValue => {
                        let line_break_idx = tokens
                            .iter()
                            .skip(token_idx)
                            .position(|token| *token == Token::LineBreak)
                            .ok_or({
                                ParserError::SyntaxError(
                                    crate::app::parser::error::SyntaxError::MissingLineBreak,
                                )
                            })?
                            + token_idx;

                        let selected_tokens = &tokens[token_idx + 1..line_break_idx];

                        token_idx += selected_tokens.len() + 1;

                        let (parsed_token, _) = parse_value(
                            selected_tokens,
                            function_signatures.clone(),
                            &variable_scope,
                            *variable_type,
                        )?;

                        parsed_tokens.push(ParsedToken::SetValue(
                            ident_name.clone(),
                            Box::new(parsed_token),
                        ));
                    }
                    Token::SetValueAddition => {
                        set_value_math_expr(
                            &tokens,
                            &function_signatures,
                            &mut token_idx,
                            &mut parsed_tokens,
                            &variable_scope,
                            variable_type,
                            &ident_name,
                            MathematicalSymbol::Addition,
                        )?;
                    }
                    Token::SetValueSubtraction => {
                        set_value_math_expr(
                            &tokens,
                            &function_signatures,
                            &mut token_idx,
                            &mut parsed_tokens,
                            &variable_scope,
                            variable_type,
                            &ident_name,
                            MathematicalSymbol::Subtraction,
                        )?;
                    }
                    Token::SetValueDivision => {
                        set_value_math_expr(
                            &tokens,
                            &function_signatures,
                            &mut token_idx,
                            &mut parsed_tokens,
                            &variable_scope,
                            variable_type,
                            &ident_name,
                            MathematicalSymbol::Division,
                        )?;
                    }
                    Token::SetValueMultiplication => {
                        set_value_math_expr(
                            &tokens,
                            &function_signatures,
                            &mut token_idx,
                            &mut parsed_tokens,
                            &variable_scope,
                            variable_type,
                            &ident_name,
                            MathematicalSymbol::Multiplication,
                        )?;
                    }
                    Token::SetValueModulo => {
                        set_value_math_expr(
                            &tokens,
                            &function_signatures,
                            &mut token_idx,
                            &mut parsed_tokens,
                            &variable_scope,
                            variable_type,
                            &ident_name,
                            MathematicalSymbol::Modulo,
                        )?;
                    }
                    _ => {
                        println!("UNIMPLEMENTED FUNCTION: {}", tokens[token_idx]);
                    }
                }
            } else if let Some(function_sig) = function_signatures.get(&ident_name) {
                // If after the function name the first thing isnt a `(` return a syntax error.
                if tokens[token_idx + 1] != Token::OpenBracket {
                    return Err(ParserError::SyntaxError(
                        crate::app::parser::error::SyntaxError::InvalidFunctionDefinition,
                    )
                    .into());
                }

                let bracket_start_slice = &tokens[token_idx + 2..];

                let bracket_idx = find_closing_bracket(bracket_start_slice)? + token_idx;

                let (variables_passed, jumped_idx) = parse_function_call_args(
                    &tokens[token_idx + 2..bracket_idx + 2],
                    &variable_scope,
                    function_sig.function_sig.args.clone(),
                    function_signatures.clone(),
                )?;

                parsed_tokens.push(ParsedToken::FunctionCall(
                    (function_sig.function_sig.clone(), ident_name),
                    variables_passed,
                ));

                token_idx += jumped_idx;
            } else {
                return Err(ParserError::VariableNotFound(ident_name).into());
            }
        } else if let Token::Return = current_token {
            token_idx += 1;

            let next_token = &tokens[token_idx];

            parsed_tokens.push(ParsedToken::ReturnValue(Box::new(parse_token_as_value(
                &tokens,
                &function_signatures,
                &variable_scope,
                this_function_signature.return_type,
                &mut token_idx,
                next_token,
            )?)));
        }

        token_idx += 1;
    }

    Ok(parsed_tokens)
}

fn set_value_math_expr(
    tokens: &Vec<Token>,
    function_signatures: &Arc<HashMap<String, UnparsedFunctionDefinition>>,
    token_idx: &mut usize,
    parsed_tokens: &mut Vec<ParsedToken>,
    variable_scope: &HashMap<String, TypeDiscriminants>,
    variable_type: &TypeDiscriminants,
    ident_name: &String,
    math_symbol: MathematicalSymbol,
) -> Result<(), anyhow::Error> {
    *token_idx += 1;

    let eval_token = tokens.get(*token_idx).ok_or(ParserError::SyntaxError(
        super::error::SyntaxError::InvalidStatementDefinition,
    ))?;

    let next_token = parse_token_as_value(
        tokens,
        function_signatures,
        variable_scope,
        *variable_type,
        token_idx,
        eval_token,
    )?;

    parsed_tokens.push(ParsedToken::SetValue(
        ident_name.clone(),
        Box::new(ParsedToken::MathematicalExpression(
            Box::new(ParsedToken::VariableReference(ident_name.clone())),
            math_symbol,
            Box::new(next_token),
        )),
    ));

    Ok(())
}

/// First token should be the first argument
pub fn parse_function_call_args(
    tokens: &[Token],
    variable_scope: &HashMap<String, TypeDiscriminants>,
    this_function_args: HashMap<String, TypeDiscriminants>,
    function_signatures: Arc<HashMap<String, UnparsedFunctionDefinition>>,
) -> Result<(Vec<ParsedToken>, usize)> {
    let mut tokens_idx = 0;

    // Arguments which will passed in to the function
    let mut arguments: Vec<ParsedToken> = vec![];

    while tokens_idx < tokens.len() {
        let current_token = tokens[tokens_idx].clone();

        if let Token::Identifier(arg_name) = current_token.clone() {
            if Token::SetValue == tokens[tokens_idx + 1] {
                let argument_type = this_function_args
                    .get(&arg_name)
                    .ok_or(ParserError::ArgumentError(arg_name))?;

                tokens_idx += 2;

                let (parsed_argument, jump_idx) = parse_value(
                    &tokens[tokens_idx..tokens.len() - 1],
                    function_signatures.clone(),
                    variable_scope,
                    *argument_type,
                )?;

                dbg!(&tokens[tokens_idx]);

                tokens_idx += jump_idx;

                arguments.push(parsed_argument);
            } else {
                return Err(ParserError::SyntaxError(
                    super::error::SyntaxError::InvalidStatementDefinition,
                )
                .into());
            }
        } else if Token::CloseBracket == current_token {
            break;
        } else if Token::Comma == current_token {
            tokens_idx += 1;
        } else {
            return Err(ParserError::SyntaxError(
                super::error::SyntaxError::InvalidStatementDefinition,
            )
            .into());
        }
    }

    Ok((arguments, tokens_idx))
}
