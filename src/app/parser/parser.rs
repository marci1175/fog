use std::{collections::HashMap, process::id, sync::Arc};

use strum::IntoDiscriminant;

use crate::app::type_system::{Type, TypeDiscriminants};

use super::{
    error::ParserError,
    types::{unparsed_const_to_typed_literal, FunctionDefinition, FunctionSignature, ParsedTokens, Tokens, UnparsedFunctionDefinition},
};

pub fn parse_code(raw_string: String) -> Result<Vec<Tokens>, ParserError> {
    let mut char_idx: usize = 0;

    let mut token_list: Vec<Tokens> = Vec::new();

    let char_list = raw_string.chars().collect::<Vec<char>>();

    let mut string_buffer = String::new();

    while char_idx < raw_string.len() {
        let current_char = char_list[char_idx];

        let single_char = match current_char {
            '+' => Some(Tokens::Addition),
            '*' => Some(Tokens::Multiplication),
            '/' => Some(Tokens::Division),
            ')' => Some(Tokens::CloseBracket),
            '(' => Some(Tokens::OpenBracket),
            '}' => Some(Tokens::CloseBraces),
            '{' => Some(Tokens::OpenBraces),
            '!' => Some(Tokens::Not),
            ';' => Some(Tokens::LineBreak),
            ',' => Some(Tokens::Comma),
            ':' => Some(Tokens::Colon),
            '%' => Some(Tokens::Modulo),

            _ => None,
        };

        if let Some(single_char_token) = single_char {
            if !string_buffer.trim().is_empty() {
                let token = match_multi_character_expression(string_buffer.clone());

                token_list.push(token);
            }

            token_list.push(single_char_token);

            string_buffer.clear();
        } else if current_char == '#' {
            let mut comment_buffer = String::new();

            let mut comment_idx = char_idx + 1;

            loop {
                let quote_char = char_list[comment_idx];
                if quote_char == '\n' {
                    token_list.push(Tokens::Comment(comment_buffer.trim().to_string()));

                    char_idx = comment_idx + 1;

                    break;
                }

                comment_buffer.push(quote_char);

                comment_idx += 1;
            }

            continue;
        } else if current_char == '"' {
            let mut quotes_buffer = String::new();

            let mut quote_idx = char_idx + 1;

            loop {
                let quote_char = char_list.get(quote_idx);
                
                match quote_char {
                    Some(quote_char) => {
                        if *quote_char == '"' {
                            token_list.push(Tokens::Literal(Type::String(quotes_buffer)));
        
                            char_idx = quote_idx + 1;
        
                            break;
                        }
        
                        quotes_buffer.push(*quote_char);
        
                        quote_idx += 1;
                    },
                    // If there are no more tokens left and we are still in the quote
                    None => {
                        return Err(ParserError::SyntaxError);
                    },
                }
            }

            continue;
        } else if current_char == '=' {
            if let Some(next_char) = char_list.get(char_idx + 1) {
                if *next_char == '=' {
                    token_list.push(Tokens::Equals);

                    char_idx += 2;
                    continue;
                }
            }

            token_list.push(Tokens::SetValue);
        } else if current_char == '&' {
            if let Some(next_char) = char_list.get(char_idx + 1) {
                if *next_char == '&' {
                    token_list.push(Tokens::And);

                    char_idx += 2;
                    continue;
                }
            }

            token_list.push(Tokens::BitAnd);
        } else if current_char == '>' {
            if let Some(next_char) = char_list.get(char_idx + 1) {
                if *next_char == '>' {
                    token_list.push(Tokens::BitRight);

                    char_idx += 2;
                    continue;
                }
            }

            token_list.push(Tokens::Bigger);
        } else if current_char == '<' {
            if let Some(next_char) = char_list.get(char_idx + 1) {
                if *next_char == '<' {
                    token_list.push(Tokens::BitLeft);

                    char_idx += 2;

                    continue;
                }
            }

            token_list.push(Tokens::Smaller);
        } else if current_char == '|' {
            if let Some(next_char) = char_list.get(char_idx + 1) {
                if *next_char == '|' {
                    token_list.push(Tokens::Or);

                    char_idx += 2;
                    continue;
                }
            }

            token_list.push(Tokens::BitOr);
        } else if current_char == ' ' && !string_buffer.trim().is_empty() {
            let token = match_multi_character_expression(string_buffer.clone());

            token_list.push(token);

            string_buffer.clear();
        } else if current_char != ' ' {
            string_buffer.push(current_char);
        } else if current_char == '-' {
            // If the last token was a number we know that we are subtracting
            if matches!(token_list[token_list.len() - 1], Tokens::Literal(_)) {
                token_list.push(Tokens::Subtraction);
            }
            // If the last token wasnt a number we know that we are defining a negative number
            else {
                string_buffer.push(current_char);
            }
        }

        char_idx += 1;
    }

    Ok(token_list)
}

