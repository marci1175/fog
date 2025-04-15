use std::cmp::Ordering;

use super::type_system::{Type, TypeDiscriminants};

pub mod parser;

#[derive(Debug)]
pub enum Tokens {
    Const(Type),
    Variable,
    TypeDefinition(TypeDiscriminants),

    Identifier(String),
    Quote(String),
    Comment(String),

    Function,

    Multiplication,
    Division,
    Addition,
    Subtraction,

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

pub enum ParsedTokens {
    Variable(Type),
    Const(Type),

    Brackets(Vec<ParsedTokens>),

    FunctionCall(FunctionArguments),
    FunctionDefinition(FunctionDefinition),

    Comparison(Comparison),

    If(If),
}

pub struct FunctionArguments(Vec<ParsedTokens>);

pub struct FunctionDefinition {
    args: Vec<Type>,

    inner: Vec<ParsedTokens>,

    return_type: Type,
}

pub struct If {
    cmp: Comparison,

    inner: Vec<ParsedTokens>,
}

pub struct Comparison {
    rhs: Type,

    lhs: Type,

    ord: Ordering,
}
