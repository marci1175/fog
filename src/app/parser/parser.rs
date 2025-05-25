use crate::app::type_system::type_system::{
    TypeDiscriminants, unparsed_const_to_typed_literal_unsafe,
};
use anyhow::Result;
use indexmap::IndexMap;
use std::{collections::HashMap, sync::Arc};
use strum::IntoDiscriminant;

use super::{
    error::{ParserError, SyntaxError},
    parse_functions::{self, create_signature_table, parse_functions},
    types::{
        CustomType, FunctionDefinition, FunctionSignature, ParsedToken, Token,
        UnparsedFunctionDefinition,
    },
};

#[derive(Debug, Clone)]
pub struct ParserState {
    tokens: Vec<Token>,

    function_table: IndexMap<String, FunctionDefinition>,

    custom_items: Arc<IndexMap<String, CustomType>>,

    imported_functions: Arc<HashMap<String, FunctionSignature>>,
}

impl ParserState {
    pub fn parse_tokens(&mut self) -> Result<()> {
        println!("Creating signature table...");
        // Create user defined signature table
        // Create an import table which can be used later by other functions
        let (unparsed_functions, source_imports, mut external_imports, custom_items) =
            create_signature_table(self.tokens.clone())?;

        let custom_items: Arc<IndexMap<String, CustomType>> = Arc::new(custom_items);

        // Extend the list of external imports with source imports aka imports from Fog source files.
        external_imports.extend(
            source_imports
                .iter()
                .map(|(fn_name, fn_def)| (fn_name.clone(), fn_def.function_sig.clone())),
        );

        let imports = Arc::new(external_imports);

        // Copy the the HashMap to this field
        self.imported_functions = imports.clone();

        println!("Parsing functions...");
        // Set the function table field of this struct
        self.function_table = parse_functions(
            Arc::new(unparsed_functions),
            imports.clone(),
            custom_items.clone(),
        )?;

        // Extend function table with imported functions. (Imported from Fog source code)
        self.function_table.extend(source_imports);

        self.custom_items = custom_items.clone();

        Ok(())
    }

    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            function_table: IndexMap::new(),
            imported_functions: Arc::new(HashMap::new()),
            custom_items: Arc::new(IndexMap::new()),
        }
    }

    pub fn function_table(&self) -> &IndexMap<String, FunctionDefinition> {
        &self.function_table
    }

    pub fn imported_functions(&self) -> &HashMap<String, FunctionSignature> {
        &self.imported_functions
    }
}

/// Pass in 0 for the `open_paren_count` if you're searching for the very next closing token on the same level.
pub fn find_closing_paren(paren_start_slice: &[Token], open_paren_count: usize) -> Result<usize> {
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
            }
            _ => continue,
        }
    }

    Err(ParserError::SyntaxError(super::error::SyntaxError::LeftOpenParentheses).into())
}

/// Pass in 0 for the `open_braces_count` if you're searching for the very next closing token on the same level.
pub fn find_closing_braces(
    braces_start_slice: &[Token],
    open_braces_count: usize,
) -> Result<usize> {
    let mut braces_layer_counter = 1;
    let iter = braces_start_slice.iter().enumerate();

    for (idx, token) in iter {
        match token {
            Token::OpenParentheses => braces_layer_counter += 1,
            Token::CloseBraces => {
                braces_layer_counter -= 1;
                if braces_layer_counter == open_braces_count {
                    return Ok(idx);
                }
            }
            _ => continue,
        }
    }

    Err(ParserError::SyntaxError(super::error::SyntaxError::LeftOpenParentheses).into())
}

