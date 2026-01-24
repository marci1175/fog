use std::{
    collections::{HashMap, HashSet},
    fs, mem,
    path::PathBuf,
    rc::Rc,
};

use common::{
    anyhow::{self, Result},
    codegen::{CustomItem, If, StructAttributes},
    compiler::ProjectConfig,
    dashmap::DashMap,
    error::{parser::ParserError, syntax::SyntaxError},
    indexmap::IndexMap,
    parser::{
        common::{ParsedToken, ParsedTokenInstance, find_closing_braces, find_closing_paren},
        dbg::fetch_and_merge_debug_information,
        function::{
            self, CompilerHint, FunctionArguments, FunctionDefinition, FunctionSignature,
            FunctionVisibility, UnparsedFunctionDefinition, parse_function_call_args,
            parse_signature_argument_tokens,
        },
        import::parse_import_path,
        value::parse_value,
        variable::{
            ControlFlowType, UniqueId, VARIABLE_ID_SOURCE, VariableReference,
            resolve_variable_expression,
        },
    },
    tokenizer::Token,
    tracing::info,
    ty::{OrdMap, OrdSet, Type, Value, ty_from_token},
};

use crate::{parser_instance::Parser, tokenizer::tokenize};

/// This function parses all of the functions found in the Token slice.
/// The returned functions still need to be parsed.
pub fn parse_functions(
    tokens: &[Token],
    // Enabled features for the current project
    enabled_features: &OrdSet<String>,
    // Module path for the current struct we are implmenting the functions for
    module_path: Vec<String>,
    // Custom types created by the user above this implementation body
    custom_types: &IndexMap<String, CustomItem>,
    // This argument basically sets whether functions are allowed to use `this` in their arguments.
    // Functions implemented for struct can use this to reference themselves.
    is_struct_implementation: bool,
) -> Result<IndexMap<String, UnparsedFunctionDefinition>, anyhow::Error>
{
    let mut function_list: IndexMap<String, UnparsedFunctionDefinition> = IndexMap::new();
    let mut function_compiler_hint_buffer: OrdSet<CompilerHint> = OrdSet::new();
    let mut function_enabling_feature: OrdSet<String> = OrdSet::new();
    let mut token_idx = 0;

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
                        let (bracket_close_idx, args) = parse_signature_argument_tokens(
                            &tokens[token_idx + 3..],
                            custom_types,
                            is_struct_implementation,
                        )?;

                        token_idx += bracket_close_idx + 3;

                        // Fetch the returned type of the function
                        if tokens[token_idx + 1] == Token::Colon {
                            let return_type = ty_from_token(&tokens[token_idx + 2], custom_types)?;

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

                                // Extract the compiler hints for the function
                                let compiler_hints: OrdSet<function::CompilerHint> =
                                    mem::take(&mut function_compiler_hint_buffer);

                                let function_enabling_features =
                                    mem::take(&mut function_enabling_feature);

                                if !function_enabling_features.is_disjoint(enabled_features)
                                    || function_enabling_features.is_empty()
                                {
                                    // Create a clone of the module path so we can modifiy it locally
                                    let mut mod_path = module_path.clone();

                                    // Store the function name in the module path
                                    mod_path.push(function_name.clone());

                                    // Store the function
                                    let insertion = function_list.insert(
                                        function_name.clone(),
                                        UnparsedFunctionDefinition {
                                            inner: braces_contains.clone(),
                                            token_offset: token_idx + 4,
                                            signature: FunctionSignature {
                                                name: function_name.clone(),
                                                args: args.clone(),
                                                return_type: return_type.clone(),
                                                // To be honest I dont really think this matters what we set it, since im not planning to make a disctinction between public and private functions
                                                // For now ;)
                                                visibility: current_token.try_into()?,
                                                module_path: mod_path.clone(),
                                                compiler_hints: compiler_hints.clone(),
                                                enabling_features: function_enabling_features
                                                    .clone(),
                                            },
                                        },
                                    );

                                    // If a function with a similar name exists throw an error as there is no function overloading an excpetion is when they are covered under different features
                                    if let Some(overwritten_function) = insertion {
                                        return Err(ParserError::SyntaxError(
                                            SyntaxError::DuplicateFunctions(
                                                function_name,
                                                overwritten_function.signature,
                                            ),
                                        )
                                        .into());
                                    }
                                }

                                // Set the iterator index
                                token_idx += braces_contains_len + 5;

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
        else {
            return Err(ParserError::InvalidImplItem.into());
        }

        token_idx += 1;
    }

    Ok(function_list)
}

