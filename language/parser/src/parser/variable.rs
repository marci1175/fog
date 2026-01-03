use std::{collections::HashMap, rc::Rc};

use common::{
    anyhow::Result,
    codegen::CustomType,
    error::{DbgInfo, parser::ParserError, syntax::SyntaxError},
    indexmap::IndexMap,
    parser::{
        common::{ParsedToken, ParsedTokenInstance},
        dbg::fetch_and_merge_debug_information,
        function::{FunctionSignature, UnparsedFunctionDefinition},
        value::{MathematicalSymbol, parse_token_as_value, parse_value},
        variable::{StructFieldReference, VariableReference},
    },
    tokenizer::Token,
    tracing::info,
    ty::{Type, ty_from_token},
};

/// This function parses the tokens after a variable.
/// This function parses actions related to variables. Such as: `var + 5` and `var =% 3`, etc.
pub fn parse_variable_expression(
    tokens: &[Token],
    // Token slice offset, this allows us to keep the correct slice indexing (without ruining token_idx)
    function_token_offset: usize,
    debug_infos: &[DbgInfo],
    current_token: &Token,
    token_idx: &mut usize,
    function_signatures: Rc<IndexMap<String, UnparsedFunctionDefinition>>,
    function_imports: Rc<HashMap<String, FunctionSignature>>,
    variable_scope: &mut IndexMap<String, Type>,
    variable_type: Type,
    custom_types: Rc<IndexMap<String, CustomType>>,
    mut variable_ref: ParsedTokenInstance,
    parsed_tokens: &mut Vec<ParsedTokenInstance>,
) -> Result<()>
{
    let origin_token_idx = *token_idx;

    match &current_token {
        Token::SetValue => {
            // Find next line break aka `;`
            let line_break_idx = tokens
                .iter()
                .skip(*token_idx)
                .position(|token| *token == Token::SemiColon)
                .ok_or(ParserError::SyntaxError(SyntaxError::MissingSemiColon))?
                + *token_idx;

            // Tokens that contain the value we set the variable to
            let selected_tokens = &tokens[*token_idx + 1..line_break_idx];

            // Increment the token_idx to the next expression `Token::LineBreak` + 1
            *token_idx += selected_tokens.len() + 1;

            // Parse the value we would be setting the variable to
            let (parsed_token, _, _) = parse_value(
                selected_tokens,
                function_token_offset,
                debug_infos,
                origin_token_idx,
                function_signatures.clone(),
                variable_scope,
                Some(variable_type.clone()),
                function_imports.clone(),
                custom_types.clone(),
            )?;

            parsed_tokens.push(ParsedTokenInstance {
                inner: ParsedToken::SetValue(
                    Box::new(variable_ref.clone()),
                    Box::new(parsed_token),
                ),
                debug_information: fetch_and_merge_debug_information(
                    debug_infos,
                    function_token_offset + origin_token_idx..function_token_offset + *token_idx,
                    true,
                )
                .unwrap(),
            });
        },
        Token::SetValueAddition => {
            set_value_math_expr(
                tokens,
                function_token_offset,
                debug_infos,
                function_signatures,
                token_idx,
                parsed_tokens,
                variable_scope,
                variable_type,
                variable_ref.clone(),
                MathematicalSymbol::Addition,
                function_imports.clone(),
                custom_types.clone(),
            )?;
        },
        Token::SetValueSubtraction => {
            set_value_math_expr(
                tokens,
                function_token_offset,
                debug_infos,
                function_signatures,
                token_idx,
                parsed_tokens,
                variable_scope,
                variable_type,
                variable_ref.clone(),
                MathematicalSymbol::Subtraction,
                function_imports.clone(),
                custom_types.clone(),
            )?;
        },
        Token::SetValueDivision => {
            set_value_math_expr(
                tokens,
                function_token_offset,
                debug_infos,
                function_signatures,
                token_idx,
                parsed_tokens,
                variable_scope,
                variable_type,
                variable_ref.clone(),
                MathematicalSymbol::Division,
                function_imports.clone(),
                custom_types.clone(),
            )?;
        },
        Token::SetValueMultiplication => {
            set_value_math_expr(
                tokens,
                function_token_offset,
                debug_infos,
                function_signatures,
                token_idx,
                parsed_tokens,
                variable_scope,
                variable_type,
                variable_ref.clone(),
                MathematicalSymbol::Multiplication,
                function_imports.clone(),
                custom_types.clone(),
            )?;
        },
        Token::SetValueModulo => {
            set_value_math_expr(
                tokens,
                function_token_offset,
                debug_infos,
                function_signatures,
                token_idx,
                parsed_tokens,
                variable_scope,
                variable_type,
                variable_ref.clone(),
                MathematicalSymbol::Modulo,
                function_imports.clone(),
                custom_types.clone(),
            )?;
        },
        Token::Dot => {
            let field_name = &tokens.get(*token_idx + 1);

            if let Type::Struct((struct_name, struct_def)) = variable_type {
                if let Some(Token::Identifier(field_name)) = field_name {
                    if let Some(struct_field_ty) = struct_def.get(field_name) {
                        if let ParsedTokenInstance {
                            inner: ParsedToken::VariableReference(var_ref),
                            ..
                        } = variable_ref
                        {
                            let new_reference = match var_ref {
                                VariableReference::StructFieldReference(
                                    mut struct_field_ref,
                                    struct_ty,
                                ) => {
                                    struct_field_ref.field_stack.push(field_name.to_string());
                                    VariableReference::StructFieldReference(
                                        struct_field_ref,
                                        struct_ty,
                                    )
                                },
                                VariableReference::BasicReference(basic_ref) => {
                                    let stack = vec![basic_ref.to_string(), field_name.to_string()];
                                    VariableReference::StructFieldReference(
                                        StructFieldReference::from_stack(stack),
                                        (struct_name, struct_def.clone()),
                                    )
                                },

                                VariableReference::ArrayReference(_, _) => {
                                    todo!()
                                },
                            };
                            variable_ref = ParsedTokenInstance {
                                inner: ParsedToken::VariableReference(new_reference),
                                debug_information: DbgInfo::default(),
                            };
                        }

                        *token_idx += 2;

                        parse_variable_expression(
                            tokens,
                            function_token_offset,
                            debug_infos,
                            &tokens[*token_idx],
                            token_idx,
                            function_signatures,
                            function_imports,
                            variable_scope,
                            struct_field_ty.clone(),
                            custom_types,
                            // This is not going to work we may need to rework struct references.
                            variable_ref,
                            parsed_tokens,
                        )?;
                    }
                    else {
                        return Err(ParserError::SyntaxError(SyntaxError::StructFieldNotFound(
                            field_name.to_string(),
                            (struct_name, struct_def),
                        ))
                        .into());
                    }
                }
                else {
                    return Err(ParserError::SyntaxError(SyntaxError::StructFieldNotFound(
                        format!("{field_name:?}"),
                        (struct_name, struct_def),
                    ))
                    .into());
                }
            }
            else {
                return Err(ParserError::SyntaxError(SyntaxError::InvalidDotPlacement).into());
            }

            if let Some(idx) = tokens
                .iter()
                .skip(*token_idx)
                .position(|token| *token == Token::SemiColon)
            {
                *token_idx += idx;
            }
            else {
                return Err(ParserError::SyntaxError(SyntaxError::MissingSemiColon).into());
            }
        },
        Token::OpenSquareBrackets => {
            if let Type::Array((inner_token, _len)) = variable_type {
                let inner_type = ty_from_token(&inner_token, &custom_types)?;

                *token_idx += 1;

                let square_brackets_break_idx = tokens
                    .iter()
                    .skip(*token_idx)
                    .position(|token| *token == Token::CloseSquareBrackets)
                    .ok_or(ParserError::SyntaxError(
                        SyntaxError::LeftOpenSquareBrackets,
                    ))?
                    + *token_idx;

                let selected_tokens = &tokens[*token_idx..square_brackets_break_idx];

                let (value, idx_jmp, _) = parse_value(
                    selected_tokens,
                    *token_idx,
                    debug_infos,
                    origin_token_idx,
                    function_signatures.clone(),
                    variable_scope,
                    Some(Type::U32),
                    function_imports.clone(),
                    custom_types.clone(),
                )?;

                *token_idx += idx_jmp;

                if let Some(Token::CloseSquareBrackets) = tokens.get(*token_idx) {
                    *token_idx += 1;

                    if tokens.get(*token_idx) != Some(&Token::SemiColon) {
                        let next_token = tokens.get(*token_idx).ok_or(ParserError::SyntaxError(
                            SyntaxError::InvalidStatementDefinition,
                        ))?;

                        parse_variable_expression(
                            tokens,
                            *token_idx,
                            debug_infos,
                            next_token,
                            token_idx,
                            function_signatures.clone(),
                            function_imports,
                            variable_scope,
                            inner_type,
                            custom_types,
                            ParsedTokenInstance {
                                inner: ParsedToken::ArrayIndexing(
                                    Box::new(variable_ref.clone()),
                                    Box::new(value.clone()),
                                ),
                                debug_information: fetch_and_merge_debug_information(
                                    debug_infos,
                                    origin_token_idx + function_token_offset
                                        ..*token_idx + function_token_offset,
                                    true,
                                )
                                .unwrap(),
                            },
                            parsed_tokens,
                        )?;
                    }
                    else {
                        // parsed_tokens.push(ParsedToken::ArrayIndexing(
                        //     Box::new(ParsedToken::VariableReference(variable_ref.clone())),
                        //     Box::new(value),
                        // ));

                        panic!("Check later if this is a syntax check.")
                    }
                }
                else {
                    return Err(
                        ParserError::SyntaxError(SyntaxError::LeftOpenSquareBrackets).into(),
                    );
                }
            }
            else {
                return Err(ParserError::TypeMismatchNonIndexable(variable_type).into());
            }
        },
        _ => {
            info!("[ERROR] Unimplemented token: {}", tokens[*token_idx]);
        },
    }

    Ok(())
}

