use std::collections::HashMap;

use strum_macros::Display;

use crate::app::type_system::type_system::{Type, TypeDiscriminants};

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
    TypeCast(Box<ParsedToken>, TypeDiscriminants),

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
