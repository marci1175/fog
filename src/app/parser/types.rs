use std::{cmp::Ordering, collections::HashMap};

use crate::app::type_system::{Type, TypeDiscriminants};

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug)]
pub enum ParsedTokens {
    Variable((String, Type)),

    Brackets(Vec<ParsedTokens>),

    FunctionCall(FunctionArguments),
    FunctionDefinition(FunctionDefinition),

    Comparison(Comparison),

    If(If),
}

#[derive(Debug)]
pub struct FunctionArguments(Vec<ParsedTokens>);

#[derive(Debug)]
pub struct FunctionDefinition {
    pub args: HashMap<String, TypeDiscriminants>,
    pub inner: Vec<ParsedTokens>,
    pub return_type: TypeDiscriminants,
}

#[derive(Debug)]
pub struct If {
    cmp: Comparison,

    inner: Vec<ParsedTokens>,
}

#[derive(Debug)]
pub struct Comparison {
    rhs: Type,

    lhs: Type,

    ord: Ordering,
}
