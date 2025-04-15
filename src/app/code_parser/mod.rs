use std::cmp::Ordering;

use super::type_system::{Types, TypesDiscriminants};

pub mod code_parser;

#[derive(Debug)]
pub enum Tokens {
    Const(Types),
    Variable,
    TypeDefinition(TypesDiscriminants),

    Identifier(String),

    FunctionArgs,
    FunctionName,

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

    LineBreak,
    Comma,

    SetValue,
}

pub enum ParsedTokens {
    Variable(Types),
    Const(Types),

    Brackets(Vec<ParsedTokens>),

    FunctionCall(FunctionArguments),
    FunctionDefinition(FunctionDefinition),

    Comparison(Comparison),

    LogicGate(LogicGate),
}

pub struct FunctionArguments(Vec<ParsedTokens>);

pub struct FunctionDefinition {
    args: Vec<Types>,

    inner: Vec<ParsedTokens>,

    return_type: Types,
}

pub struct LogicGate {
    cmp: Comparison,

    inner: Vec<ParsedTokens>,
}

pub struct Comparison {
    rhs: Types,

    lhs: Types,

    ord: Ordering,
}