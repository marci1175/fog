use crate::{
    codegen::DerefMode,
    error::parser::ParserError,
    parser::{function::parse_function_call_args, variable::resolve_variable_expression},
    tokenizer::Token,
};
use anyhow::Result;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MathematicalSymbol
{
    Addition,
    Subtraction,
    Division,
    Multiplication,
    Modulo,
}

impl TryInto<MathematicalSymbol> for Token
{
    type Error = ParserError;

    fn try_into(self) -> Result<MathematicalSymbol, Self::Error>
    {
        let expr = match self {
            Self::Addition => MathematicalSymbol::Addition,
            Self::Subtraction => MathematicalSymbol::Subtraction,
            Self::Division => MathematicalSymbol::Division,
            Self::Multiplication => MathematicalSymbol::Multiplication,
            Self::Modulo => MathematicalSymbol::Modulo,

            _ => return Err(ParserError::InternalVariableError),
        };

        Ok(expr)
    }
}

use std::{collections::HashMap, rc::Rc};

use crate::{codegen::StructAttributes, indexmap::IndexMap};

use crate::{
    codegen::{CustomItem, Order},
    error::{DbgInfo, syntax::SyntaxError},
    parser::{
        common::{ParsedToken, ParsedTokenInstance, find_closing_braces, find_closing_paren},
        dbg::fetch_and_merge_debug_information,
        function::{FunctionSignature, UnparsedFunctionDefinition},
        variable::{UniqueId, VariableReference},
    },
    ty::{Type, Value, ty_from_token, unparsed_const_to_typed_literal_unsafe},
};

