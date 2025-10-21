use std::{collections::HashMap, fs, path::PathBuf, sync::Arc};

use fog_common::{
    anyhow::Result,
    codegen::{CustomType, FunctionArgumentIdentifier, If},
    compiler::ProjectConfig,
    error::{parser::ParserError, syntax::SyntaxError},
    indexmap::IndexMap,
    parser::{
        ControlFlowType, FunctionArguments, FunctionDefinition, FunctionSignature, ParsedToken,
        UnparsedFunctionDefinition, VariableReference, find_closing_braces, find_closing_comma,
        find_closing_paren, parse_signature_argument_tokens,
    },
    tokenizer::Token,
    ty::{OrdMap, Type, TypeDiscriminant},
};

use crate::{
    parser::variable::{parse_value, parse_variable_expression},
    parser_instance::Parser,
    tokenizer::tokenize,
};

pub fn create_signature_table(
    tokens: Vec<Token>,
    module_path: Vec<String>,
) -> Result<(
    IndexMap<String, UnparsedFunctionDefinition>,
    HashMap<String, FunctionDefinition>,
    HashMap<String, FunctionSignature>,
    IndexMap<String, CustomType>,
)>
{
    let mut token_idx = 0;

    let mut function_list: IndexMap<String, UnparsedFunctionDefinition> = IndexMap::new();

    let mut source_imports: HashMap<String, FunctionDefinition> = HashMap::new();
    let mut external_imports: HashMap<String, FunctionSignature> = HashMap::new();

    let mut imported_file_list: HashMap<String, IndexMap<String, FunctionDefinition>> =
        HashMap::new();

    let mut custom_items: IndexMap<String, CustomType> = IndexMap::new();

    while token_idx < tokens.len() {
        let current_token = tokens[token_idx].clone();

        if current_token == Token::Private
            || current_token == Token::Public
            || current_token == Token::PublicLibrary
        {
            token_idx += 1;

            if tokens[token_idx] == Token::Function {
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
                            }
                            else if let Token::Identifier(identifier) =
                                tokens[token_idx + 2].clone()
                            {
                                if let Some(custom_type) = custom_items.get(&identifier) {
                                    match custom_type {
                                        CustomType::Struct(struct_def) => {
                                            TypeDiscriminant::Struct(struct_def.clone())
                                        },
                                        CustomType::Enum(index_map) => {
                                            unimplemented!()
                                        },
                                    }
                                }
                                else {
                                    return Err(ParserError::InvalidSignatureDefinition.into());
                                }
                            }
                            else {
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
                                            SyntaxError::LeftOpenParentheses,
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
                                        inner: braces_contains.clone(),
                                        function_sig: FunctionSignature {
                                            args: args.clone(),
                                            return_type: return_type.clone(),
                                            debug_attributes: None,
                                            visibility: current_token.try_into()?,
                                            module_path: module_path.clone(),
                                        },
                                    },
                                );

                                // If a function with a similar name exists throw an error as there is no function overloading
                                if let Some(overwritten_function) = insertion {
                                    return Err(ParserError::SyntaxError(
                                        SyntaxError::DuplicateFunctions(
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
                    }
                    else {
                        return Err(ParserError::InvalidSignatureDefinition.into());
                    }
                }
                else {
                    return Err(ParserError::SyntaxError(SyntaxError::InvalidFunctionName).into());
                }
            }
        }
        else if current_token == Token::Function {
            return Err(ParserError::FunctionRequiresExplicitVisibility.into());
        }
        else if current_token == Token::Import {
            if let Token::Identifier(identifier) = tokens[token_idx + 1].clone() {
                if tokens[token_idx + 2] == Token::OpenParentheses {
                    let (bracket_close_idx, args) =
                        parse_signature_argument_tokens(&tokens[token_idx + 3..])?;

                    token_idx += bracket_close_idx + 3;

                    if tokens[token_idx + 1] == Token::Colon {
                        if let Token::TypeDefinition(return_type) = tokens[token_idx + 2].clone()
                            && tokens[token_idx + 3] == Token::SemiColon
                        {
                            if external_imports.get(&identifier).is_some()
                                || function_list.get(&identifier).is_some()
                            {
                                return Err(ParserError::DuplicateSignatureImports.into());
                            }

                            external_imports.insert(
                                identifier,
                                FunctionSignature {
                                    args,
                                    return_type,
                                    debug_attributes: None,
                                    module_path: module_path.clone(),
                                    // Imported functions can only be accessed at the source file they were imported at
                                    // I might change this later to smth like pub import similar to pub mod in rust
                                    visibility: fog_common::parser::FunctionVisibility::Private,
                                },
                            );

                            continue;
                        }
                    }
                    else {
                        return Err(SyntaxError::ImportUnspecifiedReturnType.into());
                    }
                }
                // This is matched when you are importing a named declaration from another fog source file
                else if Token::DoubleColon == tokens[token_idx + 2]
                    && let Token::Identifier(lib_function_name) = &tokens[token_idx + 3]
                {
                    let imported_file_query = imported_file_list.get(&identifier);

                    if Token::SemiColon == tokens[token_idx + 4]
                        && let Some(imported_file) = imported_file_query
                        && let Some(function_def) = imported_file.get(lib_function_name)
                    {
                        // Store the imported function
                        source_imports.insert(lib_function_name.clone(), function_def.clone());

                        // Increment token index
                        token_idx += 4;

                        // Continue looping over the top-level tokens
                        continue;
                    }
                }
            }
            else if let Token::Literal(Type::String(path_to_linked_file)) =
                tokens[token_idx + 1].clone()
            {
                // Turn the String literal into path
                let path = PathBuf::from(format!("src/{path_to_linked_file}")).canonicalize()?;

                // Check if a file exists at that path
                if !fs::exists(&path)?
                    || path.extension().unwrap_or_default().to_string_lossy() == ".f"
                {
                    return Err(ParserError::LinkedSourceFileError(path).into());
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

                // panic!("Rework this when working on modules");
                let mut imported_file_mod_path = module_path.clone();

                imported_file_mod_path.push(
                    path.file_name()
                        .ok_or(ParserError::LinkedSourceFileError(path.clone()))?
                        .to_string_lossy()
                        .to_string(),
                );

                // Create a new Parser state
                let mut parser_state =
                    Parser::new(tokens, ProjectConfig::default(), imported_file_mod_path);

                println!(
                    "Imported file from `{}`. Parsing source file...",
                    path.display()
                );

                // Parse the tokens
                parser_state.parse(HashMap::new())?;

                // Save the file's name and the functions it contains so that we can refer to it later.
                imported_file_list.insert(file_name, parser_state.function_table().clone());

                token_idx += 2;
                continue;
            }

            return Err(ParserError::SyntaxError(SyntaxError::InvalidImportDefinition).into());
        }
        else if current_token == Token::Struct {
            if let Some(Token::Identifier(struct_name)) = tokens.get(token_idx + 1) {
                if let Some(Token::OpenBraces) = tokens.get(token_idx + 2) {
                    // Search for the closing brace's index
                    let braces_idx =
                        find_closing_paren(&tokens[token_idx + 3..], 0)? + token_idx + 3;

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
                        if let Token::Identifier(field_name) = current_token
                            && let Token::Colon = &struct_slice[token_idx + 1]
                        {
                            // Check if there is a comma present in the field, if not check if its the end of the struct definition
                            // Or the user did not put a comma at the end of the last field definition. This is expected
                            if Some(&Token::Comma) == struct_slice.get(token_idx + 3)
                                || token_idx + 3 == struct_slice.len()
                            {
                                if let Token::TypeDefinition(field_type) =
                                    &struct_slice[token_idx + 2]
                                {
                                    // Save the field's type and name
                                    struct_fields.insert(field_name.clone(), field_type.clone());

                                    // Increment the token index
                                    token_idx += 4;

                                    // Continue looping through, if the pattern doesnt match the syntax return an error
                                    continue;
                                }
                                else if let Token::Identifier(custom_type) =
                                    &struct_slice[token_idx + 2]
                                    && let Some(custom_item) = custom_items.get(custom_type)
                                {
                                    match custom_item {
                                        CustomType::Struct(struct_def) => {
                                            struct_fields.insert(
                                                field_name.to_string(),
                                                TypeDiscriminant::Struct(struct_def.clone()),
                                            );
                                        },
                                        CustomType::Enum(index_map) => {
                                            todo!()
                                        },
                                    }

                                    // Increment the token index
                                    token_idx += 4;

                                    // Continue looping through, if the pattern doesnt match the syntax return an error
                                    continue;
                                }
                            }
                        }

                        // Return a syntax error
                        return Err(ParserError::SyntaxError(
                            SyntaxError::InvalidStructFieldDefinition,
                        )
                        .into());
                    }

                    // Save the custom item
                    custom_items.insert(
                        struct_name.to_string(),
                        CustomType::Struct((struct_name.clone(), struct_fields.into())),
                    );
                }
            }
            else {
                return Err(ParserError::SyntaxError(SyntaxError::InvalidStructDefinition).into());
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

pub fn parse_functions(
    config: ProjectConfig,
    unparsed_functions: Arc<IndexMap<String, UnparsedFunctionDefinition>>,
    function_imports: Arc<HashMap<String, FunctionSignature>>,
    custom_items: Arc<IndexMap<String, CustomType>>,
    module_path: Vec<String>,
) -> Result<IndexMap<String, FunctionDefinition>>
{
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
                module_path.clone(),
                OrdMap::new(),
            )?,
        };

        println!(
            "Parsed function `{}({})::{fn_name}` ({}/{})",
            module_path.join("::"),
            config.version,
            fn_idx + 1,
            unparsed_functions.len()
        );
        parsed_functions.insert(fn_name.clone(), function_definition);
    }

    Ok(parsed_functions)
}

pub fn parse_function_block(
    tokens: Vec<Token>,
    function_signatures: Arc<IndexMap<String, UnparsedFunctionDefinition>>,
    this_function_signature: FunctionSignature,
    function_imports: Arc<HashMap<String, FunctionSignature>>,
    custom_items: Arc<IndexMap<String, CustomType>>,
    this_fn_args: FunctionArguments,
    module_path: Vec<String>,

    additional_variables: OrdMap<String, TypeDiscriminant>,
) -> Result<Vec<ParsedToken>>
{
    // Check if the function defined by the source code does not have an indeterminate amount of args
    if this_fn_args.ellipsis_present {
        return Err(ParserError::DeterminiateArgumentsFunction.into());
    }

    let mut token_idx = 0;

    let mut variable_scope = this_fn_args.arguments_list.clone();
    
    variable_scope.extend(additional_variables.iter().map(|(var_name, var_ty)| (var_name.clone(), var_ty.clone())));

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
                            .position(|token| *token == Token::SemiColon)
                            .ok_or(ParserError::SyntaxError(SyntaxError::MissingLineBreak))?
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
                    }
                    else {
                        parsed_tokens.push(ParsedToken::NewVariable(
                            var_name.clone(),
                            var_type.clone(),
                            Box::new(ParsedToken::Literal(var_type.clone().into())),
                        ));

                        variable_scope.insert(var_name.clone(), var_type.clone());

                        token_idx += 2;
                    }

                    if tokens[token_idx] == Token::SemiColon {
                        token_idx += 1;

                        continue;
                    }
                    else {
                        return Err(ParserError::SyntaxError(SyntaxError::MissingLineBreak).into());
                    }
                }
                else {
                    return Err(
                        ParserError::SyntaxError(SyntaxError::InvalidStatementDefinition).into(),
                    );
                }
            }
            else if let Token::Identifier(ref ident_name) = current_token {
                // If the variable exists in the current scope
                if let Some(variable_type) = variable_scope.get(ident_name).cloned() {
                    // Increment the token index
                    token_idx += 1;

                    // Parse the variable's expression
                    let variable_ref = VariableReference::BasicReference(ident_name.to_string());

                    parse_variable_expression(
                        &tokens,
                        &tokens[token_idx],
                        &mut token_idx,
                        function_signatures.clone(),
                        function_imports.clone(),
                        &mut variable_scope,
                        variable_type,
                        custom_items.clone(),
                        ParsedToken::VariableReference(variable_ref),
                        &mut parsed_tokens,
                    )?;
                }
                else if let Some(function_sig) = function_signatures.get(ident_name) {
                    // If after the function name the first thing isnt a `(` return a syntax error.
                    if tokens[token_idx + 1] != Token::OpenParentheses {
                        return Err(ParserError::SyntaxError(
                            SyntaxError::InvalidFunctionDefinition,
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
                }
                else if let Some(function_sig) = function_imports.get(ident_name) {
                    // If after the function name the first thing isnt a `(` return a syntax error.
                    if tokens[token_idx + 1] != Token::OpenParentheses {
                        return Err(ParserError::SyntaxError(
                            SyntaxError::InvalidFunctionDefinition,
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
                }
                else if let Some(custom_type) = custom_items.get(ident_name) {
                    match custom_type {
                        CustomType::Struct(struct_instance) => {
                            let variable_type = TypeDiscriminant::Struct(struct_instance.clone());
                            token_idx += 1;

                            if let Some(Token::Identifier(var_name)) = tokens.get(token_idx)
                                && let Some(Token::SetValue) = tokens.get(token_idx + 1)
                            {
                                let line_break_idx = tokens
                                    .iter()
                                    .skip(token_idx)
                                    .position(|token| *token == Token::SemiColon)
                                    .ok_or({
                                        ParserError::SyntaxError(SyntaxError::MissingLineBreak)
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
                        },
                        CustomType::Enum(enum_types) => {},
                    };
                }
                else {
                    return Err(ParserError::VariableNotFound(ident_name.clone()).into());
                }
            }
            else if Token::Return == current_token {
                has_return = true;

                token_idx += 1;

                let next_token = &tokens[token_idx];

                if this_function_signature.return_type.clone() == TypeDiscriminant::Void {
                    if *next_token != Token::SemiColon {
                        return Err(ParserError::SyntaxError(
                            SyntaxError::InvalidStatementDefinition,
                        )
                        .into());
                    }
                }
                else {
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
            }
            else if Token::If == current_token {
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
                                debug_attributes: None,
                                module_path: module_path.clone(),
                                visibility: fog_common::parser::FunctionVisibility::Branch,
                            },
                            function_imports.clone(),
                            custom_items.clone(),
                            this_fn_args.clone(),
                            module_path.clone(),
                            variable_scope.clone(),
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
                                        debug_attributes: None,
                                        visibility: fog_common::parser::FunctionVisibility::Branch,
                                        module_path: module_path.clone(),
                                    },
                                    function_imports.clone(),
                                    custom_items.clone(),
                                    this_fn_args.clone(),
                                    module_path.clone(),
                            variable_scope.clone(),
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
            }
            else if Token::Loop == current_token {
                token_idx += 1;

                if let Token::OpenBraces = tokens[token_idx] {
                    token_idx += 1;

                    let paren_close_idx = find_closing_braces(&tokens[token_idx..], 0)? + token_idx;

                    // This is what we have to evaulate in order to execute the appropriate branch of the if statement
                    let loop_body_tokens = &tokens[token_idx..paren_close_idx];

                    let loop_body = parse_function_block(
                        loop_body_tokens.to_vec(),
                        function_signatures.clone(),
                        FunctionSignature {
                            args: FunctionArguments::new(),
                            return_type: TypeDiscriminant::Void,
                            debug_attributes: None,
                            visibility: fog_common::parser::FunctionVisibility::Branch,
                            module_path: module_path.clone(),
                        },
                        function_imports.clone(),
                        custom_items.clone(),
                        this_fn_args.clone(),
                        module_path.clone(),
                            variable_scope.clone(),
                    )?;

                    token_idx = paren_close_idx + 1;

                    parsed_tokens.push(ParsedToken::Loop(loop_body));

                    continue;
                }

                return Err(ParserError::SyntaxError(SyntaxError::InvalidLoopBody).into());
            }
            else if Token::Continue == current_token {
                parsed_tokens.push(ParsedToken::ControlFlow(ControlFlowType::Continue));

                token_idx += 1;
            }
            else if Token::Break == current_token {
                parsed_tokens.push(ParsedToken::ControlFlow(ControlFlowType::Break));

                token_idx += 1;
            }

            token_idx += 1;
        }
    }

    // If there isnt a returned value and the returned type isnt `Void` raise an error
    if !has_return && this_function_signature.return_type != TypeDiscriminant::Void {
        return Err(ParserError::SyntaxError(SyntaxError::FunctionRequiresReturn).into());
    }

    Ok(parsed_tokens)
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
    OrdMap<FunctionArgumentIdentifier<String, usize>, (ParsedToken, TypeDiscriminant)>,
    usize,
)>
{
    let mut tokens_idx = 0;

    let args_list_len = tokens[tokens_idx..].len() + tokens_idx;

    // Arguments which will passed in to the function
    let mut arguments: OrdMap<
        FunctionArgumentIdentifier<String, usize>,
        (ParsedToken, TypeDiscriminant),
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

    if !this_function_args.arguments_list.is_empty() {
        return Err(ParserError::InvalidFunctionArgumentCount.into());
    }

    Ok((arguments, tokens_idx))
}
