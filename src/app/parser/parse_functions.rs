use anyhow::Result;
use indexmap::IndexMap;
use std::{collections::HashMap, fs, path::PathBuf, sync::Arc};

use crate::app::{
    parser::{parser::find_closing_comma, types::If},
    type_system::type_system::{OrdMap, Type, TypeDiscriminant},
};

use super::{
    error::{ParserError, SyntaxError},
    parser::{
        ParserState, find_closing_braces, find_closing_paren, parse_token_as_value, parse_value,
    },
    tokenizer::tokenize,
    types::{
        CustomType, FunctionDefinition, FunctionSignature, MathematicalSymbol, ParsedToken,
        StructFieldReference, Token, UnparsedFunctionDefinition, VariableReference,
    },
};

pub fn create_signature_table(
    tokens: Vec<Token>,
) -> Result<(
    IndexMap<String, UnparsedFunctionDefinition>,
    HashMap<String, FunctionDefinition>,
    HashMap<String, FunctionSignature>,
    IndexMap<String, CustomType>,
)> {
    let mut token_idx = 0;

    let mut function_list: IndexMap<String, UnparsedFunctionDefinition> = IndexMap::new();

    let mut source_imports: HashMap<String, FunctionDefinition> = HashMap::new();
    let mut external_imports: HashMap<String, FunctionSignature> = HashMap::new();

    let mut imported_file_list: HashMap<String, IndexMap<String, FunctionDefinition>> =
        HashMap::new();

    let mut custom_items: IndexMap<String, CustomType> = IndexMap::new();

    while token_idx < tokens.len() {
        let current_token = tokens[token_idx].clone();

        if current_token == Token::Function {
            if let Token::Identifier(function_name) = tokens[token_idx + 1].clone() {
                if tokens[token_idx + 2] == Token::OpenParentheses {
                    let (bracket_close_idx, args) =
                        parse_signature_argument_tokens(&tokens[token_idx + 3..])?;

                    token_idx += bracket_close_idx + 3;

                    // Fetch the returned type of the function
                    if tokens[token_idx + 1] == Token::Colon {
                        let return_type = if let Token::TypeDefinition(return_type) =
                            tokens[token_idx + 2].clone()
                        {
                            return_type
                        } else if let Token::Identifier(identifier) = tokens[token_idx + 2].clone()
                        {
                            if let Some(custom_type) = custom_items.get(&identifier) {
                                match custom_type {
                                    CustomType::Struct(struct_def) => {
                                        TypeDiscriminant::Struct(struct_def.clone())
                                    }
                                    CustomType::Enum(index_map) => {
                                        unimplemented!()
                                    }
                                }
                            } else {
                                return Err(ParserError::InvalidSignatureDefinition.into());
                            }
                        } else {
                            return Err(ParserError::InvalidSignatureDefinition.into());
                        };

                        if tokens[token_idx + 3] == Token::OpenBraces {
                            // Create a variable which stores the level of braces we are in
                            let mut brace_layer_counter = 1;

                            // Get the slice of the list which may contain the braces' scope
                            let tokens_slice = &tokens[token_idx + 4..];

                            // Create an index which indexes the tokens slice
                            let mut token_braces_idx = 0;

                            // Create a list which contains all the tokens inside the two braces
                            let mut braces_contains: Vec<Token> = vec![];

                            // Find the scope of this function
                            loop {
                                // We have itered through the whole function and its still not found, it may be an open brace.
                                if tokens_slice.len() == token_braces_idx {
                                    return Err(ParserError::SyntaxError(
                                        crate::app::parser::error::SyntaxError::LeftOpenParentheses,
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

                                // If we have arrived at the end of the braces this is when we know that this is the end of the function's scope
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
                            let insertion = function_list.insert(
                                function_name.clone(),
                                UnparsedFunctionDefinition {
                                    inner: braces_contains,
                                    function_sig: FunctionSignature { args, return_type },
                                },
                            );

                            // If a function with a similar name exists throw an error as there is no function overloading
                            if let Some(overwritten_function) = insertion {
                                return Err(ParserError::SyntaxError(
                                    super::error::SyntaxError::DuplicateFunctions(
                                        function_name,
                                        overwritten_function.function_sig,
                                    ),
                                )
                                .into());
                            }

                            // Set the iterator index
                            token_idx += braces_contains_len + 4;

                            // Countinue with the loop
                            continue;
                        }
                    }

                    return Err(ParserError::InvalidSignatureDefinition.into());
                } else {
                    return Err(ParserError::InvalidSignatureDefinition.into());
                }
            } else {
                return Err(ParserError::SyntaxError(
                    super::error::SyntaxError::InvalidFunctionName,
                )
                .into());
            }
        } else if current_token == Token::Import {
            if let Token::Identifier(identifier) = tokens[token_idx + 1].clone() {
                if tokens[token_idx + 2] == Token::OpenParentheses {
                    let (bracket_close_idx, args) =
                        parse_signature_argument_tokens(&tokens[token_idx + 3..])?;

                    token_idx += bracket_close_idx + 3;

                    if tokens[token_idx + 1] == Token::Colon {
                        if let Token::TypeDefinition(return_type) = tokens[token_idx + 2].clone() {
                            if tokens[token_idx + 3] == Token::LineBreak {
                                if external_imports.get(&identifier).is_some()
                                    || function_list.get(&identifier).is_some()
                                {
                                    return Err(ParserError::DuplicateSignatureImports.into());
                                }

                                external_imports
                                    .insert(identifier, FunctionSignature { args, return_type });

                                continue;
                            }
                        }
                    } else {
                        return Err(SyntaxError::ImportUnspecifiedReturnType.into());
                    }
                }
                // This is matched when you are importing a named declaration from another fog source file
                else if Token::DoubleColon == tokens[token_idx + 2] {
                    if let Token::Identifier(lib_function_name) = &tokens[token_idx + 3] {
                        let imported_file_query = imported_file_list.get(&identifier);

                        if Token::LineBreak == tokens[token_idx + 4] {
                            if let Some(imported_file) = imported_file_query {
                                if let Some(function_def) = imported_file.get(lib_function_name) {
                                    // Store the imported function
                                    source_imports
                                        .insert(lib_function_name.clone(), function_def.clone());

                                    // Increment token index
                                    token_idx += 4;

                                    // Continue looping over the top-level tokens
                                    continue;
                                }
                            }
                        }
                    }
                }
            } else if let Token::Literal(Type::String(path_to_linked_file)) =
                tokens[token_idx + 1].clone()
            {
                // Turn the String literal into path
                let path = PathBuf::from(format!("src/{path_to_linked_file}")).canonicalize()?;

                // Check if a file exists at that path
                if !fs::exists(&path)?
                    || path.extension().unwrap_or_default().to_string_lossy() == ".f"
                {
                    return Err(ParserError::LinkedSourceFileMissing(path).into());
                }

                // Get the File's content
                let file_contents = fs::read_to_string(&path)?;

                // Get the file's name so that it can be referred to later
                let file_name = path
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();

                // Tokenize the raw source file
                let tokens = tokenize(&file_contents)?;

                // Create a new Parser state
                let mut parser_state = ParserState::new(tokens);

                println!(
                    "Imported file from `{}`. Parsing source file...",
                    path.display()
                );

                // Parse the tokens
                parser_state.parse_tokens()?;

                // Save the file's name and the functions it contains so that we can refer to it later.
                imported_file_list.insert(file_name, parser_state.function_table().clone());

                token_idx += 2;
                continue;
            }

            return Err(ParserError::SyntaxError(
                super::error::SyntaxError::InvalidImportDefinition,
            )
            .into());
        } else if current_token == Token::Struct {
            if let Some(Token::Identifier(struct_name)) = tokens.get(token_idx + 1) {
                if let Some(Token::OpenBraces) = tokens.get(token_idx + 2) {
                    // Search for the closing brace's index
                    let braces_idx =
                        find_closing_braces(&tokens[token_idx + 3..], 0)? + token_idx + 3;

                    // Retrive the tokens from the braces
                    let struct_slice = tokens[token_idx + 3..braces_idx].to_vec();

                    // Create a list for the struct fields
                    let mut struct_fields: IndexMap<String, TypeDiscriminant> = IndexMap::new();

                    // Store the idx
                    let mut token_idx = 0;

                    // Parse the struct fields
                    while token_idx < struct_slice.len() {
                        // Get the current token
                        let current_token = &struct_slice[token_idx];

                        // Pattern match the syntax
                        if let Token::Identifier(field_name) = current_token {
                            if let Token::Colon = struct_slice[token_idx + 1] {
                                if let Some(Token::Comma) = struct_slice.get(token_idx + 3) {
                                    if let Token::TypeDefinition(field_type) =
                                        &struct_slice[token_idx + 2]
                                    {
                                        // Save the field's type and name
                                        struct_fields
                                            .insert(field_name.clone(), field_type.clone());

                                        // Increment the token index
                                        token_idx += 4;

                                        // Continue looping through, if the pattern doesnt match the syntax return an error
                                        continue;
                                    } else if let Token::Identifier(custom_type) =
                                        &struct_slice[token_idx + 2]
                                    {
                                        if let Some(custom_item) = custom_items.get(custom_type) {
                                            match custom_item {
                                                CustomType::Struct(struct_def) => {
                                                    struct_fields.insert(
                                                        field_name.to_string(),
                                                        TypeDiscriminant::Struct(
                                                            struct_def.clone(),
                                                        ),
                                                    );
                                                }
                                                CustomType::Enum(index_map) => {
                                                    todo!()
                                                }
                                            }

                                            // Increment the token index
                                            token_idx += 4;

                                            // Continue looping through, if the pattern doesnt match the syntax return an error
                                            continue;
                                        }
                                    }
                                }
                            }
                        }

                        // Return a syntax error
                        return Err(ParserError::SyntaxError(
                            super::error::SyntaxError::InvalidStructFieldDefinition,
                        )
                        .into());
                    }

                    // Save the custom item
                    custom_items.insert(
                        struct_name.to_string(),
                        CustomType::Struct((struct_name.clone(), struct_fields.into())),
                    );
                }
            } else {
                return Err(ParserError::SyntaxError(
                    super::error::SyntaxError::InvalidStructDefinition,
                )
                .into());
            }
        }

        token_idx += 1;
    }

    Ok((
        function_list,
        source_imports,
        external_imports,
        custom_items,
    ))
}

fn parse_signature_argument_tokens(tokens: &[Token]) -> Result<(usize, FunctionArguments)> {
    let bracket_closing_idx =
        find_closing_paren(tokens, 0).map_err(|_| ParserError::InvalidSignatureDefinition)?;

    let mut args = FunctionArguments::new();

    if bracket_closing_idx != 0 {
        args = parse_signature_args(&tokens[..bracket_closing_idx])?;
    }

    Ok((bracket_closing_idx, args))
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct FunctionArguments {
    pub arguments_list: OrdMap<String, TypeDiscriminant>,
    pub ellipsis_present: bool,
}

impl FunctionArguments {
    pub fn new() -> Self {
        Self {
            arguments_list: OrdMap::new(),
            ellipsis_present: false,
        }
    }
}

fn parse_signature_args(token_list: &[Token]) -> Result<FunctionArguments> {
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
                    } else {
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

pub fn parse_functions(
    unparsed_functions: Arc<IndexMap<String, UnparsedFunctionDefinition>>,
    function_imports: Arc<HashMap<String, FunctionSignature>>,
    custom_items: Arc<IndexMap<String, CustomType>>,
) -> Result<IndexMap<String, FunctionDefinition>> {
    let mut parsed_functions = IndexMap::new();

    for (fn_idx, (fn_name, unparsed_function)) in unparsed_functions.clone().iter().enumerate() {
        let function_definition = FunctionDefinition {
            function_sig: unparsed_function.function_sig.clone(),
            inner: parse_function_block(
                unparsed_function.inner.clone(),
                unparsed_functions.clone(),
                unparsed_function.function_sig.clone(),
                function_imports.clone(),
                custom_items.clone(),
                unparsed_function.function_sig.args.clone(),
            )?,
        };

        println!(
            "Parsed function `{fn_name}` ({}/{})",
            fn_idx + 1,
            unparsed_functions.len()
        );
        parsed_functions.insert(fn_name.clone(), function_definition);
    }

    Ok(parsed_functions)
}

fn parse_function_block(
    tokens: Vec<Token>,
    function_signatures: Arc<IndexMap<String, UnparsedFunctionDefinition>>,
    this_function_signature: FunctionSignature,
    function_imports: Arc<HashMap<String, FunctionSignature>>,
    custom_items: Arc<IndexMap<String, CustomType>>,
    this_fn_args: FunctionArguments,
) -> Result<Vec<ParsedToken>> {
    // Check if the function defined by the source code does not have an indeterminate amount of args
    if this_fn_args.ellipsis_present {
        return Err(ParserError::DeterminiateArgumentsFunction.into());
    }

    let mut token_idx = 0;

    let mut variable_scope = this_fn_args.arguments_list.clone();

    let mut parsed_tokens: Vec<ParsedToken> = Vec::new();

    let mut has_return = false;

    if !tokens.is_empty() {
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

                        let (parsed_value, _, _) = parse_value(
                            selected_tokens,
                            function_signatures.clone(),
                            &mut variable_scope,
                            Some(var_type.clone()),
                            function_imports.clone(),
                            custom_items.clone(),
                        )?;

                        parsed_tokens.push(ParsedToken::NewVariable(
                            var_name.clone(),
                            var_type.clone(),
                            Box::new(parsed_value.clone()),
                        ));

                        variable_scope.insert(var_name, var_type.clone());
                    } else {
                        parsed_tokens.push(ParsedToken::NewVariable(
                            var_name.clone(),
                            var_type.clone(),
                            Box::new(ParsedToken::Literal(var_type.clone().into())),
                        ));

                        variable_scope.insert(var_name.clone(), var_type.clone());

                        token_idx += 2;
                    }

                    if tokens[token_idx] == Token::LineBreak {
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
            } else if let Token::Identifier(ref ident_name) = current_token {
                // If the variable exists in the current scope
                if let Some(variable_type) = variable_scope.get(ident_name).cloned() {
                    // Increment the token index
                    token_idx += 1;

                    // Parse the variable's expression
                    let mut variable_ref =
                        VariableReference::BasicReference(ident_name.to_string());

                    parse_variable_expression(
                        &tokens,
                        &tokens[token_idx],
                        &mut token_idx,
                        function_signatures.clone(),
                        function_imports.clone(),
                        &mut variable_scope,
                        variable_type,
                        custom_items.clone(),
                        &mut variable_ref,
                        &mut parsed_tokens,
                    )?;
                } else if let Some(function_sig) = function_signatures.get(ident_name) {
                    // If after the function name the first thing isnt a `(` return a syntax error.
                    if tokens[token_idx + 1] != Token::OpenParentheses {
                        return Err(ParserError::SyntaxError(
                            crate::app::parser::error::SyntaxError::InvalidFunctionDefinition,
                        )
                        .into());
                    }

                    let paren_start_slice = &tokens[token_idx + 2..];

                    let bracket_idx = find_closing_paren(paren_start_slice, 0)? + token_idx;

                    let (variables_passed, jumped_idx) = parse_function_call_args(
                        &tokens[token_idx + 2..bracket_idx + 2],
                        &mut variable_scope,
                        function_sig.function_sig.args.clone(),
                        function_signatures.clone(),
                        function_imports.clone(),
                        custom_items.clone(),
                    )?;

                    parsed_tokens.push(ParsedToken::FunctionCall(
                        (function_sig.function_sig.clone(), ident_name.clone()),
                        variables_passed,
                    ));

                    token_idx += jumped_idx + 2;
                } else if let Some(function_sig) = function_imports.get(ident_name) {
                    // If after the function name the first thing isnt a `(` return a syntax error.
                    if tokens[token_idx + 1] != Token::OpenParentheses {
                        return Err(ParserError::SyntaxError(
                            crate::app::parser::error::SyntaxError::InvalidFunctionDefinition,
                        )
                        .into());
                    }

                    let paren_start_slice = &tokens[token_idx + 2..];

                    let bracket_idx = find_closing_paren(paren_start_slice, 0)? + token_idx;

                    let (variables_passed, jumped_idx) = parse_function_call_args(
                        &tokens[token_idx + 2..bracket_idx + 2],
                        &mut variable_scope,
                        function_sig.args.clone(),
                        function_signatures.clone(),
                        function_imports.clone(),
                        custom_items.clone(),
                    )?;

                    parsed_tokens.push(ParsedToken::FunctionCall(
                        (function_sig.clone(), ident_name.clone()),
                        variables_passed,
                    ));

                    token_idx += jumped_idx + 2;
                } else if let Some(custom_type) = custom_items.get(ident_name) {
                    match custom_type {
                        CustomType::Struct(struct_instance) => {
                            let variable_type = TypeDiscriminant::Struct(struct_instance.clone());
                            token_idx += 1;

                            if let Some(Token::Identifier(var_name)) = tokens.get(token_idx) {
                                if let Some(Token::SetValue) = tokens.get(token_idx + 1) {
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

                                    let selected_tokens = &tokens[token_idx + 2..line_break_idx];

                                    token_idx += selected_tokens.len() + 1;

                                    let (parsed_token, _, _) = parse_value(
                                        selected_tokens,
                                        function_signatures.clone(),
                                        &mut variable_scope,
                                        Some(variable_type.clone()),
                                        function_imports.clone(),
                                        custom_items.clone(),
                                    )?;

                                    parsed_tokens.push(ParsedToken::NewVariable(
                                        var_name.clone(),
                                        variable_type,
                                        Box::new(parsed_token),
                                    ));

                                    variable_scope.insert(
                                        var_name.clone(),
                                        TypeDiscriminant::Struct(struct_instance.clone()),
                                    );
                                }
                            }
                        }
                        CustomType::Enum(enum_types) => {}
                    };
                } else {
                    return Err(ParserError::VariableNotFound(ident_name.clone()).into());
                }
            } else if Token::Return == current_token {
                has_return = true;

                token_idx += 1;

                let next_token = &tokens[token_idx];

                if this_function_signature.return_type.clone() == TypeDiscriminant::Void {
                    if *next_token != Token::LineBreak {
                        return Err(ParserError::SyntaxError(
                            super::error::SyntaxError::InvalidStatementDefinition,
                        )
                        .into());
                    }
                } else {
                    let (returned_value, jmp_idx, _) = parse_value(
                        &tokens[token_idx..],
                        function_signatures.clone(),
                        &mut variable_scope,
                        Some(this_function_signature.return_type.clone()),
                        function_imports.clone(),
                        custom_items.clone(),
                    )?;

                    token_idx += jmp_idx;

                    parsed_tokens.push(ParsedToken::ReturnValue(Box::new(returned_value)));
                }
            } else if Token::If == current_token {
                token_idx += 1;

                if let Token::OpenParentheses = tokens[token_idx] {
                    token_idx += 1;
                    let paren_close_idx = find_closing_paren(&tokens[token_idx..], 0)? + token_idx;

                    // This is what we have to evaulate in order to execute the appropriate branch of the if statement
                    let cond_slice = &tokens[token_idx..paren_close_idx];

                    let (condition, _idx, _) = parse_value(
                        cond_slice,
                        function_signatures.clone(),
                        &mut variable_scope,
                        None,
                        function_imports.clone(),
                        custom_items.clone(),
                    )?;

                    token_idx = paren_close_idx + 1;

                    if Token::OpenBraces == tokens[token_idx] {
                        token_idx += 1;

                        let paren_close_idx =
                            find_closing_braces(&tokens[token_idx..], 0)? + token_idx;

                        let true_block_slice = tokens[token_idx..paren_close_idx].to_vec();

                        let true_condition_block = parse_function_block(
                            true_block_slice,
                            function_signatures.clone(),
                            FunctionSignature {
                                args: FunctionArguments::new(),
                                return_type: TypeDiscriminant::Void,
                            },
                            function_imports.clone(),
                            custom_items.clone(),
                            this_fn_args.clone(),
                        )?;

                        let mut else_condition_branch = Vec::new();

                        token_idx = paren_close_idx + 1;

                        if Some(&Token::Else) == tokens.get(token_idx) {
                            token_idx += 1;

                            if Some(&Token::OpenBraces) == tokens.get(token_idx) {
                                token_idx += 1;

                                let paren_close_idx =
                                    find_closing_braces(&tokens[token_idx..], 0)? + token_idx;

                                let false_block_slice = tokens[token_idx..paren_close_idx].to_vec();

                                else_condition_branch = parse_function_block(
                                    false_block_slice,
                                    function_signatures.clone(),
                                    FunctionSignature {
                                        args: FunctionArguments::new(),
                                        return_type: TypeDiscriminant::Void,
                                    },
                                    function_imports.clone(),
                                    custom_items.clone(),
                                    this_fn_args.clone(),
                                )?;

                                token_idx = paren_close_idx + 1;
                            }
                        }

                        parsed_tokens.push(ParsedToken::If(If {
                            condition: Box::new(condition),
                            complete_body: true_condition_block,
                            incomplete_body: else_condition_branch,
                        }));

                        continue;
                    }
                }

                return Err(
                    ParserError::SyntaxError(SyntaxError::InvalidIfConditionDefinition).into(),
                );
            } else if Token::Loop == current_token {
                token_idx += 1;

                if let Token::OpenBraces = tokens[token_idx] {
                    token_idx += 1;

                    let paren_close_idx = find_closing_braces(&tokens[token_idx..], 0)? + token_idx;

                    // This is what we have to evaulate in order to execute the appropriate branch of the if statement
                    let loop_body_tokens = &tokens[token_idx..paren_close_idx];

                    // Create a custom FunctionArguments instance for the loop
                    let loop_body_arguments = FunctionArguments {
                        // Pass in the variable scope of the previous "closure" to the loop so that variables defined above are still accessible inside the loop.
                        // We do this instead of modifying the function entirely.
                        arguments_list: this_fn_args
                            .arguments_list
                            .extend_clone(variable_scope.clone()),
                        ellipsis_present: this_fn_args.ellipsis_present,
                    };

                    let loop_body = parse_function_block(
                        loop_body_tokens.to_vec(),
                        function_signatures.clone(),
                        FunctionSignature {
                            args: FunctionArguments::new(),
                            return_type: TypeDiscriminant::Void,
                        },
                        function_imports.clone(),
                        custom_items.clone(),
                        loop_body_arguments,
                    )?;

                    token_idx = paren_close_idx + 1;

                    parsed_tokens.push(ParsedToken::Loop(loop_body));

                    continue;
                }

                return Err(ParserError::SyntaxError(SyntaxError::InvalidLoopBody).into());
            } else if Token::Continue == current_token {
                parsed_tokens.push(ParsedToken::ControlFlow(
                    super::types::ControlFlowType::Continue,
                ));

                token_idx += 1;
            } else if Token::Break == current_token {
                parsed_tokens.push(ParsedToken::ControlFlow(
                    super::types::ControlFlowType::Break,
                ));

                token_idx += 1;
            }

            token_idx += 1;
        }
    }

    // If there isnt a returned value and the returned type isnt `Void` raise an error
    if !has_return && this_function_signature.return_type != TypeDiscriminant::Void {
        return Err(
            ParserError::SyntaxError(super::error::SyntaxError::FunctionRequiresReturn).into(),
        );
    }

    Ok(parsed_tokens)
}

fn set_value_math_expr(
    tokens: &[Token],
    function_signatures: Arc<IndexMap<String, UnparsedFunctionDefinition>>,
    token_idx: &mut usize,
    parsed_tokens: &mut Vec<ParsedToken>,
    variable_scope: &mut IndexMap<String, TypeDiscriminant>,
    variable_type: TypeDiscriminant,
    variable_reference: VariableReference,
    math_symbol: MathematicalSymbol,
    standard_function_table: Arc<HashMap<String, FunctionSignature>>,
    custom_items: Arc<IndexMap<String, CustomType>>,
) -> Result<(), anyhow::Error> {
    *token_idx += 1;

    let eval_token = tokens.get(*token_idx).ok_or(ParserError::SyntaxError(
        super::error::SyntaxError::InvalidStatementDefinition,
    ))?;

    let (next_token, ty) = parse_token_as_value(
        tokens,
        function_signatures,
        variable_scope,
        Some(variable_type.clone()),
        token_idx,
        eval_token,
        standard_function_table,
        custom_items.clone(),
    )?;

    parsed_tokens.push(ParsedToken::SetValue(
        variable_reference.clone(),
        Box::new(ParsedToken::MathematicalExpression(
            Box::new(ParsedToken::VariableReference(variable_reference)),
            math_symbol,
            Box::new(next_token),
        )),
    ));

    Ok(())
}

/// First token should be the first argument
pub fn parse_function_call_args(
    tokens: &[Token],
    variable_scope: &mut IndexMap<String, TypeDiscriminant>,
    mut this_function_args: FunctionArguments,
    function_signatures: Arc<IndexMap<String, UnparsedFunctionDefinition>>,
    standard_function_table: Arc<HashMap<String, FunctionSignature>>,
    custom_items: Arc<IndexMap<String, CustomType>>,
) -> Result<(
    OrdMap<Option<String>, (ParsedToken, TypeDiscriminant)>,
    usize,
)> {
    let mut tokens_idx = 0;

    let args_list_len = tokens[tokens_idx..].len() + tokens_idx;

    // Arguments which will passed in to the function
    let mut arguments: OrdMap<Option<String>, (ParsedToken, TypeDiscriminant)> = OrdMap::new();

    // If there are no arguments just return everything as is
    if tokens.is_empty() {
        return Ok((arguments, tokens_idx));
    }

    while tokens_idx < tokens.len() {
        let current_token = tokens[tokens_idx].clone();

        if let Token::Identifier(arg_name) = current_token.clone() {
            if let Some(Token::SetValue) = tokens.get(tokens_idx + 1) {
                let argument_type = this_function_args
                    .arguments_list
                    .get(&arg_name)
                    .ok_or(ParserError::ArgumentError(arg_name.clone()))?;

                tokens_idx += 2;

                let closing_idx = find_closing_comma(&tokens[tokens_idx..])? + tokens_idx;

                let (parsed_argument, jump_idx, arg_ty) = parse_value(
                    &tokens[tokens_idx..closing_idx],
                    function_signatures.clone(),
                    variable_scope,
                    Some(argument_type.clone()),
                    standard_function_table.clone(),
                    custom_items.clone(),
                )?;

                tokens_idx += jump_idx;

                // Remove tha argument from the argument list so we can parse unnamed arguments easier
                this_function_args.arguments_list.shift_remove(&arg_name);

                arguments.insert(Some(arg_name.clone()), (parsed_argument, arg_ty));
            } else {
                let args_list_len = &tokens[tokens_idx..].len() + tokens_idx;

                let mut token_buf = Vec::new();

                let mut bracket_counter = 0;

                // We should start by finding the comma and parsing the tokens in between the current idx and the comma
                while tokens_idx < args_list_len {
                    let token = &tokens[tokens_idx];

                    if *token == Token::OpenParentheses {
                        bracket_counter += 1;
                    } else if *token == Token::CloseParentheses {
                        bracket_counter -= 1;
                    }

                    // If a comma is found parse the tokens from the slice
                    if (*token == Token::Comma && bracket_counter == 0)
                        || tokens_idx == args_list_len - 1
                    {
                        if tokens_idx == args_list_len - 1 {
                            token_buf.push(token.clone());
                        }

                        let fn_argument = this_function_args.arguments_list.first_entry();

                        if let Some(fn_argument) = fn_argument {
                            let (parsed_argument, _jump_idx, arg_ty) = parse_value(
                                &token_buf,
                                function_signatures.clone(),
                                variable_scope,
                                Some(fn_argument.get().clone()),
                                standard_function_table.clone(),
                                custom_items.clone(),
                            )?;

                            tokens_idx += 1;

                            token_buf.clear();

                            arguments
                                .insert(Some(fn_argument.key().clone()), (parsed_argument, arg_ty));

                            // Remove the argument from the argument list
                            fn_argument.shift_remove();
                        } else {
                            let (parsed_argument, _jump_idx, arg_ty) = parse_value(
                                &token_buf,
                                function_signatures.clone(),
                                variable_scope,
                                None,
                                standard_function_table.clone(),
                                custom_items.clone(),
                            )?;

                            tokens_idx += 1;

                            token_buf.clear();

                            arguments.insert(None, (parsed_argument, arg_ty));
                        }

                        break;
                    } else {
                        token_buf.push(token.clone());
                    }

                    tokens_idx += 1;
                }
            }
        } else if Token::CloseParentheses == current_token {
            break;
        } else if Token::Comma == current_token {
            tokens_idx += 1;
        } else {
            let mut token_buf = Vec::new();
            let mut bracket_counter: i32 = 0;

            // We should start by finding the comma and parsing the tokens in between the current idx and the comma
            while tokens_idx < args_list_len {
                let token = &tokens[tokens_idx];

                if *token == Token::OpenParentheses {
                    bracket_counter += 1;
                } else if *token == Token::CloseParentheses {
                    bracket_counter -= 1;
                }

                // If a comma is found parse the tokens from the slice
                if (*token == Token::Comma && bracket_counter == 0)
                    || tokens_idx == args_list_len - 1
                {
                    if tokens_idx == args_list_len - 1 {
                        token_buf.push(token.clone());
                    }

                    let fn_argument = this_function_args.arguments_list.first_entry();

                    if let Some(fn_argument) = fn_argument {
                        let (parsed_argument, _jump_idx, arg_ty) = parse_value(
                            &token_buf,
                            function_signatures.clone(),
                            variable_scope,
                            Some(fn_argument.get().clone()),
                            standard_function_table.clone(),
                            custom_items.clone(),
                        )?;

                        tokens_idx += 1;

                        token_buf.clear();

                        arguments
                            .insert(Some(fn_argument.key().clone()), (parsed_argument, arg_ty));

                        // Remove the argument from the argument list
                        fn_argument.shift_remove();
                    } else {
                        let (parsed_argument, _jump_idx, arg_ty) = parse_value(
                            &token_buf,
                            function_signatures.clone(),
                            variable_scope,
                            None,
                            standard_function_table.clone(),
                            custom_items.clone(),
                        )?;

                        tokens_idx += 1;

                        token_buf.clear();

                        arguments.insert(None, (parsed_argument, arg_ty));
                    }

                    break;
                } else {
                    token_buf.push(token.clone());
                }

                tokens_idx += 1;
            }
        }
    }

    if !this_function_args.arguments_list.is_empty() {
        return Err(ParserError::InvalidFunctionArgumentCount.into());
    }

    Ok((arguments, tokens_idx))
}

pub fn parse_variable_expression(
    tokens: &[Token],
    current_token: &Token,
    token_idx: &mut usize,
    function_signatures: Arc<IndexMap<String, UnparsedFunctionDefinition>>,
    function_imports: Arc<HashMap<String, FunctionSignature>>,
    variable_scope: &mut IndexMap<String, TypeDiscriminant>,
    variable_type: TypeDiscriminant,
    custom_items: Arc<IndexMap<String, CustomType>>,
    variable_ref: &mut VariableReference,
    parsed_tokens: &mut Vec<ParsedToken>,
) -> anyhow::Result<()> {
    match &current_token {
        Token::SetValue => {
            let line_break_idx = tokens
                .iter()
                .skip(*token_idx)
                .position(|token| *token == Token::LineBreak)
                .ok_or({
                    ParserError::SyntaxError(
                        crate::app::parser::error::SyntaxError::MissingLineBreak,
                    )
                })?
                + *token_idx;

            let selected_tokens = &tokens[*token_idx + 1..line_break_idx];

            *token_idx += selected_tokens.len() + 1;

            let (parsed_token, _, _) = parse_value(
                selected_tokens,
                function_signatures.clone(),
                variable_scope,
                Some(variable_type.clone()),
                function_imports.clone(),
                custom_items.clone(),
            )?;

            parsed_tokens.push(ParsedToken::SetValue(
                variable_ref.clone(),
                Box::new(parsed_token),
            ));
        }
        Token::SetValueAddition => {
            set_value_math_expr(
                tokens,
                function_signatures,
                token_idx,
                parsed_tokens,
                variable_scope,
                variable_type,
                variable_ref.clone(),
                MathematicalSymbol::Addition,
                function_imports.clone(),
                custom_items.clone(),
            )?;
        }
        Token::SetValueSubtraction => {
            set_value_math_expr(
                tokens,
                function_signatures,
                token_idx,
                parsed_tokens,
                variable_scope,
                variable_type,
                variable_ref.clone(),
                MathematicalSymbol::Subtraction,
                function_imports.clone(),
                custom_items.clone(),
            )?;
        }
        Token::SetValueDivision => {
            set_value_math_expr(
                tokens,
                function_signatures,
                token_idx,
                parsed_tokens,
                variable_scope,
                variable_type,
                variable_ref.clone(),
                MathematicalSymbol::Division,
                function_imports.clone(),
                custom_items.clone(),
            )?;
        }
        Token::SetValueMultiplication => {
            set_value_math_expr(
                tokens,
                function_signatures,
                token_idx,
                parsed_tokens,
                variable_scope,
                variable_type,
                variable_ref.clone(),
                MathematicalSymbol::Multiplication,
                function_imports.clone(),
                custom_items.clone(),
            )?;
        }
        Token::SetValueModulo => {
            set_value_math_expr(
                tokens,
                function_signatures,
                token_idx,
                parsed_tokens,
                variable_scope,
                variable_type,
                variable_ref.clone(),
                MathematicalSymbol::Modulo,
                function_imports.clone(),
                custom_items.clone(),
            )?;
        }
        Token::Dot => {
            let field_name = &tokens.get(*token_idx + 1);

            if let TypeDiscriminant::Struct((struct_name, struct_def)) = variable_type {
                if let Some(Token::Identifier(field_name)) = field_name {
                    if let Some(struct_field_ty) = struct_def.get(field_name) {
                        match variable_ref {
                            VariableReference::StructFieldReference(
                                struct_field_reference,
                                struct_ty,
                            ) => {
                                struct_field_reference
                                    .field_stack
                                    .push(field_name.to_string());
                            }
                            VariableReference::BasicReference(basic_ref) => {
                                *variable_ref = VariableReference::StructFieldReference(
                                    StructFieldReference::from_stack(vec![
                                        basic_ref.to_string(),
                                        field_name.to_string(),
                                    ]),
                                    (struct_name, struct_def.clone()),
                                );
                            }
                        }

                        *token_idx += 2;

                        parse_variable_expression(
                            tokens,
                            &tokens[*token_idx],
                            token_idx,
                            function_signatures,
                            function_imports,
                            variable_scope,
                            struct_field_ty.clone(),
                            custom_items,
                            variable_ref,
                            parsed_tokens,
                        )?;
                    } else {
                        return Err(ParserError::SyntaxError(SyntaxError::StructFieldNotFound(
                            field_name.to_string(),
                            (struct_name, struct_def),
                        ))
                        .into());
                    }
                } else {
                    return Err(ParserError::SyntaxError(SyntaxError::StructFieldNotFound(
                        format!("{field_name:?}"),
                        (struct_name, struct_def),
                    ))
                    .into());
                }
            } else {
                return Err(ParserError::SyntaxError(SyntaxError::InvalidDotPlacement).into());
            }

            if let Some(idx) = tokens
                .iter()
                .skip(*token_idx)
                .position(|token| *token == Token::LineBreak)
            {
                *token_idx += idx;
            } else {
                return Err(ParserError::SyntaxError(SyntaxError::MissingLineBreak).into());
            }
        }
        Token::OpenSquareBrackets => {
            if matches!(variable_type, TypeDiscriminant::Vector(_)) {
                return Err(ParserError::TypeNonIndexable(variable_type).into());
            }

            let square_brackets_break_idx = tokens
                .iter()
                .skip(*token_idx)
                .position(|token| *token == Token::OpenSquareBrackets)
                .ok_or({
                    ParserError::SyntaxError(
                        crate::app::parser::error::SyntaxError::LeftOpenSquareBrackets,
                    )
                })?
                + *token_idx;

            let selected_tokens = &tokens[*token_idx..square_brackets_break_idx];

            let (value, idx_jmp, _) = parse_value(
                selected_tokens,
                function_signatures,
                variable_scope,
                Some(TypeDiscriminant::U64),
                function_imports,
                custom_items,
            )?;

            *token_idx += idx_jmp;

            if let ParsedToken::Literal(Type::U64(idx)) = value {
                parsed_tokens.push(ParsedToken::VectorIndexing(variable_ref.clone(), idx));
            }

            if let Some(Token::CloseSquareBrackets) = tokens.get(*token_idx) {
                *token_idx += 1;
            } else {
                return Err(ParserError::SyntaxError(SyntaxError::LeftOpenSquareBrackets).into());
            }
        }
        _ => {
            println!("[ERROR] Unimplemented token: {}", tokens[*token_idx]);
        }
    }

    Ok(())
}