/// This is a top level implementation for `parse_token_as_value`
pub fn parse_value(
    tokens: &[Token],
    function_tokens_offset: usize,
    debug_infos: &[DbgInfo],
    origin_token_idx: usize,
    function_signatures: Rc<IndexMap<String, UnparsedFunctionDefinition>>,
    variable_scope: &mut IndexMap<String, (Type, UniqueId)>,
    // Always pass in the desired variable type, you can only leave this `None` if you dont know the type by design
    mut desired_variable_type: Option<Type>,
    function_imports: Rc<HashMap<String, FunctionSignature>>,
    custom_types: Rc<IndexMap<String, CustomItem>>,
) -> Result<(ParsedTokenInstance, usize, Type)>
{
    let mut token_idx = 0;

    // This is used for parsing mathematical expressions, comparisons
    let mut parsed_token: Option<ParsedTokenInstance> = None;
    let mut comparison_other_side_ty: Option<Type> = None;

    while token_idx < tokens.len() {
        let current_token = &tokens.get(token_idx).ok_or({
            ParserError::SyntaxError(SyntaxError::InvalidMathematicalExpressionDefinition)
        })?;

        if let Some(next_token) = tokens.get(token_idx + 1)
            && (*next_token == Token::Equal
                || *next_token == Token::NotEqual
                || *next_token == Token::EqBigger
                || *next_token == Token::EqSmaller
                || *next_token == Token::Bigger
                || *next_token == Token::Smaller)
        {
            // I do this so that the parser wont check the type of the value it parses to avoid returning an error here
            desired_variable_type = None;
        }

        // Please note that we are not looking at values by themselves, except in SetValue where we take the next token.
        match current_token {
            // If any mathematical expression is present in the tokens
            Token::Addition | Token::Subtraction | Token::Multiplication | Token::Division => {
                // Grab the next token after the mathematical expression
                let next_token = &tokens.get(token_idx + 1).ok_or(ParserError::SyntaxError(
                    SyntaxError::InvalidMathematicalExpressionDefinition,
                ))?;

                // If we have parsed something already move it to the left-hand side of the mathematical expression
                // Add the new parsed token to the right-hand side of the mathematical expression.
                if let Some(parsed_token) = &mut parsed_token {
                    token_idx += 1;

                    // Modify the parsed token
                    *parsed_token = ParsedTokenInstance {
                        inner: ParsedToken::MathematicalExpression(
                            // Move the token to the left side
                            Box::new(parsed_token.clone()),
                            // Add the Mathematical symbol to the enum variant
                            (*current_token).clone().try_into()?,
                            // Put the new item to the right side of the expr.
                            Box::new(
                                parse_token_as_value(
                                    tokens,
                                    function_tokens_offset,
                                    debug_infos,
                                    origin_token_idx,
                                    function_signatures.clone(),
                                    variable_scope,
                                    desired_variable_type.clone(),
                                    &mut token_idx,
                                    next_token,
                                    function_imports.clone(),
                                    custom_types.clone(),
                                )?
                                .0,
                            ),
                        ),
                        debug_information: fetch_and_merge_debug_information(
                            debug_infos,
                            origin_token_idx + function_tokens_offset
                                ..origin_token_idx + token_idx + function_tokens_offset,
                            true,
                        )
                        .unwrap(),
                    };
                }
                else {
                    return Err(ParserError::SyntaxError(
                        SyntaxError::InvalidMathematicalExpressionDefinition,
                    )
                    .into());
                }
            },

            // This pattern match is purely for initializing the value of the variable.
            // The ParsedToken generated by the pattern match will not be evaluated in future iterations.
            Token::UnparsedLiteral(_) => {
                let (parsed_value, ty) = parse_token_as_value(
                    tokens,
                    function_tokens_offset,
                    debug_infos,
                    origin_token_idx,
                    function_signatures.clone(),
                    variable_scope,
                    desired_variable_type.clone(),
                    &mut token_idx,
                    current_token,
                    function_imports.clone(),
                    custom_types.clone(),
                )?;

                // Initialize parsed token with a value.
                if parsed_token.is_none() {
                    parsed_token = Some(parsed_value);

                    comparison_other_side_ty = Some(ty);
                }
            },

            // This pattern match is purely for initializing the value of the variable.
            // The ParsedToken generated by the pattern match will not be evaluated in future iterations.
            Token::Identifier(_) | Token::OpenParentheses => {
                let (parsed_value, ty) = parse_token_as_value(
                    tokens,
                    function_tokens_offset,
                    debug_infos,
                    origin_token_idx,
                    function_signatures.clone(),
                    variable_scope,
                    desired_variable_type.clone(),
                    &mut token_idx,
                    current_token,
                    function_imports.clone(),
                    custom_types.clone(),
                )?;

                // Initialize parsed token with a value.
                if parsed_token.is_none() {
                    parsed_token = Some(parsed_value);
                    comparison_other_side_ty = Some(ty);
                }
            },

            Token::Reference => {
                let (parsed_value, ty) = parse_token_as_value(
                    tokens,
                    function_tokens_offset,
                    debug_infos,
                    origin_token_idx,
                    function_signatures.clone(),
                    variable_scope,
                    desired_variable_type.clone(),
                    &mut token_idx,
                    current_token,
                    function_imports.clone(),
                    custom_types.clone(),
                )?;

                // Initialize parsed token with a value.
                if parsed_token.is_none() {
                    parsed_token = Some(parsed_value);

                    comparison_other_side_ty = Some(ty);
                }
            },

            Token::Dereference => {
                let (parsed_value, ty) = parse_token_as_value(
                    tokens,
                    function_tokens_offset,
                    debug_infos,
                    origin_token_idx,
                    function_signatures.clone(),
                    variable_scope,
                    desired_variable_type.clone(),
                    &mut token_idx,
                    current_token,
                    function_imports.clone(),
                    custom_types.clone(),
                )?;

                // Initialize parsed token with a value.
                if parsed_token.is_none() {
                    parsed_token = Some(parsed_value);

                    comparison_other_side_ty = Some(ty);
                }
            },

            Token::Literal(literal) => {
                let (parsed_value, ty) = parse_token_as_value(
                    tokens,
                    function_tokens_offset,
                    debug_infos,
                    origin_token_idx,
                    function_signatures.clone(),
                    variable_scope,
                    Some(literal.get_type()),
                    &mut token_idx,
                    current_token,
                    function_imports.clone(),
                    custom_types.clone(),
                )?;

                // Initialize parsed token with a value.
                if parsed_token.is_none() {
                    parsed_token = Some(parsed_value);

                    comparison_other_side_ty = Some(ty);
                }
            },

            Token::Comma | Token::CloseParentheses | Token::SemiColon => {
                break;
            },

            Token::Equal
            | Token::NotEqual
            | Token::EqBigger
            | Token::EqSmaller
            | Token::Bigger
            | Token::Smaller => {
                if let Some(last_p_token) = &parsed_token {
                    token_idx += 1;

                    let next_token = &tokens[token_idx];

                    let (current_cmp_token, token_ty) = parse_token_as_value(
                        tokens,
                        function_tokens_offset,
                        debug_infos,
                        origin_token_idx,
                        function_signatures.clone(),
                        variable_scope,
                        comparison_other_side_ty.clone(),
                        &mut token_idx,
                        next_token,
                        function_imports.clone(),
                        custom_types.clone(),
                    )?;

                    parsed_token = Some(ParsedTokenInstance {
                        inner: ParsedToken::Comparison(
                            Box::new(last_p_token.clone()),
                            Order::from_token(current_token)?,
                            Box::new(current_cmp_token),
                            token_ty,
                        ),
                        debug_information: fetch_and_merge_debug_information(
                            debug_infos,
                            origin_token_idx + function_tokens_offset
                                ..origin_token_idx + token_idx + function_tokens_offset,
                            true,
                        )
                        .unwrap(),
                    });
                }
            },

            Token::OpenBraces => {
                let (parsed_value, ty) = parse_token_as_value(
                    tokens,
                    function_tokens_offset,
                    debug_infos,
                    origin_token_idx,
                    function_signatures.clone(),
                    variable_scope,
                    desired_variable_type.clone(),
                    &mut token_idx,
                    current_token,
                    function_imports.clone(),
                    custom_types.clone(),
                )?;

                // Initialize parsed token with a value.
                if parsed_token.is_none() {
                    parsed_token = Some(parsed_value);
                    comparison_other_side_ty = Some(ty);
                }
            },

            Token::As => {
                if let Some(last_token) = &parsed_token {
                    if let Some(token) = tokens.get(token_idx + 1) {
                        let target_type = ty_from_token(token, &custom_types)?;

                        token_idx += 2;

                        parsed_token = Some(ParsedTokenInstance {
                            inner: ParsedToken::TypeCast(
                                Box::new(last_token.clone()),
                                target_type.clone(),
                            ),
                            debug_information: fetch_and_merge_debug_information(
                                debug_infos,
                                origin_token_idx + function_tokens_offset
                                    ..origin_token_idx + token_idx + function_tokens_offset,
                                true,
                            )
                            .unwrap(),
                        });
                        comparison_other_side_ty = Some(target_type.clone());
                    }
                    else {
                        // Throw an error
                        return Err(ParserError::SyntaxError(SyntaxError::AsRequiresTypeDef).into());
                    }
                }
            },

            Token::This => {
                let (parsed_value, ty) = parse_token_as_value(
                    tokens,
                    function_tokens_offset,
                    debug_infos,
                    origin_token_idx,
                    function_signatures.clone(),
                    variable_scope,
                    desired_variable_type.clone(),
                    &mut token_idx,
                    &Token::This,
                    function_imports.clone(),
                    custom_types.clone(),
                )?;

                // Initialize parsed token with a value.
                if parsed_token.is_none() {
                    parsed_token = Some(parsed_value);
                    comparison_other_side_ty = Some(ty);
                }
            },

            _ => {
                // dbg!(parsed_token);
                dbg!(current_token);

                unimplemented!()
            },
        }
    }

    Ok((
        parsed_token.ok_or(ParserError::SyntaxError(
            SyntaxError::InvalidVariableDefinition,
        ))?,
        token_idx,
        comparison_other_side_ty.ok_or(ParserError::SyntaxError(
            SyntaxError::InvalidVariableDefinition,
        ))?,
    ))
}