fn match_multi_character_expression(string_buffer: String) -> Tokens {
    let trimmed_string = string_buffer.trim();

    match trimmed_string {
        "int" => Tokens::TypeDefinition(crate::app::type_system::TypeDiscriminants::I32),
        "string" => Tokens::TypeDefinition(crate::app::type_system::TypeDiscriminants::String),
        "uint" => Tokens::TypeDefinition(crate::app::type_system::TypeDiscriminants::U32),
        "float" => Tokens::TypeDefinition(crate::app::type_system::TypeDiscriminants::F32),
        "uintsmall" => Tokens::TypeDefinition(crate::app::type_system::TypeDiscriminants::U8),
        "void" => Tokens::TypeDefinition(crate::app::type_system::TypeDiscriminants::Void),
        "bool" => Tokens::TypeDefinition(crate::app::type_system::TypeDiscriminants::Boolean),
        "==" => Tokens::Equals,
        ">=" => Tokens::EqBigger,
        "<=" => Tokens::EqSmaller,
        "&&" => Tokens::And,
        "||" => Tokens::Or,
        "if" => Tokens::If,
        "+=" => Tokens::SetValueAddition,
        "-=" => Tokens::SetValueSubtraction,
        "*=" => Tokens::SetValueMultiplication,
        "/=" => Tokens::SetValueDivision,
        "%=" => Tokens::SetValueModulo,
        "function" => Tokens::Function,
        "return" => Tokens::Return,

        _ => eval_constant_definition(trimmed_string.to_string()),
    }
}

// I guess this works too lol
pub fn eval_constant_definition(raw_string: String) -> Tokens {
    if raw_string.parse::<u8>().is_ok() || raw_string.parse::<u32>().is_ok() || raw_string.parse::<f32>().is_ok() || raw_string.parse::<i32>().is_ok() {
        return Tokens::UnparsedLiteral(raw_string);
    } else {
        return Tokens::Identifier(raw_string);
    }
}