fn set_value_math_expr(
    tokens: &[Token],
    token_offset: usize,
    debug_infos: &[DbgInfo],
    function_signatures: Rc<IndexMap<String, UnparsedFunctionDefinition>>,
    token_idx: &mut usize,
    parsed_tokens: &mut Vec<ParsedTokenInstance>,
    variable_scope: &mut IndexMap<String, Type>,
    variable_type: Type,
    variable_reference: ParsedTokenInstance,
    math_symbol: MathematicalSymbol,
    standard_function_table: Rc<HashMap<String, FunctionSignature>>,
    custom_items: Rc<IndexMap<String, CustomType>>,
) -> Result<()>
{
    let origin_token_idx = *token_idx;

    *token_idx += 1;

    let eval_token = tokens.get(*token_idx).ok_or(ParserError::SyntaxError(
        SyntaxError::InvalidStatementDefinition,
    ))?;

    let (next_token, _ty) = parse_token_as_value(
        tokens,
        token_offset,
        debug_infos,
        origin_token_idx,
        function_signatures,
        variable_scope,
        Some(variable_type.clone()),
        token_idx,
        eval_token,
        standard_function_table,
        custom_items.clone(),
    )?;

    parsed_tokens.push(ParsedTokenInstance {
        inner: ParsedToken::SetValue(
            Box::new(variable_reference.clone()),
            Box::new(ParsedTokenInstance {
                inner: ParsedToken::MathematicalExpression(
                    Box::new(variable_reference),
                    math_symbol,
                    Box::new(next_token),
                ),
                debug_information: fetch_and_merge_debug_information(
                    debug_infos,
                    origin_token_idx + token_offset..*token_idx + token_offset,
                    true,
                )
                .unwrap(),
            }),
        ),
        debug_information: fetch_and_merge_debug_information(
            debug_infos,
            origin_token_idx + token_offset..*token_idx + token_offset,
            true,
        )
        .unwrap(),
    });

    Ok(())
}
