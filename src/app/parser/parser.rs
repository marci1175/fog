use std::{collections::{HashMap, HashSet}, sync::Arc};
use anyhow::Result;
use crate::app::type_system::TypeDiscriminants;

use super::{
    error::ParserError,
    parse_functions::{self, create_function_table, parse_functions},
    types::{
        FunctionDefinition, ParsedToken, Token, UnparsedFunctionDefinition,
        unparsed_const_to_typed_literal,
    },
};

#[derive(Debug, Clone)]
pub struct ParserState {
    tokens: Vec<Token>,

    function_table: HashMap<String, FunctionDefinition>,

    string_definitions: HashSet<String>,
}

impl ParserState {
    pub fn parse_tokens(&mut self) -> Result<()> {
        let unparsed_functions = create_function_table(self.tokens.clone())?;

        self.function_table = parse_functions(Arc::new(unparsed_functions))?;

        Ok(())
    }

    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            function_table: HashMap::new(),
            string_definitions: HashSet::new(),
        }
    }

    pub fn function_table(&self) -> &HashMap<String, FunctionDefinition> {
        &self.function_table
    }
}

pub fn find_closing_bracket(bracket_start_slice: &[Token]) -> Result<usize> {
    let mut bracket_layer_counter = 1;
    let iter = bracket_start_slice.iter().enumerate();

    for (idx, token) in iter {
        match token {
            Token::OpenBracket => bracket_layer_counter += 1,
            Token::CloseBracket => {
                bracket_layer_counter -= 1;
                if bracket_layer_counter == 0 {
                    return Ok(idx);
                }
            }
            _ => continue,
        }
    }

    Err(ParserError::SyntaxError(super::error::SyntaxError::OpenBraces).into())
}

pub fn parse_set_value(
    tokens: &[Token],
    parsed_tokens: &mut Vec<ParsedToken>,
    function_signatures: Arc<HashMap<String, UnparsedFunctionDefinition>>,
    variable_scope: &HashMap<String, TypeDiscriminants>,
    variable_type: TypeDiscriminants,
    variable_name: String,
) -> Result<()> {
    let mut token_idx = 0;

    while token_idx < tokens.len() {
        let current_token = &tokens
            .get(token_idx)
            .ok_or_else(|| ParserError::SyntaxError(crate::app::parser::error::SyntaxError::InvalidMathematicalExpressionDefinition))?;

        // Please note that we are not looking at values by themselves, except in SetValue where we take the next token.
        match current_token {
            Token::Addition | Token::Subtraction | Token::Multiplication | Token::Division => {
                let last_parsed = parsed_tokens.last_mut().ok_or(ParserError::SyntaxError(crate::app::parser::error::SyntaxError::InvalidMathematicalExpressionDefinition))?;

                let next_token = &tokens
                    .get(token_idx + 1)
                    .ok_or_else(|| ParserError::SyntaxError(crate::app::parser::error::SyntaxError::InvalidMathematicalExpressionDefinition))?;

                if let ParsedToken::SetValue(_variable_ref, value) = last_parsed {
                    let mathematical_expr = ParsedToken::MathematicalExpression(
                        value.clone(),
                        // Match the current token with the mathematical expression
                        (*current_token).clone().try_into()?,
                        Box::new(parse_token_as_value(
                            tokens,
                            &function_signatures,
                            variable_scope,
                            variable_type,
                            &mut token_idx,
                            next_token,
                        )?),
                    );

                    *value = Box::new(mathematical_expr);
                }

                continue;
            }
            // If the first token is a `SetValue` take the next token and turn it into a `ParsedToken` which will be in the SetValue token.
            Token::SetValue => {
                // Grab the next token in the list
                let next_token = &tokens
                    .get(token_idx + 1)
                    .ok_or_else(|| ParserError::SyntaxError(crate::app::parser::error::SyntaxError::InvalidMathematicalExpressionDefinition))?;

                // Pattern match the token into a ParsedToken
                let inner_value = parse_token_as_value(
                    tokens,
                    &function_signatures,
                    variable_scope,
                    variable_type,
                    &mut token_idx,
                    next_token,
                )?;

                parsed_tokens.push(ParsedToken::SetValue(
                    variable_name.clone(),
                    Box::new(inner_value),
                ));

                continue;
            }
            Token::Return => {}

            _ => unimplemented!(),
        }

        token_idx += 2;
    }

    Ok(())
}

/// Parses the next token as something that holds a value:
/// Like: FunctionCall, Literal, UnparsedLiteral
pub fn parse_token_as_value(
    // This is used to parse the function call's arguments
    tokens: &[Token],
    // Functions available
    function_signatures: &Arc<HashMap<String, UnparsedFunctionDefinition>>,
    // Variables available
    variable_scope: &HashMap<String, TypeDiscriminants>,
    // The variable's type which we are parsing for
    variable_type: TypeDiscriminants,
    // Universal token_idx, this sets which token we are currently parsing
    token_idx: &mut usize,
    // The next token in the list
    next_token: &Token,
) -> Result<ParsedToken> {
    // Match the token
    let inner_value = match next_token {
        Token::Literal(literal) => {
            *token_idx += 2;
            ParsedToken::Literal(literal.clone())
        }
        Token::UnparsedLiteral(unparsed_literal) => {
            *token_idx += 2;
            ParsedToken::Literal(unparsed_const_to_typed_literal(
                unparsed_literal.clone(),
                variable_type,
            )?)
        }
        Token::Identifier(identifier) => {
            // Try to find the identifier in the functions' list
            if let Some(function) = function_signatures.get(identifier) {
                // Parse the call arguments and tokens parsed.
                let (call_arguments, idx_jmp) = parse_functions::parse_function_call_args(
                    &tokens[*token_idx + 3..],
                    variable_scope,
                    function.function_sig.args.clone(),
                )?;

                // Increment the token index, and add the offset
                *token_idx += idx_jmp + 4;

                // Return the function call
                ParsedToken::FunctionCall(
                    (function.function_sig.clone(), identifier.clone()),
                    call_arguments,
                )
            // If the identifier could not be found in the function list search in the variable scope
            } else if let Some(variable) = variable_scope.get(identifier) {
                // If the variable's type doesnt match the one we want to modify throw an error.
                if variable_type != *variable {
                    return Err(ParserError::TypeError(*variable, variable_type).into());
                }

                // Return the VariableReference
                ParsedToken::VariableReference(identifier.clone())
            } else {
                // If none of the above matches throw an error about the variable not being found
                return Err(ParserError::VariableNotFound(identifier.clone()).into());
            }
        }

        _ => return Err(ParserError::SyntaxError(super::error::SyntaxError::InvalidValue(next_token.clone())).into()),
    };
    Ok(inner_value)
}