pub fn parse_tokens(
    tokens: Vec<Tokens>,
) -> Result<HashMap<String, UnparsedFunctionDefinition>, ParserError> {
    let mut token_idx = 0;
    let mut function_list: HashMap<String, UnparsedFunctionDefinition> = HashMap::new();

    while token_idx < tokens.len() {
        let current_token = tokens[token_idx].clone();

        if current_token == Tokens::Function {
            if let Tokens::Identifier(function_name) = tokens[token_idx + 1].clone() {
                if tokens[token_idx + 2] == Tokens::OpenBracket {
                    let (bracket_close_idx, args) = parse_function_argument_tokens(&tokens[token_idx + 3..], token_idx)?;
                    
                    token_idx += bracket_close_idx + 3;

                    if tokens[token_idx + 1] == Tokens::Colon {
                        if let Tokens::TypeDefinition(return_type) = tokens[token_idx + 2] {
                            if tokens[token_idx + 3] == Tokens::OpenBraces {
                                // Create a varable which stores the level of braces we are in
                                let mut brace_layer_counter = 1;

                                // Get the slice of the list which may contain the brackets' scope
                                let tokens_slice = &tokens[token_idx + 4..];

                                // Create an index which indexes the tokens slice
                                let mut token_braces_idx = 0;

                                // Create a list which contains all the tokens inside the two brackets
                                let mut braces_contains: Vec<Tokens> = vec![];

                                // Find the scope of this function
                                loop {
                                    // If a bracket is closed the layer counter should be incremented
                                    if tokens_slice[token_braces_idx] == Tokens::OpenBraces {
                                        brace_layer_counter += 1;
                                    }
                                    // If a bracket is closed the layer counter should be decreased
                                    else if tokens_slice[token_braces_idx] == Tokens::CloseBraces
                                    {
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
                                        inner: dbg!(braces_contains),
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

                    return Err(ParserError::InvalidFunctionDefinition);
                } else {
                    return Err(ParserError::InvalidFunctionDefinition);
                }
            }
        }

        token_idx += 1;
    }

    Ok(function_list)
}

fn parse_function_argument_tokens(tokens: &[Tokens], token_idx: usize) -> Result<(usize, HashMap<String, TypeDiscriminants>), ParserError> {
    let bracket_closing_idx = find_closing_bracket(tokens).map_err(|_| ParserError::InvalidFunctionDefinition)?;

    let mut args = HashMap::new();

    if bracket_closing_idx != 0 {
        args = parse_function_args(&tokens[token_idx..token_idx + bracket_closing_idx])?;
    }

    Ok((bracket_closing_idx, args))
}

fn parse_function_args(
    token_list: &[Tokens],
) -> Result<HashMap<String, TypeDiscriminants>, ParserError> {
    // Create a list of args which the function will take, we will return this later
    let mut args: HashMap<String, TypeDiscriminants> = HashMap::new();

    // Create an index which will iterate through the tokens
    let mut args_idx = 0;

    // Iter until we find a CloseBracket: ")"
    // This will be the end of the function's arguments
    while args_idx <= token_list.len() {
        // Match the signature of an argument
        // Get the variable's name
        // If the token is an identifier then we know that this is a variable name
        // If the token is a colon then we know that this is a type definition
        if let Tokens::Identifier(var_name) = token_list[args_idx].clone() {
            // Match the colon from the signature, to ensure correct signaure
            if token_list[args_idx + 1] == Tokens::Colon {
                // Get the type of the argument
                if let Tokens::TypeDefinition(var_type) = token_list[args_idx + 2] {
                    // Store the argument in the HashMap
                    args.insert(var_name, var_type);

                    // Increment the idx
                    args_idx += 4;

                    // Countinue the loop
                    continue;
                }
            }
        }

        // If the pattern didnt match the tokens return an error
        return Err(ParserError::InvalidFunctionDefinition);
    }

    Ok(args)
}

pub fn parse_functions(
    unparsed_functions: Arc<HashMap<String, UnparsedFunctionDefinition>>,
) -> Result<HashMap<String, FunctionDefinition>, ParserError> {
    let mut parsed_functions = HashMap::new();

    for (fn_name, unparsed_function) in unparsed_functions.clone().iter() {
        let function_definition = FunctionDefinition {
            inner: parse_function(
                unparsed_function.inner.clone(),
                unparsed_functions.clone(),
                unparsed_function.function_sig.args.clone(),
            )?,
            function_sig: unparsed_function.function_sig.clone(),
        };

        parsed_functions.insert(fn_name.clone(), function_definition);
    }

    Ok(parsed_functions)
}

fn parse_function(
    tokens: Vec<Tokens>,
    function_signatures: Arc<HashMap<String, UnparsedFunctionDefinition>>,
    this_function_args: HashMap<String, TypeDiscriminants>,
) -> Result<Vec<ParsedTokens>, ParserError> {
    let mut token_idx = 0;

    let mut parsed_tokens: Vec<ParsedTokens> = Vec::new();

    let mut variable_scope: HashMap<String, TypeDiscriminants> = this_function_args;

    if tokens.len() == 0 {
        return Ok(vec![]);
    }

    while token_idx <= tokens.len() - 1 {
        let current_token = tokens[token_idx].clone();

        if let Tokens::TypeDefinition(var_type) = current_token {
            if let Tokens::Identifier(var_name) = tokens[token_idx + 1].clone() {
                parsed_tokens.push(ParsedTokens::NewVariable((var_name.clone(), var_type.into())));

                variable_scope.insert(var_name.clone(), var_type);

                if tokens[token_idx + 2] == Tokens::LineBreak
                    || tokens[token_idx + 2] == Tokens::SetValue
                {
                    token_idx += 1;

                    continue;
                } else {
                    return Err(ParserError::SyntaxError);
                }
            } else {
                return Err(ParserError::SyntaxError);
            }
        } else if let Tokens::Identifier(ident_name) = current_token {
            // If the variable exists in the current scope
            if let Some(variable_type) = variable_scope.get(&ident_name)
            {
                match dbg!(&tokens[token_idx + 1]) {
                    Tokens::SetValue => {
                        let line_break_idx = tokens.iter().skip(token_idx + 1).position(|token| *token == Tokens::LineBreak).ok_or_else(|| ParserError::SyntaxError)? + token_idx + 1;

                        let selected_tokens = &tokens[token_idx + 1..line_break_idx];
                        
                        token_idx += selected_tokens.len() + 1;
                        
                        parse_set_value(selected_tokens, &mut parsed_tokens, function_signatures.clone(), &variable_scope, *variable_type, ident_name.clone())?;
                    }
                    
                    Tokens::SetValueAddition => {}
                    Tokens::SetValueSubtraction => {}
                    Tokens::SetValueDivision => {}
                    Tokens::SetValueMultiplication => {}
                    Tokens::SetValueModulo => {}

                    _ => {
                        println!("UNIMPLEMENTED FUNCTION: {}", tokens[token_idx + 1]);
                    }
                }
            } else if let Some(function_sig) = function_signatures.get(&ident_name) {
                // If after the function name the first thing isnt a `(` return a syntax error.
                if tokens[token_idx + 1] != Tokens::OpenBracket {
                    return Err(ParserError::SyntaxError);
                }
                
                let bracket_start_slice = &tokens[token_idx + 2..];

                let bracket_idx = find_closing_bracket(bracket_start_slice)? + token_idx;                    

                let (variables_passed, jumped_idx) = parse_function_call(
                    &tokens[token_idx + 2..bracket_idx + 2],
                    variable_scope.clone(),
                    function_sig.function_sig.args.clone(),
                )?;

                parsed_tokens.push(ParsedTokens::FunctionCall((function_sig.function_sig.clone(), ident_name), variables_passed));

                token_idx += jumped_idx;
            } else {
                return Err(ParserError::VariableNotFound(ident_name));
            }
        }

        token_idx += 1;
    }

    Ok(parsed_tokens)
}

fn find_closing_bracket(bracket_start_slice: &[Tokens]) -> Result<usize, ParserError> {
    let mut bracket_idx = 0;
    let mut bracket_layer_counter = 1;

    loop {
        if bracket_start_slice.len() <= bracket_idx {
            return Err(ParserError::SyntaxError);
        }
        
        if bracket_start_slice[bracket_idx] == Tokens::OpenBracket {
            bracket_layer_counter += 1;
        }
        else if bracket_start_slice[bracket_idx] == Tokens::CloseBracket {
            bracket_layer_counter -= 1;
        }

        if bracket_layer_counter == 0 {
            break;
        }

        bracket_idx += 1;
    }

    Ok(bracket_idx)
}

fn parse_function_call(
    tokens: &[Tokens],
    variable_scope: HashMap<String, TypeDiscriminants>,
    function_args: HashMap<String, TypeDiscriminants>,
) -> Result<(Vec<ParsedTokens>, usize), ParserError> {
    let mut tokens_idx = 0;

    // Arguments which will passed in to the function
    let mut arguments: Vec<ParsedTokens> = vec![];

    while tokens_idx < tokens.len() {
        let current_token = tokens[tokens_idx].clone();

        if let Tokens::Identifier(arg_name) = current_token.clone() {
            if Tokens::SetValue == tokens[tokens_idx + 1] {
                let current_arg = tokens[tokens_idx + 2].clone();

                let argument_type = function_args
                    .get(&arg_name)
                    .ok_or(ParserError::ArgumentError(arg_name))?;

                if let Tokens::Identifier(var_name) = current_arg {
                    let var_type = variable_scope
                        .get(&var_name)
                        .ok_or_else(|| ParserError::VariableNotFound(var_name.clone()))?;

                    // If the types match and the argument's name also matches then we can store the variable's name with the argument's name.
                    if argument_type == var_type {
                        tokens_idx += 4;

                        arguments.push(ParsedTokens::VariableReference(var_name));
                        
                        continue;
                    } else {
                        return Err(ParserError::TypeError(*var_type, *argument_type));
                    }
                } else if let Tokens::Literal(literal) = current_arg {
                    let literal_type = literal.discriminant();

                    if *argument_type == literal_type {
                        tokens_idx += 4;

                        arguments.push(ParsedTokens::Literal(literal));
                        
                        continue;
                    } else {
                        return Err(ParserError::TypeError(literal_type, *argument_type));
                    }
                }
            } else {
                return Err(ParserError::SyntaxError);
            }
        } else if Tokens::CloseBraces == current_token {
            break;
        } else {
            return Err(ParserError::SyntaxError);
        }

    }

    Ok((arguments, tokens_idx))
}

pub fn parse_set_value(tokens: &[Tokens], parsed_tokens: &mut Vec<ParsedTokens>, function_signatures: Arc<HashMap<String, UnparsedFunctionDefinition>>, variable_scope: &HashMap<String, TypeDiscriminants>, variable_type: TypeDiscriminants,  variable_name: String) -> Result<(), ParserError> {
    let mut token_idx = 1;

    while token_idx < tokens.len() {
        let current_token = &tokens.get(token_idx).ok_or_else(|| ParserError::SyntaxError)?;
        match current_token {
            Tokens::Literal(literal) => {
                if variable_type != literal.discriminant() {
                    return Err(ParserError::TypeError(literal.discriminant(), variable_type));
                }
                
                parsed_tokens.push(ParsedTokens::SetValue(variable_name.clone(), Box::new(ParsedTokens::Literal(literal.clone()))));
            },
            Tokens::UnparsedLiteral(raw_string) => {
                let parsed_literal = unparsed_const_to_typed_literal(raw_string.clone(), variable_type)?;
            
                parsed_tokens.push(ParsedTokens::SetValue(variable_name.clone(), Box::new(ParsedTokens::Literal(parsed_literal.clone()))));
            },
            Tokens::Identifier(ident_name) => {
                if let Some(function) = function_signatures.get(&*ident_name) {
                    if variable_type != function.function_sig.return_type {
                        return Err(ParserError::TypeError(function.function_sig.return_type, variable_type));
                    }
    
                    let bracket_idx = find_closing_bracket(&tokens[token_idx..])? + token_idx;
    
                    let (variables_passed, jumped_idx) = parse_function_call(
                        &tokens[token_idx..bracket_idx + 2],
                        variable_scope.clone(),
                        function.function_sig.args.clone(),
                    )?;
    
                    token_idx += jumped_idx;
                    
                    parsed_tokens.push(ParsedTokens::SetValue(variable_name.clone(), Box::new(ParsedTokens::FunctionCall((function.function_sig.clone(), ident_name.clone()), variables_passed))));
                } else if let Some(variable) = variable_scope.get(&*ident_name) {
                    if variable_type != *variable {
                        return Err(ParserError::TypeError(*variable, variable_type));
                    }
    
                    parsed_tokens.push(ParsedTokens::SetValue(variable_name.clone(), Box::new(ParsedTokens::VariableReference(ident_name.clone()))));
                }
            },
            Tokens::Quote(quote) => {
                parsed_tokens.push(ParsedTokens::SetValue(variable_name.clone(), Box::new(ParsedTokens::Literal(Type::String(quote.clone())))));
            },
    
            Tokens::Addition => {
                let parsed_token = parsed_tokens.last_mut();

                let next_token = tokens.get(token_idx + 1).ok_or_else(|| ParserError::SyntaxError)?;

                if let Some(ParsedTokens::Addition(lhs, rhs)) = parsed_token {
                    *lhs = Box::new(ParsedTokens::Addition(lhs.clone(), rhs.clone()));

                    *rhs = match next_token {
                        Tokens::Literal(literal) => {
                            let literal_type = literal.discriminant();

                            if literal_type != variable_type {
                                return Err(ParserError::TypeError(literal_type, variable_type));
                            }

                            Box::new(ParsedTokens::Literal(literal.clone()))
                        },
                        Tokens::UnparsedLiteral(raw_string) => {
                            let literal = unparsed_const_to_typed_literal(raw_string.clone(), variable_type)?;

                            let literal_type = literal.discriminant();
                            
                            if literal_type != variable_type {
                                return Err(ParserError::TypeError(literal_type, variable_type));
                            }

                            Box::new(ParsedTokens::Literal(literal))
                        },
                        Tokens::Identifier(identifier) => {
                            if let Some(var_type) = variable_scope.get(&*identifier) {
                                if variable_type != *var_type {
                                    return Err(ParserError::TypeError(*var_type, variable_type));
                                }

                                Box::new(ParsedTokens::VariableReference(identifier.clone()))
                            }
                            else if let Some(fn_sig) = function_signatures.get(&*identifier) {
                                // If after the function name the first thing isnt a `(` return a syntax error.
                                if tokens[token_idx + 1] != Tokens::OpenBracket {
                                    return Err(ParserError::SyntaxError);
                                }
                                
                                let bracket_start_slice = &tokens[token_idx + 2..];

                                let bracket_idx = find_closing_bracket(bracket_start_slice)? + token_idx;                    

                                let (variables_passed, jumped_idx) = parse_function_call(
                                    &tokens[token_idx + 2..bracket_idx + 2],
                                    variable_scope.clone(),
                                    fn_sig.function_sig.args.clone(),
                                )?;

                                token_idx += jumped_idx;

                                Box::new(ParsedTokens::FunctionCall((fn_sig.function_sig.clone(), identifier.clone()), variables_passed))
                            }
                            else {
                                return Err(ParserError::VariableNotFound(identifier.clone()));
                            }
                        },

                        _ => {
                            token_idx += 1;
                            continue;
                        },
                    };
                }
                else {
                    let last_token = tokens.get(token_idx - 1).ok_or_else(|| ParserError::SyntaxError)?;
                    let next_token = tokens.get(token_idx + 1).ok_or_else(|| ParserError::SyntaxError)?;

                    let lhs = match last_token {
                        Tokens::Literal(literal) => {
                            let literal_type = literal.discriminant();

                            if literal_type != variable_type {
                                return Err(ParserError::TypeError(literal_type, variable_type));
                            }

                            Box::new(ParsedTokens::Literal(literal.clone()))
                        },
                        Tokens::UnparsedLiteral(raw_string) => {
                            let literal = unparsed_const_to_typed_literal(raw_string.clone(), variable_type)?;

                            let literal_type = literal.discriminant();
                            
                            if literal_type != variable_type {
                                return Err(ParserError::TypeError(literal_type, variable_type));
                            }

                            Box::new(ParsedTokens::Literal(literal))
                        },
                        Tokens::Identifier(identifier) => {
                            if let Some(var_type) = variable_scope.get(&*identifier) {
                                if variable_type != *var_type {
                                    return Err(ParserError::TypeError(*var_type, variable_type));
                                }

                                Box::new(ParsedTokens::VariableReference(identifier.clone()))
                            }
                            else if let Some(fn_sig) = function_signatures.get(&*identifier) {
                                // If after the function name the first thing isnt a `(` return a syntax error.
                                if tokens[token_idx + 1] != Tokens::OpenBracket {
                                    return Err(ParserError::SyntaxError);
                                }
                                
                                let bracket_start_slice = &tokens[token_idx + 2..];

                                let bracket_idx = find_closing_bracket(bracket_start_slice)? + token_idx;                    

                                let (variables_passed, jumped_idx) = parse_function_call(
                                    &tokens[token_idx + 2..bracket_idx + 2],
                                    variable_scope.clone(),
                                    fn_sig.function_sig.args.clone(),
                                )?;

                                token_idx += jumped_idx;

                                Box::new(ParsedTokens::FunctionCall((fn_sig.function_sig.clone(), identifier.clone()), variables_passed))
                            }
                            else {
                                return Err(ParserError::VariableNotFound(identifier.clone()));
                            }
                        },

                        _ => {
                            token_idx += 1;
                            continue;
                        },
                    };

                    let rhs = match next_token {
                        Tokens::Literal(literal) => {
                            let literal_type = literal.discriminant();

                            if literal_type != variable_type {
                                return Err(ParserError::TypeError(literal_type, variable_type));
                            }

                            Box::new(ParsedTokens::Literal(literal.clone()))
                        },
                        Tokens::UnparsedLiteral(raw_string) => {
                            let literal = unparsed_const_to_typed_literal(raw_string.clone(), variable_type)?;

                            let literal_type = literal.discriminant();
                            
                            if literal_type != variable_type {
                                return Err(ParserError::TypeError(literal_type, variable_type));
                            }

                            Box::new(ParsedTokens::Literal(literal))
                        },
                        Tokens::Identifier(identifier) => {
                            if let Some(var_type) = variable_scope.get(&*identifier) {
                                if variable_type != *var_type {
                                    return Err(ParserError::TypeError(*var_type, variable_type));
                                }

                                Box::new(ParsedTokens::VariableReference(identifier.clone()))
                            }
                            else if let Some(fn_sig) = function_signatures.get(&*identifier) {
                                // If after the function name the first thing isnt a `(` return a syntax error.
                                if tokens[token_idx + 1] != Tokens::OpenBracket {
                                    return Err(ParserError::SyntaxError);
                                }
                                
                                let bracket_start_slice = &tokens[token_idx + 2..];

                                let bracket_idx = find_closing_bracket(bracket_start_slice)? + token_idx;                    

                                let (variables_passed, jumped_idx) = parse_function_call(
                                    &tokens[token_idx + 2..bracket_idx + 2],
                                    variable_scope.clone(),
                                    fn_sig.function_sig.args.clone(),
                                )?;

                                token_idx += jumped_idx;

                                Box::new(ParsedTokens::FunctionCall((fn_sig.function_sig.clone(), identifier.clone()), variables_passed))
                            }
                            else {
                                return Err(ParserError::VariableNotFound(identifier.clone()));
                            }
                        },

                        _ => {
                            token_idx += 1;
                            continue;
                        },
                    };

                    let last_token = parsed_tokens.last_mut().unwrap();

                    *last_token = ParsedTokens::Addition(lhs, rhs);

                    token_idx += 3;

                    continue;
                }
            }
            Tokens::Subtraction => {
                
            }
            Tokens::Multiplication => {
                
            }
            Tokens::Division => {
                
            }

            _ => {
                
            },
        }
        token_idx += 1;
    }

    

    Ok(())
}