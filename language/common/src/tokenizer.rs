use std::sync::Arc;

use crate::{
    error::SpanInfo,
    parser::function::CompilerHint,
    ty::{Type, Value},
};

/// The basic output type of the tokenizer.
#[derive(Debug, Clone, PartialEq, strum_macros::Display, Eq, Hash)]
pub enum Token
{
    Literal(Value),

    UnparsedLiteral(String),

    Identifier(String),
    DocComment(String),

    As,

    Const, // Used to flag variables as non-mutable: `const int marci = 0;`
    Variable,

    TypeDefinition(TypeToken),

    Function,
    Ellipsis,
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
    Else,
    ElseIf,

    Equal,
    NotEqual,
    Bigger,
    EqBigger,
    Smaller,
    EqSmaller,

    OpenParentheses,
    CloseParentheses,
    OpenBraces,
    CloseBraces,
    OpenSquareBrackets,
    CloseSquareBrackets,
    OpenAngledBrackets,
    CloseAngledBrackets,

    SemiColon,
    Comma,
    DoubleColon,
    Colon,
    Dot,

    SetValue,

    BitAnd,
    BitOr,
    BitLeft,
    BitRight,

    External,
    Import,

    Loop,
    While,
    For,

    Continue,
    Break,

    Implements,
    Trait,
    This,

    Private,       // priv
    Public,        // pub
    PublicLibrary, // publib

    CompilerHintSymbol, // @
    CompilerHint(CompilerHint),

    /// Used to expose functions from a module into another one.
    Export,

    LeftArrow,
    RightArrow,
    /// This can be used as a substitute in function definitions in place of the `:` indicating return type.
    Returns,

    Namespace,
}

/// This are only the tpye indicating tokens, not the actual types themselves.
/// This is just for organizing the tokens basically.
#[derive(Debug, Clone, PartialEq, strum_macros::Display, Eq, Hash)]
pub enum TypeToken
{
    I64,
    F64,
    U64,

    I32,
    F32,
    U32,

    I16,
    F16,
    U16,

    U8,

    String,
    Boolean,

    Void,
    Enum,
    Array,
    Struct,

    /// ref
    /// Example: ```ptr foo = ref bar;```
    Reference,
    /// deref
    /// Example: ```int foo = deref bar;```
    Dereference,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Spanned<T>
{
    inner: T,
    span: SpanInfo,
}

impl<T> Spanned<T>
{
    pub fn new(inner: T, span: SpanInfo) -> Self
    {
        Self { inner, span }
    }
}
