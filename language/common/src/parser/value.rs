use crate::{error::parser::ParserError, parser::function::PathMap, tokenizer::Token};
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
    codegen::CustomItem,
    error::{SpanInfo, syntax::SyntaxError},
    parser::{
        common::{ParsedTokenInstance, StatementVariant},
        dbg::fetch_and_merge_debug_information,
        function::{FunctionSignature, UnparsedFunctionDefinition},
        variable::UniqueId,
    },
    ty::Type,
};

/// This is a top level implementation for `parse_token_as_value`
pub fn parse_value(
    _tokens: &[Token],
    _function_tokens_offset: usize,
    _debug_infos: &[SpanInfo],
    _origin_token_idx: usize,
    _function_signatures: Rc<PathMap<Vec<String>, String, UnparsedFunctionDefinition>>,
    _variable_scope: &mut IndexMap<String, (Type, UniqueId)>,
    // Always pass in the desired variable type, you can only leave this `None` if you dont know the type by design
    _desired_variable_type: Option<Type>,
    _function_imports: Rc<HashMap<String, FunctionSignature>>,
    _custom_types: Rc<IndexMap<String, CustomItem>>,
    _module_path: Vec<String>,
) -> Result<(ParsedTokenInstance, usize, Type)>
{
    Ok(todo!())
}

/// Parses the next token as something that holds a value:
/// Like: FunctionCall, Literal, UnparsedLiteral
pub fn parse_token_as_value(
    // This is used to parse the function call's arguments
    _tokens: &[Token],
    _function_token_offset: usize,
    _debug_infos: &[SpanInfo],
    _origin_token_idx: usize,
    // Functions available
    _function_signatures: Rc<PathMap<Vec<String>, String, UnparsedFunctionDefinition>>,
    // Variables available
    _variable_scope: &mut IndexMap<String, (Type, UniqueId)>,
    // The variable's type which we are parsing for
    _desired_variable_type: Option<Type>,
    // Universal token_idx, this sets which token we are currently parsing
    _token_idx: &mut usize,
    // The token we want to evaluate, this is the first token of the slice most of the time
    _eval_token: &Token,
    _function_imports: Rc<HashMap<String, FunctionSignature>>,
    _custom_types: Rc<IndexMap<String, CustomItem>>,
    _module_path: Vec<String>,
) -> Result<(ParsedTokenInstance, Type)>
{
    Ok(todo!())
}

pub fn init_struct(
    struct_slice: &[Token],
    token_offset: usize,
    debug_infos: &[SpanInfo],
    origin_token_idx: usize,
    this_struct_field: &IndexMap<String, Type>,
    this_struct_name: String,
    function_signatures: Rc<PathMap<Vec<String>, String, UnparsedFunctionDefinition>>,
    function_imports: Rc<HashMap<String, FunctionSignature>>,
    custom_types: Rc<IndexMap<String, CustomItem>>,
    variable_scope: &mut IndexMap<String, (Type, UniqueId)>,
    struct_attributes: StructAttributes,
    module_path: Vec<String>,
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
                module_path.clone(),
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
            inner: StatementVariant::Literal(crate::ty::Value::Struct((
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