/// Parses the next token as something that holds a value:
/// Like: FunctionCall, Literal, UnparsedLiteral
pub fn parse_token_as_value(
    // This is used to parse the function call's arguments
    tokens: &[Token],
    function_token_offset: usize,
    debug_infos: &[DbgInfo],
    origin_token_idx: usize,
    // Functions available
    function_signatures: Rc<IndexMap<String, UnparsedFunctionDefinition>>,
    // Variables available
    variable_scope: &mut IndexMap<String, (Type, UniqueId)>,
    // The variable's type which we are parsing for
    desired_variable_type: Option<Type>,
    // Universal token_idx, this sets which token we are currently parsing
    token_idx: &mut usize,
    // The token we want to evaluate, this is the first token of the slice most of the time
    eval_token: &Token,
    function_imports: Rc<HashMap<String, FunctionSignature>>,
    custom_types: Rc<IndexMap<String, CustomItem>>,
) -> Result<(ParsedTokenInstance, Type)>
{
    // Match the token
    let (inner_parsed_token, inner_parsed_token_ty) = match eval_token {
        Token::Literal(literal) => {
            // Increment the token_idx by the tokens we have analyzed
            *token_idx += 1;

            // Check if there is an `As` keyword after the variable
            if let Some(Token::As) = tokens.get(*token_idx) {
                // If there isnt a TypeDefinition after the `As` keyword raise an error
                if let Some(Token::TypeDefinition(target_type)) = tokens.get(*token_idx + 1) {
                    // Increment the token index after checking target type
                    *token_idx += 2;

                    // Return the type casted literal
                    (
                        ParsedToken::TypeCast(
                            Box::new(ParsedTokenInstance {
                                inner: ParsedToken::Literal(literal.clone()),
                                debug_information: fetch_and_merge_debug_information(
                                    debug_infos,
                                    origin_token_idx + function_token_offset
                                        ..origin_token_idx + *token_idx + function_token_offset,
                                    true,
                                )
                                .unwrap(),
                            }),
                            target_type.clone(),
                        ),
                        target_type.clone(),
                    )
                }
                else {
                    // Throw an error
                    return Err(ParserError::SyntaxError(SyntaxError::AsRequiresTypeDef).into());
                }
            }
            else {
                // Push the ParsedToken to the list
                (ParsedToken::Literal(literal.clone()), literal.get_type())
            }
        },
        Token::UnparsedLiteral(unparsed_literal) => {
            // Increment the token_idx by the tokens we have analyzed
            *token_idx += 1;

            // Push the ParsedToken to the list
            let parsed_value = unparsed_const_to_typed_literal_unsafe(
                unparsed_literal,
                if Some(&Token::As) == tokens.get(*token_idx) {
                    None
                }
                else {
                    desired_variable_type
                },
            )?;

            let parsed_token = ParsedToken::Literal(parsed_value.clone());

            // Check if there is an `As` keyword after the variable
            if let Some(&Token::As) = tokens.get(*token_idx) {
                // If there isnt a TypeDefinition after the `As` keyword raise an error
                if let Some(Token::TypeDefinition(target_type)) = tokens.get(*token_idx + 1) {
                    // Increment the token index after checking target type
                    *token_idx += 2;

                    // Return the type casted literal
                    (
                        ParsedToken::TypeCast(
                            Box::new(ParsedTokenInstance {
                                inner: parsed_token,
                                debug_information: fetch_and_merge_debug_information(
                                    debug_infos,
                                    origin_token_idx + function_token_offset
                                        ..origin_token_idx + *token_idx + function_token_offset,
                                    true,
                                )
                                .unwrap(),
                            }),
                            target_type.clone(),
                        ),
                        target_type.clone(),
                    )
                }
                else {
                    // Throw an error
                    return Err(ParserError::SyntaxError(SyntaxError::AsRequiresTypeDef).into());
                }
            }
            else {
                (parsed_token, parsed_value.get_type())
            }
        },
        Token::Identifier(identifier) => {
            // Try to find the identifier in the functions' list
            if let Some(function) = function_signatures.get(identifier) {
                // Parse the call arguments and tokens parsed.
                let (call_arguments, idx_jmp) = parse_function_call_args(
                    &tokens[*token_idx + 2..],
                    function_token_offset,
                    *token_idx + 2,
                    debug_infos,
                    variable_scope,
                    function.signature.args.clone(),
                    function_signatures.clone(),
                    function_imports.clone(),
                    custom_types.clone(),
                    None,
                )?;

                // Return the function call
                let parsed_token: ParsedToken = ParsedToken::FunctionCall(
                    (function.signature.clone(), identifier.clone()),
                    call_arguments,
                );

                // Increment the token index, and add the offset
                *token_idx += idx_jmp + 2 + 1;

                if let Some(Token::As) = tokens.get(*token_idx) {
                    if let Some(Token::TypeDefinition(target_type)) = tokens.get(*token_idx + 1) {
                        *token_idx += 2;

                        (
                            ParsedToken::TypeCast(
                                Box::new(ParsedTokenInstance {
                                    inner: parsed_token,
                                    debug_information: fetch_and_merge_debug_information(
                                        debug_infos,
                                        origin_token_idx + function_token_offset
                                            ..origin_token_idx + *token_idx + function_token_offset,
                                        true,
                                    )
                                    .unwrap(),
                                }),
                                target_type.clone(),
                            ),
                            target_type.clone(),
                        )
                    }
                    else {
                        // Throw an error
                        return Err(ParserError::SyntaxError(SyntaxError::AsRequiresTypeDef).into());
                    }
                }
                else {
                    // If there is a desired variable type then check if the two types match
                    if let Some(desired_variable_type) = desired_variable_type {
                        // If the function's return type doesn't match the variable's return type return an error
                        if function.signature.return_type != desired_variable_type {
                            return Err(ParserError::TypeMismatch(
                                function.signature.return_type.clone(),
                                desired_variable_type,
                            )
                            .into());
                        }

                        (parsed_token, desired_variable_type)
                    }
                    // If there were no explicit type definitions, return the type which is produced by the function
                    else {
                        (parsed_token, function.signature.return_type.clone())
                    }
                }
            }
            // If the identifier could not be found in the function list search in the variable scope
            else if let Some((variable_type, variable_id)) =
                variable_scope.get(identifier).cloned()
            {
                let mut basic_reference = ParsedTokenInstance {
                    inner: ParsedToken::VariableReference(VariableReference::BasicReference(
                        identifier.clone(),
                        variable_id,
                    )),
                    debug_information: fetch_and_merge_debug_information(
                        debug_infos,
                        *token_idx..*token_idx + 1,
                        true,
                    )
                    .unwrap(),
                };

                *token_idx += 1;

                let var_ty = resolve_variable_expression(
                    tokens,
                    function_token_offset,
                    debug_infos,
                    token_idx,
                    function_signatures,
                    function_imports,
                    variable_scope,
                    (variable_type, variable_id),
                    custom_types,
                    &mut basic_reference,
                    &mut Vec::new(),
                    identifier,
                )?;

                // Return the VariableReference
                (basic_reference.inner, var_ty)
            }
            else if let Some(function_sig) = function_imports.get(identifier) {
                // Parse the call arguments and tokens parsed.
                let (call_arguments, idx_jmp) = parse_function_call_args(
                    &tokens[*token_idx + 2..],
                    function_token_offset,
                    *token_idx + 2,
                    debug_infos,
                    variable_scope,
                    function_sig.args.clone(),
                    function_signatures.clone(),
                    function_imports.clone(),
                    custom_types.clone(),
                    None,
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

                        (
                            ParsedToken::TypeCast(
                                Box::new(ParsedTokenInstance {
                                    inner: parsed_token,
                                    debug_information: fetch_and_merge_debug_information(
                                        debug_infos,
                                        origin_token_idx + function_token_offset
                                            ..origin_token_idx + *token_idx + function_token_offset,
                                        true,
                                    )
                                    .unwrap(),
                                }),
                                target_type.clone(),
                            ),
                            target_type.clone(),
                        )
                    }
                    else {
                        // Throw an error
                        return Err(ParserError::SyntaxError(SyntaxError::AsRequiresTypeDef).into());
                    }
                }
                else {
                    // If there is a desired variable type then check if the two types match
                    if let Some(desired_variable_type) = desired_variable_type {
                        // If the function's return type doesn't match the variable's return type return an error
                        if function_sig.return_type != desired_variable_type {
                            return Err(ParserError::TypeMismatch(
                                function_sig.return_type.clone(),
                                desired_variable_type,
                            )
                            .into());
                        }

                        (parsed_token, desired_variable_type)
                    }
                    // If there were no explicit type definitions, return the type which is produced by the function
                    else {
                        (parsed_token, function_sig.return_type.clone())
                    }
                }
            }
            else if let Some(custom_type) = custom_types.get(identifier) {
                match custom_type {
                    CustomItem::Struct((struct_name, struct_inner, attr)) => {
                        if let Some(Token::OpenBraces) = tokens.get(*token_idx + 1) {
                            let closing_idx = find_closing_braces(&tokens[*token_idx + 2..], 0)?;

                            let struct_init_slice =
                                &tokens[*token_idx + 2..*token_idx + 2 + closing_idx];

                            let (_jump_idx, init_struct_token) = init_struct(
                                struct_init_slice,
                                function_token_offset,
                                debug_infos,
                                origin_token_idx + *token_idx,
                                struct_inner,
                                struct_name.clone(),
                                function_signatures.clone(),
                                function_imports,
                                custom_types.clone(),
                                variable_scope,
                                attr.clone(),
                            )?;

                            // Increment the index to the token after the struct init.
                            *token_idx = *token_idx + 2 + closing_idx + 1;

                            return Ok((
                                init_struct_token,
                                Type::Struct((
                                    struct_name.clone(),
                                    struct_inner.clone(),
                                    attr.clone(),
                                )),
                            ));
                        }

                        return Err(ParserError::SyntaxError(
                            SyntaxError::InvalidStructFieldDefinition,
                        )
                        .into());
                    },
                    CustomItem::Enum((ty, variants)) => {
                        if let Some(Token::DoubleColon) = tokens.get(*token_idx + 1)
                            && let Some(Token::Identifier(variant_name)) =
                                tokens.get(*token_idx + 2)
                        {
                            // Lookup enum variant with name
                            let variant = variants.get(variant_name);

                            match variant {
                                Some(parsed_token) => {
                                    *token_idx += 3;

                                    return Ok((
                                        ParsedTokenInstance {
                                            inner: ParsedToken::Literal(Value::Enum((
                                                ty.clone(),
                                                variants.clone(),
                                                variant_name.clone(),
                                            ))),
                                            debug_information: parsed_token.debug_information,
                                        },
                                        Type::Enum((Box::new(ty.clone()), variants.clone())),
                                    ));
                                },
                                // If the variant was not found we can raise an error
                                None => {
                                    return Err(ParserError::EnumVariantNotFound(
                                        variant_name.clone(),
                                    )
                                    .into());
                                },
                            }
                        }

                        return Err(ParserError::SyntaxError(
                            SyntaxError::InvalidEnumVariantDefinition,
                        )
                        .into());
                    },
                    CustomItem::Trait { .. } => {
                        return Err(ParserError::TraitNotObject(identifier.clone()).into());
                    },
                }
            }
            else {
                // If none of the above matches throw an error about the variable not being found
                return Err(ParserError::VariableNotFound(identifier.clone()).into());
            }
        },
        Token::OpenParentheses => {
            *token_idx += 1;

            let closing_idx = find_closing_paren(&tokens[*token_idx..], 0)?;

            // Get the tokens inside the block aka the "()"
            let tokens_inside_block = &tokens[*token_idx..*token_idx + closing_idx];

            let desired_variable_type =
                desired_variable_type.ok_or(ParserError::InternalDesiredTypeMissing)?;

            let (parsed_token, _jmp_idx, _) = parse_value(
                tokens_inside_block,
                function_token_offset + *token_idx,
                debug_infos,
                origin_token_idx,
                function_signatures.clone(),
                variable_scope,
                Some(desired_variable_type.clone()),
                function_imports,
                custom_types.clone(),
            )?;

            *token_idx += closing_idx + 1;

            (
                ParsedToken::MathematicalBlock(Box::new(parsed_token)),
                desired_variable_type.clone(),
            )
        },
        Token::OpenBraces => {
            *token_idx += 1;

            let closing_idx = find_closing_braces(&tokens[*token_idx..], 0)?;

            let tokens_inside_block = &tokens[*token_idx..*token_idx + closing_idx];

            let desired_variable_type =
                desired_variable_type.ok_or(ParserError::InternalDesiredTypeMissing)?;

            let mut array_item_idx = 0;

            let mut vec_values = Vec::new();

            // We will check for the valid length of the init value later, at codegen.
            if let Type::Array((inner_token, _len)) = &desired_variable_type {
                let inner_ty = ty_from_token(inner_token, &custom_types)?;

                while array_item_idx < tokens_inside_block.len() {
                    // Parse the value of the array
                    let (parsed_token, jump_index, _) = parse_value(
                        &tokens_inside_block[array_item_idx..],
                        function_token_offset + *token_idx,
                        debug_infos,
                        origin_token_idx,
                        function_signatures.clone(),
                        variable_scope,
                        Some(inner_ty.clone()),
                        function_imports.clone(),
                        custom_types.clone(),
                    )?;

                    // Store the parsed token
                    vec_values.push(parsed_token);

                    // Increment the idx counter
                    array_item_idx += jump_index + 1;
                }

                // Increment the token index to the end of the array's tokens.
                *token_idx += array_item_idx;

                // Return the final parsed token.
                return Ok((
                    ParsedTokenInstance {
                        inner: ParsedToken::ArrayInitialization(vec_values, inner_ty.clone()),
                        debug_information: fetch_and_merge_debug_information(
                            debug_infos,
                            origin_token_idx + function_token_offset
                                ..origin_token_idx + *token_idx + function_token_offset,
                            true,
                        )
                        .unwrap(),
                    },
                    desired_variable_type.clone(),
                ));
            }
            else {
                return Err(ParserError::TypeMismatchNonIndexable(desired_variable_type).into());
            }
        },
        Token::Reference => {
            *token_idx += 1;

            let (parsed_token, jmp_idx, val_ty) = parse_value(
                &tokens[1..],
                function_token_offset + *token_idx,
                debug_infos,
                origin_token_idx,
                function_signatures.clone(),
                variable_scope,
                desired_variable_type.and_then(|ty| {
                    match ty.clone().try_as_pointer() {
                        Some(inner) => {
                            inner.map(|inner_token| {
                                ty_from_token(&inner_token, &custom_types).unwrap()
                            })
                        },
                        None => Some(ty.clone()),
                    }
                }),
                function_imports,
                custom_types.clone(),
            )?;

            *token_idx += jmp_idx + 1;

            (
                ParsedToken::GetPointerTo(Box::new(parsed_token)),
                Type::Pointer(Some(Box::new(Token::TypeDefinition(val_ty)))),
            )
        },
        Token::Dereference => {
            *token_idx += 1;

            let (parsed_token, jmp_idx, _) = parse_value(
                &tokens[1..],
                function_token_offset + *token_idx,
                debug_infos,
                origin_token_idx,
                function_signatures.clone(),
                variable_scope,
                desired_variable_type.clone(),
                function_imports,
                custom_types.clone(),
            )?;

            *token_idx += jmp_idx + 1;

            (
                ParsedToken::DerefPointer {
                    inner_expr: Box::new(parsed_token),
                    mode: DerefMode::Value,
                },
                Type::I32,
            )
        },
        Token::This => {
            *token_idx += 1;

            let (variable_type, variable_id) = variable_scope
                .get("this")
                .ok_or(ParserError::VariableNotFound(String::from("this")))?
                .clone();

            let mut basic_reference = ParsedTokenInstance {
                inner: ParsedToken::VariableReference(VariableReference::BasicReference(
                    String::from("this"),
                    variable_id,
                )),
                debug_information: DbgInfo::default(),
            };

            let var_ty = resolve_variable_expression(
                tokens,
                function_token_offset,
                debug_infos,
                token_idx,
                function_signatures,
                function_imports,
                variable_scope,
                (variable_type, variable_id),
                custom_types,
                &mut basic_reference,
                &mut Vec::new(),
                "this",
            )?;

            // Return the VariableReference
            (basic_reference.inner, var_ty)
        },
        _ => {
            // If we are parsing something else than something that hold a value return an error.
            return Err(
                ParserError::SyntaxError(SyntaxError::InvalidValue(eval_token.clone())).into(),
            );
        },
    };

    Ok((
        (ParsedTokenInstance {
            inner: inner_parsed_token,
            debug_information: fetch_and_merge_debug_information(
                debug_infos,
                // Add the magic number and call it a day
                // DO we need +1?
                origin_token_idx + function_token_offset
                    ..origin_token_idx + *token_idx + function_token_offset + 1,
                true,
            )
            .unwrap(),
        }),
        inner_parsed_token_ty,
    ))
}

