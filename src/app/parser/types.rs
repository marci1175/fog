use std::collections::HashMap;

use strum::IntoDiscriminant;
use strum_macros::Display;

use crate::app::type_system::{Type, TypeDiscriminants};

use super::error::ParserError;

#[derive(Debug, Clone, PartialEq, strum_macros::Display)]
pub enum Token {
    Literal(Type),

    UnparsedLiteral(String),

    TypeDefinition(TypeDiscriminants),
    As,

    Identifier(String),
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
pub enum MathematicalSymbol {
    Addition,
    Subtraction,
    Division,
    Multiplication,
}

impl TryInto<MathematicalSymbol> for Token {
    type Error = ParserError;

    fn try_into(self) -> Result<MathematicalSymbol, Self::Error> {
        let expr = match self {
            Self::Addition => MathematicalSymbol::Addition,
            Self::Subtraction => MathematicalSymbol::Subtraction,
            Self::Division => MathematicalSymbol::Division,
            Self::Multiplication => MathematicalSymbol::Multiplication,

            _ => return Err(ParserError::InternalVariableError),
        };

        Ok(expr)
    }
}

#[derive(Debug, Clone, Display)]
pub enum ParsedToken {
    NewVariable((String, Box<ParsedToken>)),
    VariableReference(String),
    Literal(Type),

    MathematicalExpression(Box<ParsedToken>, MathematicalSymbol, Box<ParsedToken>),

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
            return Err(ParserError::ConstTypeUndetermined(raw_string, dest_type));
        }
        TypeDiscriminants::Boolean => {
            if raw_string == "false" {
                Type::Boolean(false)
            } else if raw_string == "true" {
                Type::Boolean(true)
            } else {
                return Err(ParserError::ConstTypeUndetermined(raw_string, dest_type));
            }
        }
        TypeDiscriminants::Void => {
            return Err(ParserError::ConstTypeUndetermined(raw_string, dest_type));
        }
    };

    Ok(typed_var)
}

pub fn convert_as(value: Type, dest_type: TypeDiscriminants) -> anyhow::Result<Type> {
    if value.discriminant() == dest_type {
        return Ok(value);
    }

    if dest_type == TypeDiscriminants::Void {
        return Ok(Type::Void);
    }

    let return_val = match value {
        Type::I32(inner) => match dest_type {
            TypeDiscriminants::F32 => Type::F32(inner as f32),
            TypeDiscriminants::U32 => Type::U32(inner as u32),
            TypeDiscriminants::U8 => Type::U8(inner as u8),
            TypeDiscriminants::String => Type::String(inner.to_string()),
            TypeDiscriminants::Boolean => {
                if inner == 1 {
                    Type::Boolean(true)
                } else {
                    Type::Boolean(false)
                }
            }

            TypeDiscriminants::I32 | TypeDiscriminants::Void => unreachable!(),
        },
        Type::F32(inner) => match dest_type {
            TypeDiscriminants::I32 => Type::I32(inner as i32),
            TypeDiscriminants::U32 => Type::U32(inner as u32),
            TypeDiscriminants::U8 => Type::U8(inner as u8),
            TypeDiscriminants::String => Type::String(inner.to_string()),
            TypeDiscriminants::Boolean => {
                if inner == 1.0 {
                    Type::Boolean(true)
                } else {
                    Type::Boolean(false)
                }
            }

            TypeDiscriminants::F32 | TypeDiscriminants::Void => unreachable!(),
        },
        Type::U32(inner) => match dest_type {
            TypeDiscriminants::F32 => Type::F32(inner as f32),
            TypeDiscriminants::I32 => Type::I32(inner as i32),
            TypeDiscriminants::U8 => Type::U8(inner as u8),
            TypeDiscriminants::String => Type::String(inner.to_string()),
            TypeDiscriminants::Boolean => {
                if inner == 1 {
                    Type::Boolean(true)
                } else {
                    Type::Boolean(false)
                }
            }

            TypeDiscriminants::U32 | TypeDiscriminants::Void => unreachable!(),
        },
        Type::U8(inner) => match dest_type {
            TypeDiscriminants::F32 => Type::F32(inner as f32),
            TypeDiscriminants::I32 => Type::I32(inner as i32),
            TypeDiscriminants::U32 => Type::U32(inner as u32),
            TypeDiscriminants::String => Type::String(inner.to_string()),
            TypeDiscriminants::Boolean => {
                if inner == 1 {
                    Type::Boolean(true)
                } else {
                    Type::Boolean(false)
                }
            }

            TypeDiscriminants::U8 | TypeDiscriminants::Void => unreachable!(),
        },
        Type::String(inner) => match dest_type {
            TypeDiscriminants::I32 => Type::I32(inner.parse::<i32>()?),
            TypeDiscriminants::F32 => Type::F32(inner.parse::<f32>()?),
            TypeDiscriminants::U32 => Type::U32(inner.parse::<u32>()?),
            TypeDiscriminants::U8 => Type::U8(inner.parse::<u8>()?),
            TypeDiscriminants::Boolean => Type::Boolean(inner.parse::<bool>()?),

            TypeDiscriminants::String | TypeDiscriminants::Void => unreachable!(),
        },

        Type::Boolean(inner) => match dest_type {
            TypeDiscriminants::I32 => Type::I32(inner as i32),
            TypeDiscriminants::F32 => Type::F32(inner as i32 as f32),
            TypeDiscriminants::U32 => Type::U32(inner as u32),
            TypeDiscriminants::U8 => Type::U8(inner as u8),
            TypeDiscriminants::String => Type::String(inner.to_string()),

            TypeDiscriminants::Boolean | TypeDiscriminants::Void => unreachable!(),
        },
        Type::Void => unreachable!(),
    };

    Ok(return_val)
}
