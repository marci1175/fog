use std::{collections::HashMap, process::id, sync::Arc};

use strum::IntoDiscriminant;

use crate::app::type_system::{Type, TypeDiscriminants};

use super::{
    error::ParserError,
    types::{FunctionDefinition, ParsedTokens, Tokens, UnparsedFunctionDefinition},
};

pub fn parse_code(raw_string: String) -> Vec<Tokens> {
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
                let quote_char = char_list[quote_idx];
                if quote_char == '"' {
                    token_list.push(Tokens::Quote(quotes_buffer));

                    char_idx = quote_idx + 1;

                    break;
                }

                quotes_buffer.push(quote_char);

                quote_idx += 1;
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
            if matches!(token_list[token_list.len() - 1], Tokens::Const(_)) {
                token_list.push(Tokens::Subtraction);
            }
            // If the last token wasnt a number we know that we are defining a negative number
            else {
                string_buffer.push(current_char);
            }
        }

        char_idx += 1;
    }

    token_list
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
    if let Ok(const_eval_f32) = raw_string.parse::<u8>() {
        Tokens::Const(Type::U8(const_eval_f32))
    }
    else if let Ok(const_eval_f32) = raw_string.parse::<u32>() {
        return Tokens::Const(Type::U32(const_eval_f32));
    }
    else if let Ok(const_eval_f32) = raw_string.parse::<i32>() {
        return Tokens::Const(Type::I32(const_eval_f32));
    }
    else if let Ok(const_eval_f32) = raw_string.parse::<f32>() {
        return Tokens::Const(Type::F32(const_eval_f32));
    }
    else {
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
                    let bracket_close_idx = tokens[token_idx + 2..].iter().position(|token| *token == Tokens::CloseBracket).ok_or(ParserError::InvalidFunctionDefinition)? + (token_idx + 2);
                    
                    let args = parse_function_args(&tokens[token_idx + 3..bracket_close_idx])?;
                    
                    if tokens[bracket_close_idx + 1] == Tokens::Colon {
                        if let Tokens::TypeDefinition(return_type) = tokens[bracket_close_idx + 2] {
                            if tokens[bracket_close_idx + 3] == Tokens::OpenBraces {
                                // Create a varable which stores the level of braces we are in
                                let mut brace_layer_counter = 1;

                                // Get the slice of the list which may contain the brackets' scope
                                let tokens_slice = &tokens[bracket_close_idx + 4..];
                                
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
                                    else if tokens_slice[token_braces_idx] == Tokens::CloseBraces {
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
                                function_list.insert(function_name, UnparsedFunctionDefinition { args, inner: braces_contains, return_type });

                                // Set the iterator index
                                token_idx = bracket_close_idx + 3 + braces_contains_len + 2;

                                // Countinue with the loop
                                continue;
                            }
                        }
                    }
                    
                    return Err(ParserError::InvalidFunctionDefinition);
                }
                else {
                    return Err(ParserError::InvalidFunctionDefinition);
                }
            }
        }

        token_idx += 1;
    }

    Ok(function_list)
}

