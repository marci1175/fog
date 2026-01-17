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
        value::MathematicalSymbol,
        variable::{ArrayIndexing, UniqueId, VariableReference, get_struct_field_stack},
    },
    tokenizer::Token,
    tracing,
    ty::{Type, ty_from_token},
};

use crate::parser::value::{parse_token_as_value, parse_value};

/// This function parses the tokens after a variable.
/// This function parses actions related to variables. Such as: `var + 5` and `var =% 3`, etc.
/// TODO: Make this fn have a side effect on `var_ref` and just wrap the value into a parsed token instance at the end
pub fn resolve_variable_expression(
    tokens: &[Token],
    // Token slice offset, this allows us to keep the correct slice indexing (without ruining token_idx)
    function_token_offset: usize,
    debug_infos: &[DbgInfo],
    token_idx: &mut usize,
    function_signatures: Rc<IndexMap<String, UnparsedFunctionDefinition>>,
    function_imports: Rc<HashMap<String, FunctionSignature>>,
    variable_scope: &mut IndexMap<String, (Type, UniqueId)>,
    (variable_type, variable_id): (Type, UniqueId),
    custom_types: Rc<IndexMap<String, CustomType>>,
    variable_ref: &mut ParsedTokenInstance,
    parsed_tokens: &mut Vec<ParsedTokenInstance>,
    variable_name: &str,
) -> Result<Type>
{
    let var_ref = variable_ref.inner.try_as_variable_reference_mut().unwrap();
    let origin_token_idx = *token_idx;

    let current_token = tokens.get(*token_idx);

    if let Some(current_token) = current_token {
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
                        function_token_offset + origin_token_idx
                            ..function_token_offset + *token_idx,
                        true,
                    )
                    .unwrap(),
                });
            },
            Token::SetValueAddition
            | Token::SetValueSubtraction
            | Token::SetValueDivision
            | Token::SetValueMultiplication
            | Token::SetValueModulo => {
                set_value_math_expr(
                    tokens,
                    function_token_offset,
                    debug_infos,
                    function_signatures,
                    token_idx,
                    variable_scope,
                    variable_type.clone(),
                    variable_ref,
                    current_token.clone().try_into()?,
                    function_imports.clone(),
                    custom_types.clone(),
                )?;
            },
            Token::OpenSquareBrackets => {
                if !matches!(variable_type, Type::Array(_)) {
                    return Err(ParserError::TypeMismatchNonIndexable(variable_type.clone()).into());
                }

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

                let (value, _idx_jmp, _) = parse_value(
                    selected_tokens,
                    function_token_offset + *token_idx,
                    debug_infos,
                    origin_token_idx,
                    function_signatures.clone(),
                    variable_scope,
                    Some(Type::U32),
                    function_imports.clone(),
                    custom_types.clone(),
                )?;

                *token_idx = square_brackets_break_idx;

                if let Some(Token::CloseSquareBrackets) = tokens.get(*token_idx) {
                    *token_idx += 1;

                    *var_ref = VariableReference::ArrayReference(ArrayIndexing {
                        variable_reference: Box::new(var_ref.clone()),
                        idx: Box::new(value.clone()),
                    });

                    if let Type::Array((inner_ty, _len)) = variable_type.clone() {
                        let ty = resolve_variable_expression(
                            tokens,
                            function_token_offset,
                            debug_infos,
                            token_idx,
                            function_signatures,
                            function_imports,
                            variable_scope,
                            (ty_from_token(&inner_ty, &custom_types)?, variable_id),
                            custom_types,
                            variable_ref,
                            parsed_tokens,
                            variable_name,
                        )?;

                        return Ok(ty);
                    }
                    else {
                        unreachable!(
                            "This is unreachable as there is a type check at the beginning of this code."
                        );
                    }
                }
                else {
                    return Err(
                        ParserError::SyntaxError(SyntaxError::LeftOpenSquareBrackets).into(),
                    );
                }
            },
            Token::Dot => {
                if let Type::Struct(struct_def) = variable_type {
                    *token_idx += 1;

                    // Stack the field names on top of the variable name
                    let field_type = get_struct_field_stack(
                        tokens,
                        token_idx,
                        variable_name,
                        &struct_def,
                        var_ref,
                    )?;

                    // Continue parsing it
                    let ty = resolve_variable_expression(
                        tokens,
                        function_token_offset,
                        debug_infos,
                        token_idx,
                        function_signatures,
                        function_imports,
                        variable_scope,
                        // Even though this is a field of a struct it still falls under the same original variable id
                        (field_type, variable_id),
                        custom_types,
                        variable_ref,
                        parsed_tokens,
                        variable_name,
                    )?;

                    return Ok(ty);
                }
                else {
                    return Err(ParserError::TypeWithoutFields(variable_type).into());
                }
            },
            Token::SemiColon => {
                *token_idx += 1;
            },
            Token::As => {
                if let Some(Token::TypeDefinition(target_type)) = tokens.get(*token_idx + 1) {
                    // let desired_variable_type =
                    //     desired_variable_type.ok_or(ParserError::InternalDesiredTypeMissing)?;
                    // if *target_type != desired_variable_type {
                    //     return Err(ParserError::TypeMismatch(
                    //         target_type.clone(),
                    //         desired_variable_type,
                    //     )
                    //     .into());
                    // }

                    // Increment the token index after checking target type
                    *token_idx += 2;

                    resolve_variable_expression(
                        tokens,
                        function_token_offset,
                        debug_infos,
                        token_idx,
                        function_signatures,
                        function_imports,
                        variable_scope,
                        (variable_type, variable_id),
                        custom_types,
                        &mut ParsedTokenInstance {
                            inner: ParsedToken::TypeCast(
                                Box::new((*variable_ref).clone()),
                                target_type.clone(),
                            ),
                            debug_information: fetch_and_merge_debug_information(
                                debug_infos,
                                origin_token_idx + function_token_offset
                                    ..origin_token_idx + *token_idx + function_token_offset,
                                true,
                            )
                            .unwrap(),
                        },
                        parsed_tokens,
                        variable_name,
                    )?;

                    // Return the type casted literal
                    return Ok(target_type.clone());
                }
                else {
                    // Throw an error
                    return Err(ParserError::SyntaxError(SyntaxError::AsRequiresTypeDef).into());
                }
            },
            // Everything else should be igonred as we can assume it is a part of another expression
            // All of the variable expressions are captured by the tokens before.
            // Since this function parses everything after the indent, it can happen that only a variable is referenced.
            // In which case, this function would want to parse that expression unrelated to this function.
            // ie: `var1 > var2` Parsing will start at `>` which is not related to the var1 variable
            // This trace can be ignored.
            _ => {
                tracing::trace!("Invalid variable expr token: {}", tokens[*token_idx]);
            },
        }
    }

    // If we didnt return anything before this we can return variable type.
    Ok(variable_type)
}

pub fn set_value_math_expr(
    tokens: &[Token],
    token_offset: usize,
    debug_infos: &[DbgInfo],
    function_signatures: Rc<IndexMap<String, UnparsedFunctionDefinition>>,
    token_idx: &mut usize,
    variable_scope: &mut IndexMap<String, (Type, UniqueId)>,
    variable_type: Type,
    variable_reference: &mut ParsedTokenInstance,
    math_symbol: MathematicalSymbol,
    standard_function_table: Rc<HashMap<String, FunctionSignature>>,
    custom_items: Rc<IndexMap<String, CustomType>>,
) -> Result<()>
{
    let origin_token_idx = *token_idx;

    *token_idx += 1;

    let eval_token = tokens.get(*token_idx).ok_or(ParserError::SyntaxError(
        SyntaxError::InvalidVariableDefinition,
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

    *variable_reference = ParsedTokenInstance {
        inner: ParsedToken::SetValue(
            Box::new(variable_reference.clone()),
            Box::new(ParsedTokenInstance {
                inner: ParsedToken::MathematicalExpression(
                    Box::new(variable_reference.clone()),
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
    };

    Ok(())
}