impl Parser
{
    /// Creates signature table
    /// Returns all of the custom types, etc
    pub fn create_signature_table(
        &self,
        dep_fn_list: Rc<DashMap<Vec<String>, FunctionSignature>>,
    ) -> Result<(
        IndexMap<String, UnparsedFunctionDefinition>,
        HashSet<Vec<String>>,
        HashMap<String, FunctionSignature>,
        IndexMap<String, CustomItem>,
        HashMap<Vec<String>, FunctionDefinition>,
    )>
    {
        let tokens = self.tokens.clone();
        let enabled_features = self.enabled_features.clone();
        let module_path = self.module_path.clone();
        let project_config = self.config.clone();

        let mut token_idx = 0;

        /*
            TODO: Make it so that this function has a side effect on main instead of storing all the imports here.
            TODO: Recode the importing in the source code.
        */

        let mut function_list: IndexMap<String, UnparsedFunctionDefinition> = IndexMap::new();
        // The key is the function's name
        let mut external_imports: HashMap<String, FunctionSignature> = HashMap::new();
        let mut dependency_imports: HashSet<Vec<String>> = HashSet::new();
        let mut imported_file_list: HashMap<Vec<String>, FunctionDefinition> = HashMap::new();

        let mut function_compiler_hint_buffer: OrdSet<CompilerHint> = OrdSet::new();
        let mut function_enabling_feature: OrdSet<String> = OrdSet::new();

        let mut custom_types: IndexMap<String, CustomItem> = IndexMap::new();

        while token_idx < tokens.len() {
            let current_token = tokens[token_idx].clone();
            if current_token == Token::Private
                || current_token == Token::Public
                || current_token == Token::PublicLibrary
            {
                token_idx += 1;

                if tokens[token_idx] == Token::Function {
                    if let Token::Identifier(function_name) = tokens[token_idx + 1].clone() {
                        // Check if the name of the function is correct
                        if function_name.starts_with("__internal") {
                            return Err(ParserError::FunctionNameReserved.into());
                        }

                        if tokens[token_idx + 2] == Token::OpenParentheses {
                            let (bracket_close_idx, args) = parse_signature_argument_tokens(
                                &tokens[token_idx + 3..],
                                &custom_types,
                                false,
                            )?;

                            token_idx += bracket_close_idx + 3;

                            // Fetch the returned type of the function
                            if tokens[token_idx + 1] == Token::Colon {
                                let return_type =
                                    ty_from_token(&tokens[token_idx + 2], &custom_types)?;

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
                                        else if tokens_slice[token_braces_idx]
                                            == Token::CloseBraces
                                        {
                                            brace_layer_counter -= 1;
                                        }

                                        // If we have arrived at the end of the braces this is when we know that this is the end of the function's scope
                                        if brace_layer_counter == 0 {
                                            break;
                                        }

                                        // Store the current item in the token buffer
                                        braces_contains
                                            .push(tokens_slice[token_braces_idx].clone());

                                        // Increment the index
                                        token_braces_idx += 1;
                                    }

                                    let braces_contains_len = braces_contains.len();

                                    // Extract the compiler hints for the function
                                    let compiler_hints: OrdSet<function::CompilerHint> =
                                        mem::take(&mut function_compiler_hint_buffer);

                                    let function_enabling_features =
                                        mem::take(&mut function_enabling_feature);

                                    if !function_enabling_features.is_disjoint(&enabled_features)
                                        || function_enabling_features.is_empty()
                                    {
                                        // Create a clone of the module path so we can modifiy it locally
                                        let mut mod_path = module_path.clone();

                                        // Store the function name in the module path
                                        mod_path.push(function_name.clone());

                                        // Store the function
                                        let insertion = function_list.insert(
                                            function_name.clone(),
                                            UnparsedFunctionDefinition {
                                                inner: braces_contains.clone(),
                                                token_offset: token_idx + 4,
                                                signature: FunctionSignature {
                                                    name: function_name.clone(),
                                                    args: args.clone(),
                                                    return_type: return_type.clone(),
                                                    visibility: current_token.try_into()?,
                                                    module_path: mod_path.clone(),
                                                    compiler_hints: compiler_hints.clone(),
                                                    enabling_features: function_enabling_features
                                                        .clone(),
                                                },
                                            },
                                        );

                                        // If a function with a similar name exists throw an error as there is no function overloading an excpetion is when they are covered under different features
                                        if let Some(overwritten_function) = insertion {
                                            return Err(ParserError::SyntaxError(
                                                SyntaxError::DuplicateFunctions(
                                                    function_name,
                                                    overwritten_function.signature,
                                                ),
                                            )
                                            .into());
                                        }
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
                        return Err(
                            ParserError::SyntaxError(SyntaxError::InvalidFunctionName).into()
                        );
                    }
                }
            }
            else if current_token == Token::Function {
                return Err(ParserError::FunctionRequiresExplicitVisibility.into());
            }
            else if current_token == Token::External {
                if let Some(Token::Identifier(identifier)) = tokens.get(token_idx + 1).cloned()
                    && tokens[token_idx + 2] == Token::OpenParentheses
                {
                    if external_imports.get(&identifier).is_some()
                        || function_list.get(&identifier).is_some()
                    {
                        return Err(ParserError::DuplicateSignatureImports(identifier).into());
                    }

                    let mut mod_path = module_path.clone();
                    mod_path.push(identifier.clone());

                    // Set the index to the item after the `(` for the helper function
                    token_idx += 3;

                    let fn_sig = parse_function_signature(
                        &tokens,
                        &mut token_idx,
                        &custom_types,
                        mod_path,
                        identifier,
                        false,
                    )?;

                    external_imports.insert(fn_sig.name.clone(), fn_sig);

                    continue;
                }
            }
            else if current_token == Token::Import {
                if let Some(Token::Identifier(_)) = tokens.get(token_idx + 1) {
                    let (import_path, idx) = parse_import_path(&tokens[token_idx + 1..])?;

                    token_idx += idx + 1;

                    dependency_imports.insert(import_path);

                    continue;
                }
                else if let Token::Literal(Value::String(path_to_linked_file)) =
                    tokens[token_idx + 1].clone()
                {
                    // Turn the String literal into path
                    let path =
                        PathBuf::from(format!("src/{path_to_linked_file}")).canonicalize()?;

                    // Check if a file exists at that path
                    if !fs::exists(&path)?
                        || path.extension().unwrap_or_default().to_string_lossy() == ".f"
                    {
                        return Err(ParserError::LinkedSourceFileError(path).into());
                    }

                    // Get the File's content
                    let file_contents = fs::read_to_string(&path)?;

                    // Tokenize the raw source file
                    let (tokens, token_ranges, _) = tokenize(&file_contents, None)?;

                    // Create a new Parser state
                    let mut parser_state = Parser::new(
                        tokens,
                        token_ranges,
                        ProjectConfig::default(),
                        vec![
                            path.file_prefix()
                                .ok_or(ParserError::LinkedSourceFileError(path.clone()))?
                                .to_string_lossy()
                                .to_string(),
                        ],
                        enabled_features.clone(),
                    );

                    // Parse the tokens
                    parser_state.parse(dep_fn_list.clone())?;

                    // Save the file's name and the functions it contains so that we can refer to it later.
                    imported_file_list.extend(parser_state.function_table().clone().iter().map(
                        |(_fn_name, fn_entry)| {
                            (fn_entry.signature.module_path.clone(), fn_entry.clone())
                        },
                    ));

                    info!("Imported file `{}`.", path.display());

                    token_idx += 2;

                    continue;
                }

                return Err(ParserError::SyntaxError(SyntaxError::InvalidImportDefinition).into());
            }
            else if current_token == Token::Struct {
                if let Some(Token::Identifier(struct_name)) = tokens.get(token_idx + 1)
                    && let Some(Token::OpenBraces) = tokens.get(token_idx + 2)
                {
                    // Search for the closing brace's index
                    let braces_idx =
                        find_closing_braces(&tokens[token_idx + 3..], 0)? + token_idx + 3;

                    // Retrive the tokens from the braces
                    let struct_slice = tokens[token_idx + 3..braces_idx].to_vec();

                    // Create a list for the struct fields
                    let mut struct_fields: IndexMap<String, Type> = IndexMap::new();

                    // Store the idx
                    let mut body_idx = 0;

                    // Parse the struct fields
                    while body_idx < struct_slice.len() {
                        // Get the current token
                        let current_token = &struct_slice[body_idx];

                        // Pattern match the syntax
                        if let Token::Identifier(field_name) = current_token
                            && let Token::Colon = &struct_slice[body_idx + 1]
                        {
                            // Check if there is a comma present in the field, if not check if its the end of the struct definition
                            // Or the user did not put a comma at the end of the last field definition. This is expected
                            if Some(&Token::Comma) == struct_slice.get(body_idx + 3)
                                || body_idx + 3 == struct_slice.len()
                            {
                                if let Token::TypeDefinition(field_type) =
                                    &struct_slice[body_idx + 2]
                                {
                                    // Save the field's type and name
                                    struct_fields.insert(field_name.clone(), field_type.clone());

                                    // Increment the token index
                                    body_idx += 4;

                                    // Continue looping through, if the pattern doesnt match the syntax return an error
                                    continue;
                                }
                                else if let Token::Identifier(custom_type) =
                                    &struct_slice[body_idx + 2]
                                    && let Some(custom_item) = custom_types.get(custom_type)
                                {
                                    match custom_item {
                                        CustomItem::Struct(struct_def) => {
                                            struct_fields.insert(
                                                field_name.to_string(),
                                                Type::Struct(struct_def.clone()),
                                            );
                                        },
                                        CustomItem::Enum((ty, enum_body)) => {
                                            struct_fields.insert(
                                                field_name.to_string(),
                                                Type::Enum((
                                                    Box::new(ty.clone()),
                                                    enum_body.clone(),
                                                )),
                                            );
                                        },
                                        CustomItem::Trait { name, .. } => {
                                            return Err(
                                                ParserError::TraitNotObject(name.clone()).into()
                                            );
                                        },
                                    }

                                    // Increment the token index
                                    body_idx += 4;

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
                    custom_types.insert(
                        struct_name.to_string(),
                        CustomItem::Struct((
                            struct_name.clone(),
                            struct_fields.into(),
                            // Create a new attributes instance
                            StructAttributes::default(),
                        )),
                    );

                    token_idx = braces_idx + 1;
                    continue;
                }

                return Err(ParserError::SyntaxError(SyntaxError::InvalidStructDefinition).into());
            }
            else if current_token == Token::Trait {
                if let Some(Token::Identifier(trait_name)) = tokens.get(token_idx + 1)
                    && let Some(Token::OpenBraces) = tokens.get(token_idx + 2)
                {
                    // Set idx to the next token
                    token_idx += 3;

                    // Find closing braces
                    let closing_idx = find_closing_braces(&tokens[token_idx..], 0)? + token_idx;

                    // The trait body's token slice
                    let trait_body = &tokens[token_idx..closing_idx];

                    // Local slice indexing
                    let mut idx = 0;

                    // Store the trait's functions we have parsed
                    let mut trait_functions: OrdMap<String, FunctionSignature> = OrdMap::new();

                    // Parse the entire body
                    while trait_body.len() > idx {
                        if let Some(Token::Identifier(fn_name)) = trait_body.get(idx) {
                            // Check that we are not redefining any functions
                            if trait_functions.contains_key(fn_name) {
                                return Err(
                                    ParserError::FunctionRedefinition(fn_name.clone()).into()
                                );
                            }

                            // Modify module path
                            let mut module_path = module_path.clone();

                            // Store trait name in mod path
                            module_path.push(fn_name.clone());

                            // Increment index
                            idx += 2;

                            // This function parses until the return type and offset the idx to the expr closing `;`
                            let fn_sig = parse_function_signature(
                                trait_body,
                                &mut idx,
                                &custom_types,
                                module_path,
                                fn_name.clone(),
                                true,
                            )?;

                            // Store the function of the trait
                            trait_functions.insert(fn_name.clone(), fn_sig);

                            // Increment by one to go the next fn sig
                            idx += 1;
                        }
                        else {
                            return Err(ParserError::InvalidTraitItem.into());
                        }
                    }

                    // Set the correct index position
                    token_idx = closing_idx;

                    // Store custom type
                    custom_types.insert(
                        trait_name.clone(),
                        CustomItem::Trait {
                            name: trait_name.clone(),
                            functions: trait_functions,
                        },
                    );
                }
                else {
                    return Err(
                        ParserError::SyntaxError(SyntaxError::InvalidTraitDefinition).into(),
                    );
                }
            }
            else if current_token == Token::CompilerHintSymbol {
                token_idx += 1;

                if let Token::CompilerHint(compiler_hint) = &tokens[token_idx] {
                    if *compiler_hint == CompilerHint::Feature {
                        token_idx += 1;

                        if let Some(Token::Literal(Value::String(feature_name))) =
                            tokens.get(token_idx)
                        {
                            if let Some(available_features) = &project_config.features
                                && !available_features.contains(feature_name)
                            {
                                return Err(ParserError::InvalidFeatureRequirement(
                                    feature_name.clone(),
                                    available_features.clone(),
                                )
                                .into());
                            }
                            function_enabling_feature.insert(feature_name.clone());
                        }
                        else {
                            return Err(ParserError::InvalidFunctionFeature(
                                tokens.get(token_idx).cloned(),
                            )
                            .into());
                        }
                    }
                    else {
                        function_compiler_hint_buffer.insert(compiler_hint.clone());
                    }
                }
                else {
                    return Err(ParserError::InvalidCompilerHint(tokens[token_idx].clone()).into());
                }
            }
            else if let Token::Enum(ty) = current_token.clone() {
                let is_ty_inferred = ty.is_none();
                // If there is an inner type try to fetch.
                // If the inner function fails it raises an error.
                // If there was no pre defined type we will use `U32` as default.
                let variant_type = ty
                    .map(|inner| ty_from_token(&inner, &custom_types))
                    .unwrap_or(Ok(Type::U32))?;

                if let Some(Token::Identifier(enum_name)) = tokens.get(token_idx + 1)
                    && let Some(Token::OpenBraces) = tokens.get(token_idx + 2)
                {
                    // SeRch for the closing brace's index
                    let braces_idx =
                        find_closing_braces(&tokens[token_idx + 3..], 0)? + token_idx + 3;

                    // Retrive the tokens from the braces
                    let variant_body = tokens[token_idx + 3..braces_idx].to_vec();
                    let mut body_idx = 0;

                    let mut variant_fields: OrdMap<String, ParsedTokenInstance> = OrdMap::new();

                    while body_idx < variant_body.len() {
                        if let Some(Token::Identifier(variant_name)) = variant_body.get(body_idx) {
                            if let Some(Token::SetValue) = variant_body.get(body_idx + 1) {
                                body_idx += 2;

                                // This function will stop parsing at `,` or `;` or `)` or at the end of the list
                                let (parsed_token_instance, idx, _ty) = parse_value(
                                    &variant_body[body_idx..],
                                    0,
                                    &self.tokens_debug_info,
                                    token_idx,
                                    Rc::new(function_list.clone()),
                                    &mut IndexMap::new(),
                                    Some(variant_type.clone()),
                                    self.imported_functions.clone(),
                                    Rc::new(custom_types.clone()),
                                )?;

                                body_idx += idx;

                                // Check correct signature by checking if we are currently at a `,` or at the end of the token list
                                if variant_body.get(body_idx) == Some(&Token::Comma)
                                    || body_idx == variant_body.len()
                                {
                                    // Store enum variant
                                    variant_fields
                                        .insert(variant_name.clone(), parsed_token_instance);

                                    // If we are not at the end of the list increment the body_idx by one.
                                    if body_idx != variant_body.len() {
                                        // Increment index and iterate once again
                                        body_idx += 1;
                                    }

                                    continue;
                                }
                            }
                            // Check if we can infer value
                            else if is_ty_inferred {
                                // Get which enum variant is this one
                                let nth = variant_fields.len();

                                // Store the variant inferred value
                                variant_fields.insert(
                                    variant_name.clone(),
                                    ParsedTokenInstance {
                                        inner: ParsedToken::Literal(Value::U32(nth as u32)),
                                        debug_information: fetch_and_merge_debug_information(
                                            &self.tokens_debug_info,
                                            token_idx + body_idx..token_idx + body_idx + 2,
                                            true,
                                        )
                                        .unwrap(),
                                    },
                                );

                                // Check for correct syntax
                                if let Some(Token::Comma) = variant_body.get(body_idx + 1) {
                                    body_idx += 2;

                                    continue;
                                }
                            }
                        }

                        return Err(ParserError::SyntaxError(
                            SyntaxError::InvalidEnumBodyDefinition,
                        )
                        .into());
                    }

                    // Store custom type
                    custom_types.insert(
                        enum_name.clone(),
                        CustomItem::Enum((variant_type, variant_fields)),
                    );

                    token_idx = braces_idx + 1;

                    continue;
                }

                return Err(
                    ParserError::SyntaxError(SyntaxError::CustomTypeRequiresName(current_token))
                        .into(),
                );
            }
            else if let Token::Identifier(struct_name_ident) = current_token.clone() {
                // Create a clone so that we can use this when looking up information, since it would create an immutable and a mutable borrow.
                let custom_types_clone = custom_types.clone();

                // Check if this identifier is a struct.
                // Syntax should be like this
                // <struct-name> implements (trait name) { <functions> }
                if let Some(CustomItem::Struct((_, _, attributes))) =
                    custom_types.get_mut(&struct_name_ident)
                {
                    token_idx += 1;

                    // Raise error if the next token isnt `Token::Implements` because of syntax
                    if tokens.get(token_idx) != Some(&Token::Implements) {
                        return Err(
                            ParserError::SyntaxError(SyntaxError::InvalidFunctionImplDef).into(),
                        );
                    }

                    token_idx += 1;

                    // Fetch the next token
                    let next_token = tokens.get(token_idx);

                    // Fetch the implementation body
                    let (implementation_body, trait_to_impl) =
                        if let Some(Token::Identifier(trait_name)) = next_token {
                            // Increment idx to the Token::OpenBraces
                            token_idx += 1;

                            if let Some(CustomItem::Trait { functions, .. }) =
                                custom_types_clone.get(trait_name)
                                && Some(&Token::OpenBraces) == tokens.get(token_idx)
                            {
                                // Move the idx into the impl body
                                token_idx += 1;

                                // We will remove each function as we parse them, so that we can check that every function is implemented
                                let impl_fns = functions.clone();

                                // Fetch closing idx
                                let closing_idx =
                                    find_closing_braces(&tokens[token_idx..], 0)? + token_idx;

                                // The token slice of the tokens that contain all the function implementations for this trait
                                let impl_body = &tokens[token_idx..closing_idx];

                                // Closing idx is the `}` so increment one more
                                token_idx = closing_idx;

                                (impl_body, Some((impl_fns, trait_name.clone())))
                            }
                            else {
                                return Err(ParserError::SyntaxError(
                                    SyntaxError::InvalidFunctionImplDef,
                                )
                                .into());
                            }
                        }
                        // Parse the functions we have implmemented for the struct
                        else if let Some(Token::OpenBraces) = next_token {
                            // Move the idx into the impl body
                            token_idx += 1;

                            // Fetch closing idx
                            let closing_idx =
                                find_closing_braces(&tokens[token_idx..], 0)? + token_idx;

                            // The token slice of the tokens that contain all the function implementations
                            let impl_body = &tokens[token_idx..closing_idx];

                            // Closing idx is the `}` so increment one more
                            token_idx = closing_idx;

                            (impl_body, None)
                        }
                        // If the other cases dont match return an error
                        else {
                            return Err(ParserError::SyntaxError(
                                SyntaxError::InvalidFunctionImplDef,
                            )
                            .into());
                        };

                    let mut module_path = module_path.clone();

                    // Store the struct's name in the module path
                    module_path.push(struct_name_ident.clone());

                    // Parse the functions we have implmemented with the Trait
                    let functions = parse_functions(
                        implementation_body,
                        &enabled_features,
                        module_path.clone(),
                        &custom_types_clone,
                        // Allow the usage of `this`
                        true,
                    )?;

                    if let Some((mut fns_to_impl, trait_name)) = trait_to_impl {
                        // Check if all of the trait's functions have been implemented
                        fns_to_impl.retain(|k, v| {
                            // Remove those functions from this local list that match the trait fn's signature, meaning that they are correct
                            // Only keep those that are invalid
                            if let Some(impled_fn) = functions.get(k) {
                                // Check sig eq
                                !(impled_fn.signature.name == *v.name
                                    && impled_fn.signature.args.check_arg_eq(&v.args)
                                    && impled_fn.signature.return_type == v.return_type
                                    && impled_fn.signature.args.receiver_referenced
                                        == v.args.receiver_referenced)
                            }
                            else {
                                true
                            }
                        });

                        // Check if there are any functions left in the list and return an error if there are
                        // This means that functions werent or werent correctly implemented
                        if !fns_to_impl.is_empty() {
                            return Err(ParserError::InvalidTraitImplementation(
                                fns_to_impl
                                    .keys()
                                    .map(|k| k.to_string())
                                    .collect::<Vec<String>>(),
                                trait_name,
                            )
                            .into());
                        }

                        // Store the implemented trait
                        attributes
                            .traits
                            .insert(trait_name, OrdMap::from(functions));

                        continue;
                    }

                    attributes
                        .implemented_unparsed_functions
                        .extend(functions.iter().map(|(n, d)| (n.to_owned(), d.to_owned())));
                }
                else {
                    return Err(ParserError::CustomItemNotFound(struct_name_ident.clone()).into());
                }
            }

            token_idx += 1;
        }

        dbg!(custom_types.get("marci"));

        Ok((
            function_list,
            dependency_imports,
            external_imports,
            custom_types,
            imported_file_list,
        ))
    }

    pub fn parse_functions(
        &self,
        unparsed_functions: Rc<IndexMap<String, UnparsedFunctionDefinition>>,
        function_imports: Rc<HashMap<String, FunctionSignature>>,
        custom_items: &mut IndexMap<String, CustomItem>,
    ) -> Result<IndexMap<String, FunctionDefinition>>
    {
        let config = self.config.clone();
        let module_path = self.module_path.clone();

        let custom_items_clone = Rc::new(custom_items.clone());

        // Parse the struct implementations
        for (_, item) in custom_items.iter_mut() {
            if let CustomItem::Struct((_, _, attr)) = item {
                for (fn_name, def) in attr.implemented_unparsed_functions.iter() {
                    let impl_definition = FunctionDefinition {
                        signature: def.signature.clone(),
                        inner: self.parse_function_block(
                            def.inner.clone(),
                            def.token_offset,
                            // TODO: Improve this
                            Rc::new(IndexMap::new()),
                            def.signature.clone(),
                            function_imports.clone(),
                            // TODO: And this
                            custom_items_clone.clone(),
                            def.signature.args.clone(),
                            OrdMap::new(),
                        )?,
                        token_offset: def.token_offset,
                    };

                    // Check where we should put this function.
                    // If there is a receiver in the function we will store it in the struct attribute
                    // If there isnt we will store this function just like an import but with the struct's name added to the path
                    if impl_definition.signature.args.receiver_referenced {
                        // Check for name collision in impls
                        if attr
                            .implemented_parsed_functions
                            .insert(fn_name.clone(), impl_definition)
                            .is_some()
                        {
                            return Err(ParserError::FunctionRedefinition(fn_name.clone()).into());
                        }
                    }
                    else {
                        // TODO: When recoding imports we should also make it so that people are able to import functions in the source code. THen well be able to use this part
                        unimplemented!()
                    }
                }
            }
        }

        let mut parsed_functions: IndexMap<String, FunctionDefinition> = IndexMap::new();

        // Parse the functions themselves
        for (fn_idx, (fn_name, unparsed_function)) in unparsed_functions.clone().iter().enumerate()
        {
            let function_definition = FunctionDefinition {
                signature: unparsed_function.signature.clone(),
                inner: self.parse_function_block(
                    unparsed_function.inner.clone(),
                    unparsed_function.token_offset,
                    unparsed_functions.clone(),
                    unparsed_function.signature.clone(),
                    function_imports.clone(),
                    custom_items_clone.clone(),
                    unparsed_function.signature.args.clone(),
                    OrdMap::new(),
                )?,
                token_offset: unparsed_function.token_offset,
            };

            info!(
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
        &self,
        tokens: Vec<Token>,
        function_token_offset: usize,
        function_signatures: Rc<IndexMap<String, UnparsedFunctionDefinition>>,
        this_function_signature: FunctionSignature,
        function_imports: Rc<HashMap<String, FunctionSignature>>,
        custom_items: Rc<IndexMap<String, CustomItem>>,
        this_fn_args: FunctionArguments,
        additional_variables: OrdMap<String, (Type, UniqueId)>,
    ) -> Result<Vec<ParsedTokenInstance>>
    {
        let module_path = self.module_path.clone();

        // Check if the function defined by the source code does not have an indeterminate amount of args
        if this_fn_args.ellipsis_present {
            return Err(ParserError::DeterminiateArgumentsFunction.into());
        }

        let mut token_idx = 0;

        let mut variable_scope = this_fn_args.arguments.clone();

        variable_scope.extend(
            additional_variables
                .iter()
                .map(|(var_name, var_ty)| (var_name.clone(), var_ty.clone())),
        );

        let mut parsed_token_instances: Vec<ParsedTokenInstance> = Vec::new();

        let mut has_return = false;

        if !tokens.is_empty() {
            while token_idx < tokens.len() {
                // Store the token index at the beginning of the iteration.
                let origin_token_idx = token_idx;
                let current_token = tokens[token_idx].clone();

                if let Token::TypeDefinition(var_type) = current_token {
                    if let Token::Identifier(var_name) = tokens[token_idx + 1].clone() {
                        let unique_variable_id = VARIABLE_ID_SOURCE.get_unique_id();

                        if tokens[token_idx + 2] == Token::SetValue {
                            let line_break_idx = tokens
                                .iter()
                                .skip(token_idx + 2)
                                .position(|token| *token == Token::SemiColon)
                                .ok_or(ParserError::SyntaxError(SyntaxError::MissingSemiColon))?
                                + token_idx
                                + 2;

                            let selected_tokens_range = token_idx + 3..line_break_idx;
                            let selected_tokens = &tokens[selected_tokens_range.clone()];

                            let (parsed_value, _idx, _) = parse_value(
                                selected_tokens,
                                function_token_offset,
                                &self.tokens_debug_info,
                                selected_tokens_range.start,
                                function_signatures.clone(),
                                &mut variable_scope,
                                Some(var_type.clone()),
                                function_imports.clone(),
                                custom_items.clone(),
                            )?;

                            // Set the new idx
                            token_idx = line_break_idx;

                            parsed_token_instances.push(ParsedTokenInstance {
                                inner: ParsedToken::NewVariable {
                                    variable_name: var_name.clone(),
                                    variable_type: var_type.clone(),
                                    variable_value: Box::new(parsed_value),
                                    variable_id: unique_variable_id,
                                    is_mutable: true,
                                },
                                // Checked
                                debug_information: fetch_and_merge_debug_information(
                                    &self.tokens_debug_info,
                                    function_token_offset + origin_token_idx
                                        ..function_token_offset + token_idx + 1,
                                    true,
                                )
                                .unwrap(),
                            });

                            variable_scope.insert(var_name, (var_type.clone(), unique_variable_id));
                        }
                        else {
                            // All variables must have a default value
                            return Err(
                                ParserError::MissingVariableValue(var_name, var_type).into()
                            );
                        }

                        if tokens[token_idx] == Token::SemiColon {
                            token_idx += 1;

                            continue;
                        }
                        else {
                            return Err(
                                ParserError::SyntaxError(SyntaxError::MissingSemiColon).into()
                            );
                        }
                    }
                    else {
                        return Err(ParserError::SyntaxError(
                            SyntaxError::InvalidVariableDefinition,
                        )
                        .into());
                    }
                }
                // Handle operations to variables like `foo[0] = 5`
                else if let Token::Identifier(ref ident_name) = current_token {
                    // If the variable exists in the current scope
                    if let Some(variable_type) = variable_scope.get(ident_name).cloned() {
                        // Increment the token index
                        token_idx += 1;
                        // Put the variable name into a basic reference
                        let variable_ref =
                            VariableReference::BasicReference(ident_name.to_string(), 0);

                        // Token idx copy for the slice indexing
                        // Afaik we should be using token_idx + 1 (we increment above) to correctly index the slice (we would be using ..= otherwise )
                        let token_idx_copy = token_idx;

                        // Parse the expression involving the variable
                        resolve_variable_expression(
                            &tokens,
                            function_token_offset,
                            &self.tokens_debug_info,
                            &mut token_idx,
                            function_signatures.clone(),
                            function_imports.clone(),
                            &mut variable_scope,
                            variable_type,
                            custom_items.clone(),
                            &mut ParsedTokenInstance {
                                inner: ParsedToken::VariableReference(variable_ref),
                                debug_information: fetch_and_merge_debug_information(
                                    &self.tokens_debug_info,
                                    origin_token_idx + function_token_offset
                                        ..token_idx_copy + function_token_offset,
                                    true,
                                )
                                .unwrap(),
                            },
                            &mut parsed_token_instances,
                            ident_name,
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
                            function_token_offset + token_idx + 2,
                            origin_token_idx,
                            &self.tokens_debug_info,
                            &mut variable_scope,
                            function_sig.signature.args.clone(),
                            function_signatures.clone(),
                            function_imports.clone(),
                            custom_items.clone(),
                        )?;

                        token_idx += jumped_idx + 2;

                        parsed_token_instances.push(ParsedTokenInstance {
                            inner: ParsedToken::FunctionCall(
                                (function_sig.signature.clone(), ident_name.clone()),
                                variables_passed,
                            ),
                            debug_information: fetch_and_merge_debug_information(
                                &self.tokens_debug_info,
                                origin_token_idx + function_token_offset
                                    ..origin_token_idx + token_idx + function_token_offset,
                                true,
                            )
                            .unwrap(),
                        });
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
                            function_token_offset,
                            token_idx + 2,
                            &self.tokens_debug_info,
                            &mut variable_scope,
                            function_sig.args.clone(),
                            function_signatures.clone(),
                            function_imports.clone(),
                            custom_items.clone(),
                        )?;

                        token_idx += jumped_idx + 2;

                        parsed_token_instances.push(ParsedTokenInstance {
                            inner: ParsedToken::FunctionCall(
                                (function_sig.clone(), ident_name.clone()),
                                variables_passed,
                            ),
                            debug_information: fetch_and_merge_debug_information(
                                &self.tokens_debug_info,
                                origin_token_idx + function_token_offset
                                    ..function_token_offset + token_idx + 2,
                                true,
                            )
                            .unwrap(),
                        });
                    }
                    else if let Some(custom_type) = custom_items.get(ident_name) {
                        let unique_variable_id = VARIABLE_ID_SOURCE.get_unique_id();

                        match custom_type {
                            CustomItem::Struct(struct_instance) => {
                                let variable_type = Type::Struct(struct_instance.clone());

                                token_idx += 1;

                                // Check the next token after the struct name
                                if let Some(Token::Identifier(var_name)) = tokens.get(token_idx)
                                    && let Some(Token::SetValue) = tokens.get(token_idx + 1)
                                {
                                    let line_break_idx = tokens
                                        .iter()
                                        .skip(token_idx)
                                        .position(|token| *token == Token::SemiColon)
                                        .ok_or({
                                            ParserError::SyntaxError(SyntaxError::MissingSemiColon)
                                        })?
                                        + token_idx;

                                    let selected_tokens_range = token_idx + 2..line_break_idx;
                                    let selected_tokens = &tokens[selected_tokens_range.clone()];

                                    token_idx += selected_tokens.len() + 1;

                                    let (parsed_token, _, _) = parse_value(
                                        selected_tokens,
                                        function_token_offset,
                                        &self.tokens_debug_info,
                                        token_idx,
                                        function_signatures.clone(),
                                        &mut variable_scope,
                                        Some(variable_type.clone()),
                                        function_imports.clone(),
                                        custom_items.clone(),
                                    )?;

                                    parsed_token_instances.push(ParsedTokenInstance {
                                        inner: ParsedToken::NewVariable {
                                            variable_name: var_name.clone(),
                                            variable_type: variable_type.clone(),
                                            variable_value: Box::new(parsed_token),
                                            variable_id: unique_variable_id,
                                            is_mutable: true,
                                        },
                                        debug_information: fetch_and_merge_debug_information(
                                            &self.tokens_debug_info,
                                            origin_token_idx + function_token_offset
                                                ..token_idx + function_token_offset + 2,
                                            true,
                                        )
                                        .unwrap(),
                                    });

                                    variable_scope.insert(
                                        var_name.clone(),
                                        (variable_type, unique_variable_id),
                                    );
                                }
                                else {
                                    // Assume that the user tried to access the struct name as a variable
                                    return Err(
                                        ParserError::VariableNotFound(ident_name.clone()).into()
                                    );
                                }
                            },
                            CustomItem::Enum(inner) => {
                                let variable_type =
                                    Type::Enum((Box::new(inner.0.clone()), inner.1.clone()));

                                token_idx += 1;

                                if let Some(Token::Identifier(var_name)) = tokens.get(token_idx)
                                    && let Some(Token::SetValue) = tokens.get(token_idx + 1)
                                {
                                    let line_break_idx = tokens
                                        .iter()
                                        .skip(token_idx)
                                        .position(|token| *token == Token::SemiColon)
                                        .ok_or({
                                            ParserError::SyntaxError(SyntaxError::MissingSemiColon)
                                        })?
                                        + token_idx;

                                    let selected_tokens_range = token_idx + 2..line_break_idx;
                                    let selected_tokens = &tokens[selected_tokens_range.clone()];

                                    token_idx += selected_tokens.len() + 1;

                                    let (parsed_token, _, _) = parse_value(
                                        selected_tokens,
                                        function_token_offset,
                                        &self.tokens_debug_info,
                                        token_idx,
                                        function_signatures.clone(),
                                        &mut variable_scope,
                                        Some(variable_type.clone()),
                                        function_imports.clone(),
                                        custom_items.clone(),
                                    )?;

                                    parsed_token_instances.push(ParsedTokenInstance {
                                        inner: ParsedToken::NewVariable {
                                            variable_name: var_name.clone(),
                                            variable_type: variable_type.clone(),
                                            variable_value: Box::new(parsed_token),
                                            variable_id: unique_variable_id,
                                            is_mutable: true,
                                        },
                                        debug_information: fetch_and_merge_debug_information(
                                            &self.tokens_debug_info,
                                            origin_token_idx + function_token_offset
                                                ..token_idx + function_token_offset + 2,
                                            true,
                                        )
                                        .unwrap(),
                                    });

                                    variable_scope.insert(
                                        var_name.clone(),
                                        (variable_type, unique_variable_id),
                                    );
                                }
                            },
                            CustomItem::Trait { name, .. } => {
                                return Err(ParserError::TraitNotObject(name.clone()).into());
                            },
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

                    if this_function_signature.return_type.clone() == Type::Void {
                        if *next_token != Token::SemiColon
                            && this_function_signature.visibility != FunctionVisibility::Branch
                        {
                            return Err(ParserError::SyntaxError(
                                SyntaxError::InvalidVariableDefinition,
                            )
                            .into());
                        }
                    }
                    else {
                        let (returned_value, jmp_idx, _) = parse_value(
                            &tokens[token_idx..],
                            function_token_offset,
                            &self.tokens_debug_info,
                            origin_token_idx,
                            function_signatures.clone(),
                            &mut variable_scope,
                            Some(this_function_signature.return_type.clone()),
                            function_imports.clone(),
                            custom_items.clone(),
                        )?;

                        token_idx += jmp_idx;

                        parsed_token_instances.push(ParsedTokenInstance {
                            inner: ParsedToken::ReturnValue(Box::new(returned_value)),
                            debug_information: fetch_and_merge_debug_information(
                                &self.tokens_debug_info,
                                origin_token_idx + function_token_offset
                                    ..token_idx + function_token_offset + 1,
                                true,
                            )
                            .unwrap(),
                        });
                    }
                }
                else if Token::If == current_token {
                    token_idx += 1;

                    if let Token::OpenParentheses = tokens[token_idx] {
                        token_idx += 1;
                        let paren_close_idx =
                            find_closing_paren(&tokens[token_idx..], 0)? + token_idx;

                        // This is what we have to evaulate in order to execute the appropriate branch of the if statement
                        let cond_slice = &tokens[token_idx..paren_close_idx];

                        let (condition, _cond_slice_len, _) = parse_value(
                            cond_slice,
                            function_token_offset,
                            &self.tokens_debug_info,
                            token_idx,
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

                            let true_condition_block = self.parse_function_block(
                                true_block_slice,
                                token_idx + function_token_offset,
                                function_signatures.clone(),
                                FunctionSignature {
                                    name: String::new(),
                                    args: FunctionArguments::new(),
                                    return_type: Type::Void,
                                    module_path: module_path.clone(),
                                    visibility: FunctionVisibility::Branch,
                                    compiler_hints: OrdSet::new(),
                                    enabling_features: OrdSet::new(),
                                },
                                function_imports.clone(),
                                custom_items.clone(),
                                this_fn_args.clone(),
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

                                    let false_block_slice =
                                        tokens[token_idx..paren_close_idx].to_vec();

                                    else_condition_branch = self.parse_function_block(
                                        false_block_slice,
                                        function_token_offset + token_idx,
                                        function_signatures.clone(),
                                        FunctionSignature {
                                            name: String::new(),
                                            args: FunctionArguments::new(),
                                            return_type: Type::Void,
                                            visibility: FunctionVisibility::Branch,
                                            module_path: module_path.clone(),
                                            compiler_hints: OrdSet::new(),
                                            enabling_features: OrdSet::new(),
                                        },
                                        function_imports.clone(),
                                        custom_items.clone(),
                                        this_fn_args.clone(),
                                        variable_scope.clone(),
                                    )?;

                                    token_idx = paren_close_idx + 1;
                                }
                            }

                            parsed_token_instances.push(ParsedTokenInstance {
                                inner: ParsedToken::If(If {
                                    condition: Box::new(condition),
                                    true_branch: true_condition_block,
                                    false_branch: else_condition_branch,
                                }),
                                debug_information: fetch_and_merge_debug_information(
                                    &self.tokens_debug_info,
                                    origin_token_idx + function_token_offset
                                        ..token_idx + function_token_offset,
                                    true,
                                )
                                .unwrap(),
                            });

                            continue;
                        }
                    }

                    return Err(ParserError::SyntaxError(
                        SyntaxError::InvalidIfConditionDefinition,
                    )
                    .into());
                }
                else if Token::Loop == current_token {
                    token_idx += 1;

                    if let Token::OpenBraces = tokens[token_idx] {
                        token_idx += 1;

                        let paren_close_idx =
                            find_closing_braces(&tokens[token_idx..], 0)? + token_idx;

                        // This is what we have to evaulate in order to execute the appropriate branch of the if statement
                        let loop_body_tokens = &tokens[token_idx..paren_close_idx];

                        let loop_body = self.parse_function_block(
                            loop_body_tokens.to_vec(),
                            function_token_offset + token_idx,
                            function_signatures.clone(),
                            FunctionSignature {
                                name: String::new(),
                                args: FunctionArguments::new(),
                                return_type: Type::Void,
                                visibility: FunctionVisibility::Branch,
                                module_path: module_path.clone(),
                                compiler_hints: OrdSet::new(),
                                enabling_features: OrdSet::new(),
                            },
                            function_imports.clone(),
                            custom_items.clone(),
                            this_fn_args.clone(),
                            variable_scope.clone(),
                        )?;

                        token_idx = paren_close_idx + 1;

                        parsed_token_instances.push(ParsedTokenInstance {
                            inner: ParsedToken::Loop(loop_body),
                            debug_information: fetch_and_merge_debug_information(
                                &self.tokens_debug_info,
                                origin_token_idx + function_token_offset
                                    ..token_idx + function_token_offset,
                                true,
                            )
                            .unwrap(),
                        });

                        continue;
                    }

                    return Err(ParserError::SyntaxError(SyntaxError::InvalidLoopBody).into());
                }
                else if Token::Continue == current_token {
                    token_idx += 1;

                    parsed_token_instances.push(ParsedTokenInstance {
                        inner: ParsedToken::ControlFlow(ControlFlowType::Continue),
                        debug_information: fetch_and_merge_debug_information(
                            &self.tokens_debug_info,
                            origin_token_idx + function_token_offset
                                ..token_idx + function_token_offset,
                            true,
                        )
                        .unwrap(),
                    });
                }
                else if Token::Break == current_token {
                    token_idx += 1;

                    parsed_token_instances.push(ParsedTokenInstance {
                        inner: ParsedToken::ControlFlow(ControlFlowType::Break),
                        debug_information: fetch_and_merge_debug_information(
                            &self.tokens_debug_info,
                            origin_token_idx + function_token_offset
                                ..token_idx + function_token_offset,
                            true,
                        )
                        .unwrap(),
                    });
                }

                token_idx += 1;
            }
        }

        // If there isnt a returned value and the returned type isnt `Void` raise an error
        if !has_return && this_function_signature.return_type != Type::Void {
            return Err(ParserError::SyntaxError(SyntaxError::FunctionRequiresReturn).into());
        }

        Ok(parsed_token_instances)
    }
}

pub fn parse_function_signature(
    tokens: &[Token],
    token_idx: &mut usize,
    custom_types: &IndexMap<String, CustomItem>,
    module_path: Vec<String>,
    function_name: String,
    is_struct_implementation: bool,
) -> anyhow::Result<FunctionSignature>
{
    let (bracket_close_idx, args) = parse_signature_argument_tokens(
        &tokens[*token_idx..],
        custom_types,
        is_struct_implementation,
    )?;

    *token_idx += bracket_close_idx;

    if tokens[*token_idx + 1] == Token::Colon {
        // Check for SemiColon for shits and giggles
        if tokens[*token_idx + 3] != Token::SemiColon {
            return Err(ParserError::SyntaxError(SyntaxError::MissingSemiColon).into());
        }

        // Get return type for function
        let return_ty = ty_from_token(&tokens[*token_idx + 2], custom_types)?;

        // Increment idx
        *token_idx += 3;

        Ok(FunctionSignature {
            name: function_name,
            args,
            return_type: return_ty,
            module_path,
            // Imported functions can only be accessed at the source file they were imported at
            // I might change this later to smth like pub import similar to pub mod in rust
            visibility: FunctionVisibility::Private,
            compiler_hints: OrdSet::new(),
            enabling_features: OrdSet::new(),
        })
    }
    else {
        Err(SyntaxError::FunctionSignatureReturnTypeRequired.into())
    }
}