/// This is a top level implementation for `parse_token_as_value`
pub fn parse_value(
    tokens: &[Token],
    function_signatures: Arc<IndexMap<String, UnparsedFunctionDefinition>>,
    variable_scope: &IndexMap<String, TypeDiscriminants>,
    variable_type: TypeDiscriminants,
    function_imports: Arc<HashMap<String, FunctionSignature>>,
    custom_items: Arc<IndexMap<String, CustomType>>,
) -> Result<(ParsedToken, usize)> {
    let mut token_idx = 0;

    let mut parsed_token: Option<ParsedToken> = None;

    while token_idx < tokens.len() {
        let current_token = &tokens.get(token_idx).ok_or({
            ParserError::SyntaxError(
                crate::app::parser::error::SyntaxError::InvalidMathematicalExpressionDefinition,
            )
        })?;

        // Please note that we are not looking at values by themselves, except in SetValue where we take the next token.
        match current_token {
            // If any mathematical expression is present in the tokens
            Token::Addition | Token::Subtraction | Token::Multiplication | Token::Division => {
                // Grab the next token after the mathematical expression
                let next_token = &tokens.get(token_idx + 1).ok_or(ParserError::SyntaxError(
                    crate::app::parser::error::SyntaxError::InvalidMathematicalExpressionDefinition,
                ))?;

                // If we have parsed something already move it to the left-hand side of the mathematical expression
                // Add the new parsed token to the right-hand side of the mathematical expression.
                if let Some(parsed_token) = &mut parsed_token {
                    token_idx += 1;

                    // Modify the parsed token
                    *parsed_token = ParsedToken::MathematicalExpression(
                        // Move the token to the left side
                        Box::new(parsed_token.clone()),
                        // Add the Mathematical symbol to the enum variant
                        (*current_token).clone().try_into()?,
                        // Put the new item to the right side of the expr.
                        Box::new(parse_token_as_value(
                            tokens,
                            &function_signatures,
                            variable_scope,
                            variable_type.clone(),
                            &mut token_idx,
                            next_token,
                            function_imports.clone(),
                            custom_items.clone(),
                        )?),
                    );
                } else {
                    return Err(ParserError::SyntaxError(
                        super::error::SyntaxError::InvalidMathematicalExpressionDefinition,
                    )
                    .into());
                }
            }

            // This pattern match is purely for initializing the value of the variable.
            // The ParsedToken generated by the pattern match will not be evaluated in future iterations.
            Token::UnparsedLiteral(_raw_string) => {
                let parsed_value = parse_token_as_value(
                    tokens,
                    &function_signatures,
                    variable_scope,
                    variable_type.clone(),
                    &mut token_idx,
                    current_token,
                    function_imports.clone(),
                    custom_items.clone(),
                )?;

                // Initialize parsed token with a value.
                if parsed_token.is_none() {
                    parsed_token = Some(parsed_value.clone());
                }
            }

            // This pattern match is purely for initializing the value of the variable.
            // The ParsedToken generated by the pattern match will not be evaluated in future iterations.
            Token::Literal(_) | Token::Identifier(_) | Token::OpenParentheses => {
                let parsed_value = parse_token_as_value(
                    tokens,
                    &function_signatures,
                    variable_scope,
                    variable_type.clone(),
                    &mut token_idx,
                    current_token,
                    function_imports.clone(),
                    custom_items.clone(),
                )?;

                // Initialize parsed token with a value.
                if parsed_token.is_none() {
                    parsed_token = Some(parsed_value.clone());
                }
            }

            Token::Comma | Token::CloseParentheses | Token::LineBreak => break,

            _ => unimplemented!(),
        }
    }

    Ok((
        parsed_token.ok_or(ParserError::SyntaxError(
            super::error::SyntaxError::InvalidStatementDefinition,
        ))?,
        token_idx,
    ))
}

