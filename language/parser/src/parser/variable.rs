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
        variable::{StructFieldReference, VariableReference, handle_variable},
    },
    tokenizer::Token,
    tracing::info,
    ty::{Type, ty_from_token},
};

/// This function parses the tokens after a variable.
/// This function parses actions related to variables. Such as: `var + 5` and `var =% 3`, etc.
/// TODO: Make this fn have a side effect on `var_ref` and just wrap the value into a parsed token instance at the end
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
    variable_name: &str,
) -> Result<ParsedTokenInstance>
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
                variable_scope,
                variable_type,
                &mut variable_ref,
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
                variable_scope,
                variable_type,
                &mut variable_ref,
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
                variable_scope,
                variable_type,
                &mut variable_ref,
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
                variable_scope,
                variable_type,
                &mut variable_ref,
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
                variable_scope,
                variable_type,
                &mut variable_ref,
                MathematicalSymbol::Modulo,
                function_imports.clone(),
                custom_types.clone(),
            )?;
        },
        Token::Dot => {
            let var_type = handle_variable(
                tokens,
                function_token_offset,
                debug_infos,
                origin_token_idx,
                &function_signatures,
                variable_scope,
                None,
                token_idx,
                &function_imports,
                &custom_types,
                variable_name,
                variable_ref.inner.try_as_variable_reference_mut().unwrap(),
                variable_type,
            )?;

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
            let var_type = handle_variable(
                tokens,
                function_token_offset,
                debug_infos,
                origin_token_idx,
                &function_signatures,
                variable_scope,
                None,
                token_idx,
                &function_imports,
                &custom_types,
                variable_name,
                variable_ref.inner.try_as_variable_reference_mut().unwrap(),
                variable_type,
            )?;
        },
        // Token::As => {
        //     if let Some(Token::TypeDefinition(target_type)) = tokens.get(*token_idx + 1) {
        //         let desired_variable_type =
        //             desired_variable_type.ok_or(ParserError::InternalDesiredTypeMissing)?;

        //         if *target_type != desired_variable_type {
        //             return Err(ParserError::TypeMismatch(
        //                 target_type.clone(),
        //                 desired_variable_type,
        //             )
        //             .into());
        //         }

        //         // Increment the token index after checking target type
        //         *token_idx += 2;

        //         let parsed_tkn = parse_variable_expression(
        //             tokens,
        //             function_token_offset,
        //             debug_infos,
        //             current_token,
        //             token_idx,
        //             function_signatures,
        //             function_imports,
        //             variable_scope,
        //             variable_type,
        //             custom_types,
        //             ParsedTokenInstance {
        //                 inner: ParsedToken::TypeCast(Box::new(variable_ref), target_type.clone()),
        //                 debug_information: fetch_and_merge_debug_information(
        //                     debug_infos,
        //                     origin_token_idx + function_token_offset
        //                         ..origin_token_idx + *token_idx + function_token_offset,
        //                     true,
        //                 )
        //                 .unwrap(),
        //             },
        //             parsed_tokens,
        //             variable_name,
        //         )?;

        //         // Return the type casted literal
        //         return Ok(parsed_tkn);
        //     }
        //     else {
        //         // Throw an error
        //         return Err(ParserError::SyntaxError(SyntaxError::AsRequiresTypeDef).into());
        //     }
        // },
        _ => {
            info!("[ERROR] Unimplemented token: {}", tokens[*token_idx]);
        },
    }

    Ok(variable_ref)
}

fn set_value_math_expr(
    tokens: &[Token],
    token_offset: usize,
    debug_infos: &[DbgInfo],
    function_signatures: Rc<IndexMap<String, UnparsedFunctionDefinition>>,
    token_idx: &mut usize,
    variable_scope: &mut IndexMap<String, Type>,
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
