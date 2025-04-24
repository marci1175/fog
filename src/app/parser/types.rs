use std::{cmp::Ordering, collections::HashMap};

use strum_macros::Display;

use crate::app::type_system::{Type, TypeDiscriminants};

use super::error::ParserError;

#[derive(Debug, Clone, PartialEq, strum_macros::Display)]
pub enum Token {
    Literal(Type),

    UnparsedLiteral(String),

    TypeDefinition(TypeDiscriminants),

    Identifier(String),
    Quote(String),
    Comment(String),

    Function,
    Return,

    Multiplication,
    Division,
    Addition,
    Subtraction,
    Modulo,
    SetValueMultiplication,
    SetValueDivision,
    SetValueAddition,
    SetValueSubtraction,
    SetValueModulo,

    And,
    Or,
    Not,

    If,

    Equals,
    Bigger,
    EqBigger,
    Smaller,
    EqSmaller,

    OpenBracket,
    CloseBracket,
    OpenBraces,
    CloseBraces,

    LineBreak,
    Comma,
    Colon,

    SetValue,

    BitAnd,
    BitOr,
    BitLeft,
    BitRight,
}

#[derive(Debug, Clone)]
pub struct TokenBlock {
    inner: Vec<ParsedToken>,
}

#[derive(Debug, Clone)]
pub enum MathematicalExpressionType {
    Addition,
    Subtraction,
    Division,
    Multiplication,
}

impl TryInto<MathematicalExpressionType> for Token {
    type Error = ParserError;

    fn try_into(self) -> Result<MathematicalExpressionType, Self::Error> {
        let expr = match self {
            Self::Addition => MathematicalExpressionType::Addition, 
            Self::Subtraction => MathematicalExpressionType::Subtraction, 
            Self::Division => MathematicalExpressionType::Division, 
            Self::Multiplication => MathematicalExpressionType::Multiplication,

            _ => return Err(ParserError::InternalVariableError) 
        };

        Ok(expr)
    }
}

#[derive(Debug, Clone, Display)]
pub enum ParsedToken {
    NewVariable((String, Box<ParsedToken>)),
    VariableReference(String),
    Literal(Type),

    MathematicalExpression(Box<ParsedToken>, MathematicalExpressionType, Box<ParsedToken>),

    Brackets(Vec<ParsedToken>, TypeDiscriminants),

    FunctionCall((FunctionSignature, String), Vec<ParsedToken>),

    SetValue(String, Box<ParsedToken>),

    If(If),
}

#[derive(Clone, Debug)]
pub struct UnparsedFunctionDefinition {
    pub function_sig: FunctionSignature,
    pub inner: Vec<Token>,
}

#[derive(Debug, Clone)]
pub struct FunctionDefinition {
    pub function_sig: FunctionSignature,
    pub inner: Vec<ParsedToken>,
}

#[derive(Debug, Clone)]
pub struct FunctionSignature {
    pub args: HashMap<String, TypeDiscriminants>,
    pub return_type: TypeDiscriminants,
}

#[derive(Debug, Clone)]
pub struct If {
    cmp: Comparison,

    inner: Vec<ParsedToken>,
}

#[derive(Debug, Clone)]
pub struct Comparison {
    rhs: Type,

    lhs: Type,

    cmp: Cmp,
}

#[derive(Debug, Clone)]
pub struct Cmp {}

pub fn unparsed_const_to_typed_literal(
    raw_string: String,
    dest_type: TypeDiscriminants,
) -> Result<Type, ParserError> {
    let typed_var = match dest_type {
        TypeDiscriminants::I32 => Type::I32(
            raw_string
                .parse::<i32>()
                .map_err(|_| ParserError::ConstTypeUndetermined(raw_string, dest_type))?,
        ),
        TypeDiscriminants::F32 => Type::F32(
            raw_string
                .parse::<f32>()
                .map_err(|_| ParserError::ConstTypeUndetermined(raw_string, dest_type))?,
        ),
        TypeDiscriminants::U32 => Type::U32(
            raw_string
                .parse::<u32>()
                .map_err(|_| ParserError::ConstTypeUndetermined(raw_string, dest_type))?,
        ),
        TypeDiscriminants::U8 => Type::U8(
            raw_string
                .parse::<u8>()
                .map_err(|_| ParserError::ConstTypeUndetermined(raw_string, dest_type))?,
        ),
        TypeDiscriminants::String => {
            return Err(ParserError::ConstTypeUndetermined(raw_string, dest_type).into());
        }
        TypeDiscriminants::Boolean => {
            if raw_string == "false" {
                Type::Boolean(false)
            } else if raw_string == "true" {
                Type::Boolean(true)
            } else {
                return Err(ParserError::ConstTypeUndetermined(raw_string, dest_type).into());
            }
        }
        TypeDiscriminants::Void => {
            return Err(ParserError::ConstTypeUndetermined(raw_string, dest_type).into());
        }
    };

    Ok(typed_var)
}