/// Parses the next token as something that holds a value:
/// Like: FunctionCall, Literal, UnparsedLiteral
pub fn parse_token_as_value(
    // This is used to parse the function call's arguments
    tokens: &[Token],
    // Functions available
    function_signatures: &Arc<IndexMap<String, UnparsedFunctionDefinition>>,
    // Variables available
    variable_scope: &IndexMap<String, TypeDiscriminants>,
    // The variable's type which we are parsing for
    variable_type: TypeDiscriminants,
    // Universal token_idx, this sets which token we are currently parsing
    token_idx: &mut usize,
    // The token we want to evaluate
    eval_token: &Token,
    function_imports: Arc<HashMap<String, FunctionSignature>>,
    custom_items: Arc<IndexMap<String, CustomType>>,
) -> Result<ParsedToken> {
    // Match the token
    let inner_value = match eval_token {
        Token::Literal(literal) => {
            let literal_type = literal.discriminant();

            // Increment the token_idx by the tokens we have analyzed
            *token_idx += 1;

            // Check if there is an `As` keyword after the variable
            if let Some(Token::As) = tokens.get(*token_idx) {
                // If there isnt a TypeDefinition after the `As` keyword raise an error
                if let Some(Token::TypeDefinition(target_type)) = tokens.get(*token_idx + 1) {
                    // Increment the token index after checking target type
                    *token_idx += 2;

                    // Return the type casted literal
                    ParsedToken::TypeCast(
                        Box::new(ParsedToken::Literal(literal.clone())),
                        target_type.clone(),
                    )
                } else {
                    // Throw an error
                    return Err(ParserError::SyntaxError(
                        super::error::SyntaxError::AsRequiresTypeDef,
                    )
                    .into());
                }
            } else {
                // If the literal's type doesn't match the variable's type return an error
                if literal_type != variable_type {
                    return Err(ParserError::TypeError(literal_type, variable_type).into());
                }

                // Push the ParsedToken to the list
                ParsedToken::Literal(literal.clone())
            }
        }
        Token::UnparsedLiteral(unparsed_literal) => {
            // Increment the token_idx by the tokens we have analyzed
            *token_idx += 1;

            // Push the ParsedToken to the list
            let parsed_token = ParsedToken::Literal(unparsed_const_to_typed_literal_unsafe(
                unparsed_literal.clone(),
                variable_type.clone(),
            )?);

            // Check if there is an `As` keyword after the variable
            if let Some(Token::As) = tokens.get(*token_idx) {
                // If there isnt a TypeDefinition after the `As` keyword raise an error
                if let Some(Token::TypeDefinition(target_type)) = tokens.get(*token_idx + 1) {
                    // Ezt lehet hogy késöbb ki kell majd venni
                    if target_type.clone() != variable_type.clone() {
                        return Err(
                            ParserError::TypeError(target_type.clone(), variable_type).into()
                        );
                    }

                    // Increment the token index after checking target type
                    *token_idx += 2;

                    // Return the type casted literal
                    ParsedToken::TypeCast(Box::new(parsed_token), target_type.clone())
                } else {
                    // Throw an error
                    return Err(ParserError::SyntaxError(
                        super::error::SyntaxError::AsRequiresTypeDef,
                    )
                    .into());
                }
            } else {
                parsed_token
            }
        }
        Token::Identifier(identifier) => {
            // Try to find the identifier in the functions' list
            if let Some(function) = function_signatures.get(identifier) {
                // Parse the call arguments and tokens parsed.
                let (call_arguments, idx_jmp) = parse_functions::parse_function_call_args(
                    &tokens[*token_idx + 2..],
                    variable_scope,
                    function.function_sig.args.clone(),
                    function_signatures.clone(),
                    function_imports.clone(),
                    custom_items.clone(),
                )?;

                // Return the function call
                let parsed_token: ParsedToken = ParsedToken::FunctionCall(
                    (function.function_sig.clone(), identifier.clone()),
                    call_arguments,
                );

                // Increment the token index, and add the offset
                *token_idx += idx_jmp + 2 + 1;

                if let Some(Token::As) = tokens.get(*token_idx) {
                    if let Some(Token::TypeDefinition(target_type)) = tokens.get(*token_idx + 1) {
                        *token_idx += 2;

                        ParsedToken::TypeCast(Box::new(parsed_token), target_type.clone())
                    } else {
                        // Throw an error
                        return Err(ParserError::SyntaxError(
                            super::error::SyntaxError::AsRequiresTypeDef,
                        )
                        .into());
                    }
                } else {
                    // If the function's return type doesn't match the variable's return type return an error
                    if function.function_sig.return_type != variable_type {
                        return Err(ParserError::TypeError(
                            function.function_sig.return_type.clone(),
                            variable_type,
                        )
                        .into());
                    }

                    parsed_token
                }
            }
            // If the identifier could not be found in the function list search in the variable scope
            else if let Some(variable) = variable_scope.get(identifier) {
                let parsed_token = ParsedToken::VariableReference(identifier.clone());

                *token_idx += 1;

                if let Some(Token::As) = tokens.get(*token_idx) {
                    if let Some(Token::TypeDefinition(target_type)) = tokens.get(*token_idx + 1) {
                        // Ezt lehet hogy késöbb ki kell majd venni
                        if *target_type != variable_type {
                            return Err(
                                ParserError::TypeError(target_type.clone(), variable_type).into()
                            );
                        }

                        // Increment the token index after checking target type
                        *token_idx += 2;

                        // Return the type casted literal
                        return Ok(ParsedToken::TypeCast(
                            Box::new(parsed_token),
                            target_type.clone(),
                        ));
                    } else {
                        // Throw an error
                        return Err(ParserError::SyntaxError(
                            super::error::SyntaxError::AsRequiresTypeDef,
                        )
                        .into());
                    }
                }

                // If the variable's type doesnt match the one we want to modify throw an error.
                if variable_type != *variable {
                    return Err(ParserError::TypeError(variable.clone(), variable_type).into());
                }

                // Return the VariableReference
                parsed_token
            } else if let Some(function_sig) = function_imports.get(identifier) {
                // Parse the call arguments and tokens parsed.
                let (call_arguments, idx_jmp) = parse_functions::parse_function_call_args(
                    &tokens[*token_idx + 2..],
                    variable_scope,
                    function_sig.args.clone(),
                    function_signatures.clone(),
                    function_imports.clone(),
                    custom_items.clone(),
                )?;

                // Return the function call
                let parsed_token: ParsedToken = ParsedToken::FunctionCall(
                    (function_sig.clone(), identifier.clone()),
                    call_arguments,
                );

                // Increment the token index, and add the offset
                *token_idx += idx_jmp + 2 + 1;

                if let Some(Token::As) = tokens.get(*token_idx) {
                    if let Some(Token::TypeDefinition(target_type)) = tokens.get(*token_idx + 1) {
                        *token_idx += 2;

                        ParsedToken::TypeCast(Box::new(parsed_token), target_type.clone())
                    } else {
                        // Throw an error
                        return Err(ParserError::SyntaxError(
                            super::error::SyntaxError::AsRequiresTypeDef,
                        )
                        .into());
                    }
                } else {
                    // If the function's return type doesn't match the variable's return type return an error
                    if function_sig.return_type != variable_type {
                        return Err(ParserError::TypeError(
                            function_sig.return_type.clone(),
                            variable_type,
                        )
                        .into());
                    }

                    parsed_token
                }
            } else if let Some(custom_type) = custom_items.get(identifier) {
                match custom_type {
                    CustomType::Struct((_struct_name, struct_inner)) => {
                        if let Some(Token::OpenBraces) = tokens.get(*token_idx + 1) {
                            let closing_idx = find_closing_braces(&tokens[*token_idx + 2..], 0)?;

                            let struct_init_slice =
                                &tokens[*token_idx + 2..*token_idx + 2 + closing_idx];

                            let (_jump_idx, init_struct_token) = init_struct(
                                struct_init_slice,
                                struct_inner,
                                function_signatures.clone(),
                                function_imports,
                                custom_items.clone(),
                                variable_scope,
                            )?;

                            *token_idx = *token_idx + 2 + closing_idx + 1;

                            return Ok(init_struct_token);
                        }

                        return Err(ParserError::SyntaxError(
                            SyntaxError::InvalidStructFieldDefinition,
                        )
                        .into());
                    }
                    CustomType::Enum(index_map) => {
                        todo!()
                    }
                }
            } else {
                // If none of the above matches throw an error about the variable not being found
                return Err(ParserError::VariableNotFound(identifier.clone()).into());
            }
        }
        Token::OpenParentheses => {
            *token_idx += 1;

            let closing_idx = find_closing_paren(&tokens[*token_idx..], 0)?;

            // Get the tokens inside the block aka the "()"
            let tokens_inside_block = &tokens[*token_idx..*token_idx + closing_idx];

            let (parsed_token, _jmp_idx) = parse_value(
                tokens_inside_block,
                function_signatures.clone(),
                variable_scope,
                variable_type,
                function_imports,
                custom_items.clone(),
            )?;

            *token_idx += closing_idx + 1;

            ParsedToken::MathematicalBlock(Box::new(parsed_token))
        }

        _ => {
            // If we are parsing something else than something that hold a value return an error.
            return Err(
                ParserError::SyntaxError(super::error::SyntaxError::InvalidValue(
                    eval_token.clone(),
                ))
                .into(),
            );
        }
    };
    Ok(inner_value)
}

