use std::{cmp::Ordering, collections::HashMap};

use crate::app::type_system::{Type, TypeDiscriminants};

use super::error::ParserError;

#[derive(Debug, Clone, PartialEq, strum_macros::Display)]
pub enum Tokens {
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
    inner: Vec<ParsedTokens>,
}

#[derive(Debug, Clone)]
pub enum ParsedTokens {
    NewVariable((String, Type)),
    VariableReference(String),
    Literal(Type),

    Addition(Box<ParsedTokens>, Box<ParsedTokens>),
    Brackets(Vec<ParsedTokens>, TypeDiscriminants),

    FunctionCall((FunctionSignature, String), Vec<ParsedTokens>),

    SetValue(String, Box<ParsedTokens>),

    If(If),
}

#[derive(Clone, Debug)]
pub struct UnparsedFunctionDefinition {
    pub function_sig: FunctionSignature,
    pub inner: Vec<Tokens>,
}

#[derive(Debug, Clone)]
pub struct FunctionDefinition {
    pub function_sig: FunctionSignature,
    pub inner: Vec<ParsedTokens>,
}

#[derive(Debug, Clone)]
pub struct FunctionSignature {
    pub args: HashMap<String, TypeDiscriminants>,
    pub return_type: TypeDiscriminants,
}

#[derive(Debug, Clone)]
pub struct If {
    cmp: Comparison,

    inner: Vec<ParsedTokens>,
}

#[derive(Debug, Clone)]
pub struct Comparison {
    rhs: Type,

    lhs: Type,

    cmp: Cmp,
}

#[derive(Debug, Clone)]
pub struct Cmp {

}

// pub fn auto_cast_to_type(val: Type, dest_ty: TypeDiscriminants) -> Result<Type, ParserError> {
//     let recasted_val = match (val, dest_ty) {
//         (Type::I32(_), TypeDiscriminants::F32) => todo!(),
//         (Type::I32(_), TypeDiscriminants::U32) => todo!(),
//         (Type::I32(_), TypeDiscriminants::U8) => todo!(),
//         (Type::I32(_), TypeDiscriminants::String) => todo!(),
//         (Type::I32(_), TypeDiscriminants::Boolean) => todo!(),
//         (Type::I32(_), TypeDiscriminants::Void) => todo!(),
//         (Type::F32(_), TypeDiscriminants::I32) => todo!(),
//         (Type::F32(_), TypeDiscriminants::U32) => todo!(),
//         (Type::F32(_), TypeDiscriminants::U8) => todo!(),
//         (Type::F32(_), TypeDiscriminants::String) => todo!(),
//         (Type::F32(_), TypeDiscriminants::Boolean) => todo!(),
//         (Type::F32(_), TypeDiscriminants::Void) => todo!(),
//         (Type::U32(_), TypeDiscriminants::I32) => todo!(),
//         (Type::U32(_), TypeDiscriminants::F32) => todo!(),
//         (Type::U32(_), TypeDiscriminants::U8) => todo!(),
//         (Type::U32(_), TypeDiscriminants::String) => todo!(),
//         (Type::U32(_), TypeDiscriminants::Boolean) => todo!(),
//         (Type::U32(_), TypeDiscriminants::Void) => todo!(),
//         (Type::U8(_), TypeDiscriminants::I32) => todo!(),
//         (Type::U8(_), TypeDiscriminants::F32) => todo!(),
//         (Type::U8(_), TypeDiscriminants::U32) => todo!(),
//         (Type::U8(_), TypeDiscriminants::String) => todo!(),
//         (Type::U8(_), TypeDiscriminants::Boolean) => todo!(),
//         (Type::U8(_), TypeDiscriminants::Void) => todo!(),
//         (Type::String(_), TypeDiscriminants::I32) => todo!(),
//         (Type::String(_), TypeDiscriminants::F32) => todo!(),
//         (Type::String(_), TypeDiscriminants::U32) => todo!(),
//         (Type::String(_), TypeDiscriminants::U8) => todo!(),
//         (Type::String(_), TypeDiscriminants::Boolean) => todo!(),
//         (Type::String(_), TypeDiscriminants::Void) => todo!(),
//         (Type::Boolean(_), TypeDiscriminants::I32) => todo!(),
//         (Type::Boolean(_), TypeDiscriminants::F32) => todo!(),
//         (Type::Boolean(_), TypeDiscriminants::U32) => todo!(),
//         (Type::Boolean(_), TypeDiscriminants::U8) => todo!(),
//         (Type::Boolean(_), TypeDiscriminants::String) => todo!(),
//         (Type::Boolean(_), TypeDiscriminants::Void) => todo!(),
//         (Type::Void, TypeDiscriminants::I32) => todo!(),
//         (Type::Void, TypeDiscriminants::F32) => todo!(),
//         (Type::Void, TypeDiscriminants::U32) => todo!(),
//         (Type::Void, TypeDiscriminants::U8) => todo!(),
//         (Type::Void, TypeDiscriminants::String) => todo!(),
//         (Type::Void, TypeDiscriminants::Boolean) => todo!(),

//         _ => panic!(
//             "[INTERNAL ERROR] Automatic type conversion should never be called on an object which matches the destination type."
//         ),
//     };

//     Ok(recasted_val)
// }

pub fn unparsed_const_to_typed_literal(raw_string: String, dest_type: TypeDiscriminants) -> Result<Type, ParserError> {
    let typed_var = match dest_type {
        TypeDiscriminants::I32 => Type::I32(raw_string.parse::<i32>().map_err(|_| ParserError::ConstTypeUndetermined(raw_string, dest_type))?),
        TypeDiscriminants::F32 => Type::F32(raw_string.parse::<f32>().map_err(|_| ParserError::ConstTypeUndetermined(raw_string, dest_type))?),
        TypeDiscriminants::U32 => Type::U32(raw_string.parse::<u32>().map_err(|_| ParserError::ConstTypeUndetermined(raw_string, dest_type))?),
        TypeDiscriminants::U8 => Type::U8(raw_string.parse::<u8>().map_err(|_| ParserError::ConstTypeUndetermined(raw_string, dest_type))?),
        TypeDiscriminants::String => {
            return Err(ParserError::ConstTypeUndetermined(raw_string, dest_type).into());
        },
        TypeDiscriminants::Boolean => {
            if raw_string == "false" {
                Type::Boolean(false)
            }
            else if raw_string == "true" {
                Type::Boolean(true)
            }
            else {
                return Err(ParserError::ConstTypeUndetermined(raw_string, dest_type).into());
            }
        },
        TypeDiscriminants::Void => {
            return Err(ParserError::ConstTypeUndetermined(raw_string, dest_type).into());
        },
    };

    Ok(typed_var)
}