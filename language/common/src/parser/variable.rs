use std::{collections::HashMap, rc::Rc};

use crate::{
    codegen::CustomType,
    error::{DbgInfo, parser::ParserError, syntax::SyntaxError},
    parser::{
        common::{ParsedToken, ParsedTokenInstance},
        dbg::fetch_and_merge_debug_information,
        function::{FunctionSignature, UnparsedFunctionDefinition},
        value::parse_value,
    },
    tokenizer::Token,
    ty::{OrdMap, Type, ty_from_token},
};
use indexmap::IndexMap;
use strum_macros::Display;

#[derive(Debug, Clone, Display, PartialEq, Eq, Hash)]
pub enum ControlFlowType
{
    Break,
    Continue,
}

#[derive(Debug, Clone, Display, PartialEq, Eq, Hash, strum_macros::EnumTryAs)]
/// VariableReferences are the lowest layer of referencing a variable. This is enum wrapped in a ParsedToken, consult the documentation of that enum variant for more information.ű
/// VariableReferences should not contain themselves as they are only for referencing a variable, there is not much more to it.
pub enum VariableReference
{
    /// Struct field reference
    StructFieldReference(StructFieldRef),
    /// Variable name
    BasicReference(String),
    /// Variable name, array index
    ArrayReference(ArrayIndexing),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StructFieldRef
{
    /// Variable reference enum this shows the underlying variable its referring from
    pub variable_ref: Box<VariableReference>,
    /// This field is for verifying types (Even if the struct fields match it could still be two different structs)
    pub struct_name: String,
    /// The actual struct body, this contains the fields paired with their types. (In order of insertion)
    /// This field uses an [`OrdMap`] so that [`Hash`] can be implemented.
    pub struct_fields: OrdMap<String, Type>,
    /// This is the fields name we are refering
    pub field_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ArrayIndexing
{
    pub variable_reference: Box<VariableReference>,
    pub idx: Box<ParsedTokenInstance>,
}

/// The first item of the StructFieldReference is used to look up the name of the variable which stores the Struct.
/// The functions which take the iterator of the `field_stack` field should not be passed the first item of the iterator, since the first item is used to look up the name of the variable which stores the struct.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StructFieldReference
{
    /// The name of the fields which get referenced
    pub field_stack: Vec<String>,
}

impl Default for StructFieldReference
{
    fn default() -> Self
    {
        Self::new()
    }
}

impl StructFieldReference
{
    /// Creates an instnace from a single entry
    pub fn from_single_entry(field_name: String) -> Self
    {
        Self {
            field_stack: vec![field_name],
        }
    }

    /// Initializes an instance from a list of field entries
    pub fn from_stack(field_stack: Vec<String>) -> Self
    {
        Self { field_stack }
    }

    /// Creates an instnace from an empty list
    pub fn new() -> Self
    {
        Self {
            // Most struct references are one field deep, might aswell take a guess
            field_stack: Vec::with_capacity(1),
        }
    }
}

pub fn handle_variable(
    tokens: &[Token],
    function_token_offset: usize,
    debug_infos: &[DbgInfo],
    origin_token_idx: usize,
    function_signatures: &Rc<IndexMap<String, UnparsedFunctionDefinition>>,
    variable_scope: &mut IndexMap<String, Type>,
    desired_variable_type: Option<Type>,
    token_idx: &mut usize,
    function_imports: &Rc<HashMap<String, FunctionSignature>>,
    custom_types: &Rc<IndexMap<String, CustomType>>,
    variable_name: &str,
    variable_reference: &mut VariableReference,
    // Last parsed token's type
    variable_type: Type,
) -> anyhow::Result<Type>
{
    if let Some(Token::Dot) = tokens.get(*token_idx) {
        if let Type::Struct(struct_def) = variable_type {
            *token_idx += 1;

            // Stack the field names on top of the variable name
            let field_type = get_struct_field_stack(
                tokens,
                token_idx,
                variable_name,
                &struct_def,
                variable_reference,
            )?;

            // Continue parsing it
            let handling_continuation = handle_variable(
                tokens,
                function_token_offset,
                debug_infos,
                origin_token_idx,
                function_signatures,
                variable_scope,
                desired_variable_type,
                token_idx,
                function_imports,
                custom_types,
                variable_name,
                variable_reference,
                field_type,
            )?;

            Ok(handling_continuation)
        }
        else {
            Err(
                ParserError::SyntaxError(SyntaxError::InvalidStructName(variable_name.to_string()))
                    .into(),
            )
        }
    }
    else if let Some(Token::OpenSquareBrackets) = tokens.get(*token_idx) {
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

        let (value, idx_jmp, _) = parse_value(
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

        *token_idx += idx_jmp;

        if let Some(Token::CloseSquareBrackets) = tokens.get(*token_idx) {
            *token_idx += 1;

            *variable_reference = VariableReference::ArrayReference(ArrayIndexing {
                variable_reference: Box::new(variable_reference.clone()),
                idx: Box::new(value.clone()),
            });

            if let Type::Array((inner_ty, _len)) = variable_type.clone() {
                let handling_continuation = handle_variable(
                    tokens,
                    function_token_offset,
                    debug_infos,
                    origin_token_idx,
                    function_signatures,
                    variable_scope,
                    desired_variable_type,
                    token_idx,
                    function_imports,
                    custom_types,
                    variable_name,
                    variable_reference,
                    ty_from_token(&inner_ty, custom_types)?,
                )?;

                Ok(handling_continuation)
            }
            else {
                unreachable!(
                    "This is unreachable as there is a type check at the beginning of this code."
                );
            }
        }
        else {
            Err(ParserError::SyntaxError(SyntaxError::LeftOpenSquareBrackets).into())
        }
    }
    else {
        Ok(variable_type)
    }
}

/// Parses the tokens passed in and stores the field names into [`StructFieldReference`].
/// This function returns the last field's type.
/// TODO: When i want to implement traits and the ability to call functions on types. ( function foo(self, x: int) ) This needs modification.
fn get_struct_field_stack(
    tokens: &[Token],
    token_idx: &mut usize,
    identifier: &str,
    (struct_name, struct_fields): &(String, OrdMap<String, Type>),
    var_ref: &mut VariableReference,
) -> anyhow::Result<Type>
{
    // Match field name
    if let Some(Token::Identifier(field_name)) = tokens.get(*token_idx) {
        // Lookup struct field
        let struct_field_query = struct_fields.get(field_name);

        // Store field name
        *var_ref = VariableReference::StructFieldReference(StructFieldRef {
            variable_ref: Box::new(var_ref.clone()),
            struct_name: struct_name.clone(),
            struct_fields: struct_fields.clone(),
            field_name: field_name.clone(),
        });

        // If it is not a struct but is a some store the struct field name and return
        if let Some(field_type) = struct_field_query {
            *token_idx += 1;

            // Match syntax
            if let Some(Token::Dot) = tokens.get(*token_idx) {
                // Increment idx
                *token_idx += 1;

                let next_struct_def = field_type
                    .try_as_struct_ref()
                    .ok_or(ParserError::TypeWithoutFields(field_type.clone()))?;

                // Call this function once again and iterate
                get_struct_field_stack(tokens, token_idx, identifier, next_struct_def, var_ref)
            }
            else {
                // Return field type
                Ok(field_type.clone())
            }
        }
        else {
            Err(ParserError::SyntaxError(SyntaxError::StructFieldNotFound(
                field_name.clone(),
                (struct_name.clone(), struct_fields.clone()),
            ))
            .into())
        }
    }
    else {
        Err(ParserError::SyntaxError(SyntaxError::InvalidStructFieldReference).into())
    }
}