pub fn init_struct(
    struct_slice: &[Token],
    this_struct_field: &IndexMap<String, TypeDiscriminants>,
    function_signatures: Arc<IndexMap<String, UnparsedFunctionDefinition>>,
    function_imports: Arc<HashMap<String, FunctionSignature>>,
    custom_items: Arc<IndexMap<String, CustomType>>,
    variable_scope: &IndexMap<String, TypeDiscriminants>,
) -> anyhow::Result<(usize, ParsedToken)> {
    let mut struct_field_init_map: IndexMap<String, Box<ParsedToken>> = IndexMap::new();

    let mut idx: usize = 0;

    while idx < struct_slice.len() {
        if let Some(Token::Identifier(field_name)) = struct_slice.get(idx) {
            if let Some(Token::Colon) = struct_slice.get(idx + 1) {
                let selected_tokens = &struct_slice[idx + 2..];

                let (parsed_value, jump_idx) = parse_value(
                    selected_tokens,
                    function_signatures.clone(),
                    variable_scope,
                    this_struct_field
                        .get(field_name)
                        .ok_or(ParserError::SyntaxError(
                            SyntaxError::InvalidStructFieldDefinition,
                        ))?
                        .clone(),
                    function_imports.clone(),
                    custom_items.clone(),
                )?;

                idx += jump_idx + 2;

                struct_field_init_map.insert(field_name.to_string(), Box::new(parsed_value));

                if let Some(Token::Comma) = struct_slice.get(idx) {
                    idx += 1;
                    continue;
                }
            }
        }

        return Err(ParserError::SyntaxError(SyntaxError::InvalidStructFieldDefinition).into());
    }

    Ok((
        idx,
        ParsedToken::InitalizeStruct(this_struct_field.clone(), struct_field_init_map),
    ))
}
