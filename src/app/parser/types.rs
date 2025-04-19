use std::{cmp::Ordering, collections::HashMap};

use crate::app::type_system::{Type, TypeDiscriminants};

use super::error::ParserError;

#[derive(Debug, Clone, PartialEq, strum_macros::Display)]
pub enum Tokens {
    Const(Type),
    Variable,
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
pub enum ParsedTokens {
    Variable((String, Type)),

    Brackets(Vec<ParsedTokens>),

    FunctionCall(Vec<String>),

    Comparison(Comparison),

    SetValue((String, Type)),

    If(If),
}

// #[derive(Debug)]
// pub struct FunctionArguments();

#[derive(Clone, Debug)]
pub struct UnparsedFunctionDefinition {
    pub args: HashMap<String, TypeDiscriminants>,
    pub inner: Vec<Tokens>,
    pub return_type: TypeDiscriminants,
}

#[derive(Debug, Clone)]
pub struct FunctionDefinition {
    pub args: HashMap<String, TypeDiscriminants>,
    pub inner: Vec<ParsedTokens>,
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

    ord: Ordering,
}

pub fn auto_cast_to_type(val: Type, dest_ty: TypeDiscriminants) -> Result<Type, ParserError> {
    let recasted_val = match (val, dest_ty) {
        (Type::I32(_), TypeDiscriminants::F32) => todo!(),
        (Type::I32(_), TypeDiscriminants::U32) => todo!(),
        (Type::I32(_), TypeDiscriminants::U8) => todo!(),
        (Type::I32(_), TypeDiscriminants::String) => todo!(),
        (Type::I32(_), TypeDiscriminants::Boolean) => todo!(),
        (Type::I32(_), TypeDiscriminants::Void) => todo!(),
        (Type::F32(_), TypeDiscriminants::I32) => todo!(),
        (Type::F32(_), TypeDiscriminants::U32) => todo!(),
        (Type::F32(_), TypeDiscriminants::U8) => todo!(),
        (Type::F32(_), TypeDiscriminants::String) => todo!(),
        (Type::F32(_), TypeDiscriminants::Boolean) => todo!(),
        (Type::F32(_), TypeDiscriminants::Void) => todo!(),
        (Type::U32(_), TypeDiscriminants::I32) => todo!(),
        (Type::U32(_), TypeDiscriminants::F32) => todo!(),
        (Type::U32(_), TypeDiscriminants::U8) => todo!(),
        (Type::U32(_), TypeDiscriminants::String) => todo!(),
        (Type::U32(_), TypeDiscriminants::Boolean) => todo!(),
        (Type::U32(_), TypeDiscriminants::Void) => todo!(),
        (Type::U8(_), TypeDiscriminants::I32) => todo!(),
        (Type::U8(_), TypeDiscriminants::F32) => todo!(),
        (Type::U8(_), TypeDiscriminants::U32) => todo!(),
        (Type::U8(_), TypeDiscriminants::String) => todo!(),
        (Type::U8(_), TypeDiscriminants::Boolean) => todo!(),
        (Type::U8(_), TypeDiscriminants::Void) => todo!(),
        (Type::String(_), TypeDiscriminants::I32) => todo!(),
        (Type::String(_), TypeDiscriminants::F32) => todo!(),
        (Type::String(_), TypeDiscriminants::U32) => todo!(),
        (Type::String(_), TypeDiscriminants::U8) => todo!(),
        (Type::String(_), TypeDiscriminants::Boolean) => todo!(),
        (Type::String(_), TypeDiscriminants::Void) => todo!(),
        (Type::Boolean(_), TypeDiscriminants::I32) => todo!(),
        (Type::Boolean(_), TypeDiscriminants::F32) => todo!(),
        (Type::Boolean(_), TypeDiscriminants::U32) => todo!(),
        (Type::Boolean(_), TypeDiscriminants::U8) => todo!(),
        (Type::Boolean(_), TypeDiscriminants::String) => todo!(),
        (Type::Boolean(_), TypeDiscriminants::Void) => todo!(),
        (Type::Void, TypeDiscriminants::I32) => todo!(),
        (Type::Void, TypeDiscriminants::F32) => todo!(),
        (Type::Void, TypeDiscriminants::U32) => todo!(),
        (Type::Void, TypeDiscriminants::U8) => todo!(),
        (Type::Void, TypeDiscriminants::String) => todo!(),
        (Type::Void, TypeDiscriminants::Boolean) => todo!(),

        _ => panic!("[INTERNAL ERROR] Automatic type conversion should never be called on an object which matches the destination type.")
    };

    Ok(recasted_val)
}