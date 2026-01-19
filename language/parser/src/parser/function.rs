use std::{
    collections::{HashMap, HashSet},
    fs, mem,
    path::PathBuf,
    rc::Rc,
};

use common::{
    anyhow::{self, Result},
    codegen::{CustomType, FunctionArgumentIdentifier, If, StructAttributes},
    compiler::ProjectConfig,
    dashmap::DashMap,
    error::{DbgInfo, parser::ParserError, syntax::SyntaxError},
    indexmap::IndexMap,
    parser::{
        common::{
            ParsedToken, ParsedTokenInstance, find_closing_braces, find_closing_comma,
            find_closing_paren,
        },
        dbg::fetch_and_merge_debug_information,
        function::{
            self, CompilerHint, FunctionArguments, FunctionDefinition, FunctionSignature,
            FunctionVisibility, UnparsedFunctionDefinition,
        },
        import::parse_import_path,
        variable::{ControlFlowType, UniqueId, VARIABLE_ID_SOURCE, VariableReference},
    },
    tokenizer::Token,
    tracing::info,
    ty::{OrdMap, OrdSet, Type, Value, ty_from_token},
};

use crate::{
    parser::{value::parse_value, variable::resolve_variable_expression},
    parser_instance::Parser,
    tokenizer::tokenize,
};

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
        IndexMap<String, CustomType>,
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

        let mut custom_types: IndexMap<String, CustomType> = IndexMap::new();

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
                                &custom_types,
                            )?;

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
                                    if let Some(custom_type) = custom_types.get(&identifier) {
                                        match custom_type {
                                            CustomType::Struct(struct_def) => {
                                                Type::Struct(struct_def.clone())
                                            },
                                            CustomType::Enum((ty, enum_def)) => {
                                                Type::Enum((Box::new(ty.clone()), enum_def.clone()))
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
                                                function_sig: FunctionSignature {
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
                                                    overwritten_function.function_sig,
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
                    let (bracket_close_idx, args) =
                        parse_signature_argument_tokens(&tokens[token_idx + 3..], &custom_types)?;

                    token_idx += bracket_close_idx + 3;

                    if external_imports.get(&identifier).is_some()
                        || function_list.get(&identifier).is_some()
                    {
                        return Err(ParserError::DuplicateSignatureImports(identifier).into());
                    }

                    // Create a clone of the module path so we can modifiy it locally
                    let mut mod_path = module_path.clone();

                    // Store the function name in the module path
                    mod_path.push(identifier.clone());

                    if tokens[token_idx + 1] == Token::Colon {
                        if let Token::TypeDefinition(return_type) = tokens[token_idx + 2].clone()
                            && tokens[token_idx + 3] == Token::SemiColon
                        {
                            external_imports.insert(
                                identifier.clone(),
                                FunctionSignature {
                                    name: identifier,
                                    args,
                                    return_type,
                                    module_path: mod_path,
                                    // Imported functions can only be accessed at the source file they were imported at
                                    // I might change this later to smth like pub import similar to pub mod in rust
                                    visibility: FunctionVisibility::Private,
                                    compiler_hints: OrdSet::new(),
                                    enabling_features: OrdSet::new(),
                                },
                            );

                            continue;
                        }
                        else if let Token::Identifier(custom_item_name) =
                            tokens[token_idx + 2].clone()
                            && tokens[token_idx + 3] == Token::SemiColon
                        {
                            if let Some(custom_item) = custom_types.get(&custom_item_name) {
                                match custom_item {
                                    CustomType::Struct(struct_inner) => {
                                        external_imports.insert(
                                            identifier.clone(),
                                            FunctionSignature {
                                                name: identifier,
                                                args,
                                                return_type: Type::Struct(struct_inner.clone()),
                                                module_path: mod_path,
                                                // Imported functions can only be accessed at the source file they were imported at
                                                // I might change this later to smth like pub import similar to pub use in rust
                                                visibility: FunctionVisibility::Private,
                                                compiler_hints: OrdSet::new(),
                                                enabling_features: OrdSet::new(),
                                            },
                                        );

                                        continue;
                                    },
                                    CustomType::Enum((ty, body)) => {
                                        external_imports.insert(
                                            identifier.clone(),
                                            FunctionSignature {
                                                name: identifier,
                                                args,
                                                return_type: Type::Enum((
                                                    Box::new(ty.clone()),
                                                    body.clone(),
                                                )),
                                                module_path: mod_path,
                                                // Imported functions can only be accessed at the source file they were imported at
                                                // I might change this later to smth like pub import similar to pub use in rust
                                                visibility: FunctionVisibility::Private,
                                                compiler_hints: OrdSet::new(),
                                                enabling_features: OrdSet::new(),
                                            },
                                        );

                                        continue;
                                    },
                                }
                            }
                            else {
                                dbg!(&custom_types);

                                panic!(
                                    "Custom type not found, check custom types map...... Monkey see Monkey think"
                                )
                            }
                        }
                        else {
                            panic!();
                        }
                    }
                    else {
                        return Err(SyntaxError::ImportUnspecifiedReturnType.into());
                    }
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
                    // SeRch for the closing brace's index
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
                                        CustomType::Struct(struct_def) => {
                                            struct_fields.insert(
                                                field_name.to_string(),
                                                Type::Struct(struct_def.clone()),
                                            );
                                        },
                                        CustomType::Enum((ty, enum_body)) => {
                                            struct_fields.insert(
                                                field_name.to_string(),
                                                Type::Enum((
                                                    Box::new(ty.clone()),
                                                    enum_body.clone(),
                                                )),
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
                        CustomType::Struct((struct_name.clone(), struct_fields.into(), StructAttributes::default())),
                    );

                    token_idx = braces_idx + 1;
                    continue;
                }

                return Err(ParserError::SyntaxError(SyntaxError::InvalidStructDefinition).into());
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
                        CustomType::Enum((variant_type, variant_fields)),
                    );

                    token_idx = braces_idx + 1;

                    continue;
                }

                return Err(
                    ParserError::SyntaxError(SyntaxError::CustomTypeRequiresName(current_token))
                        .into(),
                );
            }

            token_idx += 1;
        }

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
        custom_items: Rc<IndexMap<String, CustomType>>,
    ) -> Result<IndexMap<String, FunctionDefinition>>
    {
        let config = self.config.clone();
        let module_path = self.module_path.clone();

        let mut parsed_functions: IndexMap<String, FunctionDefinition> = IndexMap::new();

        for (fn_idx, (fn_name, unparsed_function)) in unparsed_functions.clone().iter().enumerate()
        {
            let function_definition = FunctionDefinition {
                signature: unparsed_function.function_sig.clone(),
                inner: self.parse_function_block(
                    unparsed_function.inner.clone(),
                    unparsed_function.token_offset,
                    unparsed_functions.clone(),
                    unparsed_function.function_sig.clone(),
                    function_imports.clone(),
                    custom_items.clone(),
                    unparsed_function.function_sig.args.clone(),
                    OrdMap::new(),
                )?,
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
        custom_items: Rc<IndexMap<String, CustomType>>,
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
                            function_sig.function_sig.args.clone(),
                            function_signatures.clone(),
                            function_imports.clone(),
                            custom_items.clone(),
                        )?;

                        token_idx += jumped_idx + 2;

                        parsed_token_instances.push(ParsedTokenInstance {
                            inner: ParsedToken::FunctionCall(
                                (function_sig.function_sig.clone(), ident_name.clone()),
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
                            CustomType::Struct(struct_instance) => {
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
                            CustomType::Enum(inner) => {
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

/// The slice should startwith the first token from inside the Parentheses.
/// This function quits at the ")". (Excluding function calls)
pub fn parse_function_call_args(
    tokens: &[Token],
    function_tokens_offset: usize,
    mut origin_token_idx: usize,
    debug_infos: &[DbgInfo],
    variable_scope: &mut IndexMap<String, (Type, UniqueId)>,
    mut this_function_args: FunctionArguments,
    function_signatures: Rc<IndexMap<String, UnparsedFunctionDefinition>>,
    standard_function_table: Rc<HashMap<String, FunctionSignature>>,
    custom_items: Rc<IndexMap<String, CustomType>>,
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
                    standard_function_table.clone(),
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
                        standard_function_table.clone(),
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
                        standard_function_table.clone(),
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
                    args.arguments.insert(
                        var_name,
                        (var_type.clone(), VARIABLE_ID_SOURCE.get_unique_id()),
                    );

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
                    args.arguments.insert(
                        var_name,
                        (custom_ty.clone(), VARIABLE_ID_SOURCE.get_unique_id()),
                    );

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