pub fn init_struct(
    struct_slice: &[Token],
    token_offset: usize,
    debug_infos: &[DbgInfo],
    origin_token_idx: usize,
    this_struct_field: &IndexMap<String, Type>,
    this_struct_name: String,
    function_signatures: Rc<IndexMap<String, UnparsedFunctionDefinition>>,
    function_imports: Rc<HashMap<String, FunctionSignature>>,
    custom_types: Rc<IndexMap<String, CustomItem>>,
    variable_scope: &mut IndexMap<String, (Type, UniqueId)>,
    struct_attributes: StructAttributes,
) -> Result<(usize, ParsedTokenInstance)>
{
    let mut struct_field_init_map: IndexMap<String, Box<ParsedTokenInstance>> = IndexMap::new();

    let mut idx: usize = 0;

    let mut nth_field: usize = 0;

    while idx < struct_slice.len() {
        if let Some(Token::Identifier(field_name)) = struct_slice.get(idx)
            && let Some(Token::Colon) = struct_slice.get(idx + 1)
        {
            let selected_tokens = &struct_slice[idx + 2..];

            let (parsed_value, jump_idx, _) = parse_value(
                selected_tokens,
                token_offset,
                debug_infos,
                origin_token_idx + idx,
                function_signatures.clone(),
                variable_scope,
                Some(
                    this_struct_field
                        .get(field_name)
                        .ok_or(ParserError::SyntaxError(
                            SyntaxError::InvalidStructFieldDefinition,
                        ))?
                        .clone(),
                ),
                function_imports.clone(),
                custom_types.clone(),
            )?;

            idx += jump_idx + 2;

            struct_field_init_map.insert(field_name.to_string(), Box::new(parsed_value));

            if let Some(Token::Comma) = struct_slice.get(idx) {
                nth_field += 1;
                idx += 1;
                continue;
            }
            else if nth_field + 1 == this_struct_field.len() {
                nth_field += 1;
                idx += 1;
                continue;
            }
        }

        return Err(ParserError::SyntaxError(SyntaxError::InvalidStructFieldDefinition).into());
    }

    Ok((
        idx,
        ParsedTokenInstance {
            inner: ParsedToken::Literal(crate::ty::Value::Struct((
                this_struct_name,
                this_struct_field.clone().into(),
                struct_field_init_map.into(),
                struct_attributes,
            ))),
            debug_information: fetch_and_merge_debug_information(
                debug_infos,
                origin_token_idx + token_offset..origin_token_idx + idx + token_offset,
                true,
            )
            .unwrap(),
        },
    ))
}