fn parse_function_args(token_list: &[Tokens]) -> Result<HashMap<String, TypeDiscriminants>, ParserError> {
    // Create a list of args which the function will take, we will return this later
    let mut args: HashMap<String, TypeDiscriminants> = HashMap::new();

    let mut args_idx = 0;

    //Iter until we find a CloseBracket: ")"
    while args_idx < token_list.len() {
        // Match the signature of an argument
        // Get the variable's name
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

pub fn parse_functions(unparsed_functions: Arc<HashMap<String, UnparsedFunctionDefinition>>) -> Result<HashMap<String, FunctionDefinition>, ParserError> {
    let mut parsed_functions = HashMap::new();

    for (fn_name, unparsed_function) in unparsed_functions.clone().iter() {
        let function_definition = FunctionDefinition { args: unparsed_function.args.clone(), inner: parse_function(unparsed_function.inner.clone(), unparsed_functions.clone(), unparsed_function.args.clone())?, return_type: unparsed_function.return_type };
    
        parsed_functions.insert(fn_name.clone(), function_definition);
    }

    Ok(parsed_functions)
}

fn parse_function(tokens: Vec<Tokens>, function_signatures: Arc<HashMap<String, UnparsedFunctionDefinition>>, this_function_args: HashMap<String, TypeDiscriminants>) -> Result<Vec<ParsedTokens>, ParserError> {
    let mut token_idx = 0;

    let mut parsed_tokens: Vec<ParsedTokens> = Vec::new();

    let mut variable_scope: HashMap<String, TypeDiscriminants> = HashMap::new();

    while token_idx < tokens.len() {
        let current_token = tokens[token_idx].clone();
        
        if let Tokens::TypeDefinition(var_type) = current_token {
            if let Tokens::Identifier(var_name) = tokens[token_idx + 1].clone() {
                parsed_tokens.push(ParsedTokens::Variable((var_name.clone(), var_type.into())));
        
                variable_scope.insert(var_name.clone(), var_type);
        
                if tokens[token_idx + 2] == Tokens::LineBreak || tokens[token_idx + 2] == Tokens::SetValue {
                    token_idx += 1;
        
                    continue;
                }
                else {
                    return Err(ParserError::SyntaxError);                    
                }
            }
            else {
                return Err(ParserError::SyntaxError);
            }
        }
        else if let Tokens::Identifier(ident_name) = current_token {
            // If the variable exists in the current scope
            if variable_scope.contains_key(&ident_name) || this_function_args.contains_key(&ident_name) {
                match tokens[token_idx + 1] {
                    Tokens::SetValue => {
                        // If there was a constant
                        let current_token = tokens[token_idx + 2].clone();
                        
                        if let Tokens::Const(const_var) = tokens[token_idx + 2].clone() {
                            let variable_type = variable_scope.get(&ident_name).ok_or_else(|| ParserError::VariableNotFound(ident_name.clone()))?;
                            if const_var.discriminant() == *variable_type {
                                parsed_tokens.push(ParsedTokens::SetValue((ident_name, const_var)));
                            }
                            else {
                                return Err(ParserError::VariableTypeMismatch(ident_name, *variable_type, const_var.discriminant()));
                            }
                        }
                        // Else if there is a function call
                        else if let Tokens::Identifier(fn_name) = current_token {
                            
                        }
                    }
                    Tokens::SetValueAddition => {},
                    Tokens::SetValueSubtraction => {},
                    Tokens::SetValueDivision => {},
                    Tokens::SetValueMultiplication => {},
                    Tokens::SetValueModulo => {},

                    _ => {
                        println!("UNIMPLEMENTED FUNCTION: {}", tokens[token_idx + 1]);
                    },
                }
    
                if tokens[token_idx + 3] == Tokens::LineBreak {
                    token_idx += 4;
    
                    continue;
                }
                else {
                    return Err(ParserError::SyntaxError);                    
                }
            }
            else if let Some(function_sig) = function_signatures.get(&ident_name) {
                // If after the function name the first thing isnt a `(` return a syntax error.
                if tokens[token_idx + 1] != Tokens::OpenBracket {
                    return Err(ParserError::SyntaxError);
                }

                let variables_passed = parse_function_call(&tokens[token_idx + 2..], variable_scope.clone(), function_sig.args.clone())?;
                
                // parsed_tokens.push(ParsedTokens::FunctionCall());
            }
            else {
                return Err(ParserError::VariableNotFound(ident_name));
            }
        }

        token_idx += 1;
    }

    Ok(parsed_tokens)
}

fn parse_function_call(tokens: &[Tokens], variable_scope: HashMap<String, TypeDiscriminants>, function_args: HashMap<String, TypeDiscriminants>) -> Result<Vec<String>, ParserError> {
    let mut tokens_idx = 0;

    // Variables which will passed in to the function
    let mut variable_names: Vec<String> = vec![];

    while tokens_idx < tokens.len() {
        let current_token = tokens[tokens_idx].clone();

        if let Tokens::Identifier(arg_name) = current_token {
            if Tokens::Comma == tokens[tokens_idx + 1] {
                let current_arg = tokens[tokens_idx + 2].clone();
                if let Tokens::Identifier(var_name) = current_arg {
                    let var_type = variable_scope.get(&var_name).ok_or_else(|| ParserError::VariableNotFound(var_name.clone()))?;
        
                    let argument_type = dbg!(&function_args).get(&arg_name).ok_or(ParserError::ArgumentError(arg_name))?;

                    // If the types match and the argument's name also matches then we can store the variable's name with the argument's name.
                    if argument_type == var_type {
                        variable_names.push(var_name);
                    }
                    else {
                        return Err(ParserError::TypeError(*var_type, *argument_type));
                    }
                }
                else if let Tokens::Const(const_type) = current_arg {
                    unimplemented!("Implement using consts for functions!")
                }
            }
            else {
                return Err(ParserError::SyntaxError);
            }
        }
        else if Tokens::CloseBraces == current_token {
            break;
        }
        else {
            return Err(ParserError::SyntaxError);
        }
        


        tokens_idx += 1;
    }

    Ok(variable_names)
}