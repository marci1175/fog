use crate::{
    codegen::{LogicalOperator, Order},
    error::{Spanned, parser::ParserError},
    parser::{
        common::ItemVisibility, function::CompilerInstructionDiscriminants,
        numeric_value::MathematicalSymbol,
    },
    ty::{Type, Value},
};
use strum::EnumTryAs;

/// The basic output type of the tokenizer.
#[derive(Debug, Clone, PartialEq, strum_macros::Display, Eq, Hash, EnumTryAs)]
pub enum Token
{
    /// A literal is a concrete value.
    /// A literal may be created at the tokenization stage for concrete values.
    /// Numbers are not concrete as they could have different sizes or types depending on the length or accuracy of the number.
    Literal(Value),

    /// All numbers are first tokenized as an unparsed literal, since their type is not concrete at the tokenization stage.
    UnparsedLiteral(String),

    Identifier(String),
    DocComment(String),

    As,

    /// Used to flag variables as non-mutable: `const int marci = 0;`
    Const,
    /// This is used when defining statics through ffi
    Static,

    Variable,

    TypeDefinition(TypeToken),

    /// ... - for ffi functions only
    Ellipsis,
    Return,

    /// This is for caluclating with mathematical symbols: ```<val> <math expr> <val>```
    MathSym(MathematicalSymbol),

    /// This is for expressions directly modifying a variable: ```<val> <math expr>= <val>```
    SetValueMathSym(MathematicalSymbol),

    /// XOR, Or, AND can only be asserted between boolean values.
    LogicalOperator(LogicalOperator),

    Not,

    If,
    Else,
    ElseIf,

    Comparison(Order),

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

    /// Othervise known as equals.
    SetValue,

    /// &
    BitAnd,
    /// |
    BitOr,
    /// <<
    BitLeft,
    /// >>
    BitRight,

    /// For ffi decls
    External,

    /// For dependency imports
    Import,

    Loop,
    While,
    For,

    Continue,
    Break,

    /// Implements keyword for implementing for items
    Implements,

    Trait,

    /// This is used as the function receiver when implementing for items
    This,

    /// Item vis pub, publib etc etc
    ItemVisibility(ItemVisibility),

    /// @
    CompilerHintSymbol,
    /// The acutal keywords for the compiler
    CompilerInstruction(CompilerInstructionDiscriminants),

    /// Used to expose functions from a module into another one.
    Export,

    LeftArrow,
    RightArrow,
    /// This can be used as a substitute in function definitions in place of the `:` indicating return type.
    Returns,

    /// Used at "modules"
    Namespace,

    /// Pointer management
    Reference,
    Dereference,
    In,
}

impl PartialEq<TokenDiscriminants> for Spanned<Token>
{
    fn eq(&self, other: &TokenDiscriminants) -> bool
    {
        self.get_inner() == other
    }
}

impl PartialEq<TokenDiscriminants> for Token
{
    fn eq(&self, other: &TokenDiscriminants) -> bool
    {
        match self {
            Token::Literal(_) => other == &TokenDiscriminants::Literal,
            Token::SetValueMathSym(sym) | Token::MathSym(sym) => {
                match sym {
                    MathematicalSymbol::Addition => {
                        *other == TokenDiscriminants::SetValueMathSymAddition
                    },
                    MathematicalSymbol::Subtraction => {
                        *other == TokenDiscriminants::SetValueMathSymSubtraction
                    },
                    MathematicalSymbol::Division => {
                        *other == TokenDiscriminants::SetValueMathSymDivision
                    },
                    MathematicalSymbol::Multiplication => {
                        *other == TokenDiscriminants::SetValueMathSymMultiplication
                    },
                    MathematicalSymbol::Modulo => {
                        *other == TokenDiscriminants::SetValueMathSymModulo
                    },
                    MathematicalSymbol::Power => *other == TokenDiscriminants::SetValueMathSymPower,
                }
            },
            Token::UnparsedLiteral(_) => other == &TokenDiscriminants::UnparsedLiteral,
            Token::Identifier(_) => other == &TokenDiscriminants::Identifier,
            Token::DocComment(_) => other == &TokenDiscriminants::DocComment,
            Token::TypeDefinition(_) => other == &TokenDiscriminants::TypeDefinition,
            Token::CompilerInstruction(_) => other == &TokenDiscriminants::CompilerInstruction,
            Token::ItemVisibility(_) => other == &TokenDiscriminants::ItemVisibility,
            Token::As => other == &TokenDiscriminants::As,
            Token::Const => other == &TokenDiscriminants::Const,
            Token::Variable => other == &TokenDiscriminants::Variable,
            Token::Ellipsis => other == &TokenDiscriminants::Ellipsis,
            Token::Return => other == &TokenDiscriminants::Return,
            Token::LogicalOperator(_) => other == &TokenDiscriminants::LogicalOperator,
            Token::Not => other == &TokenDiscriminants::Not,
            Token::If => other == &TokenDiscriminants::If,
            Token::Else => other == &TokenDiscriminants::Else,
            Token::ElseIf => other == &TokenDiscriminants::ElseIf,
            Token::Comparison(_) => other == &TokenDiscriminants::Comparison,
            Token::OpenParentheses => other == &TokenDiscriminants::OpenParentheses,
            Token::CloseParentheses => other == &TokenDiscriminants::CloseParentheses,
            Token::OpenBraces => other == &TokenDiscriminants::OpenBraces,
            Token::CloseBraces => other == &TokenDiscriminants::CloseBraces,
            Token::OpenSquareBrackets => other == &TokenDiscriminants::OpenSquareBrackets,
            Token::CloseSquareBrackets => other == &TokenDiscriminants::CloseSquareBrackets,
            Token::OpenAngledBrackets => other == &TokenDiscriminants::OpenAngledBrackets,
            Token::CloseAngledBrackets => other == &TokenDiscriminants::CloseAngledBrackets,
            Token::SemiColon => other == &TokenDiscriminants::SemiColon,
            Token::Comma => other == &TokenDiscriminants::Comma,
            Token::DoubleColon => other == &TokenDiscriminants::DoubleColon,
            Token::Colon => other == &TokenDiscriminants::Colon,
            Token::Dot => other == &TokenDiscriminants::Dot,
            Token::SetValue => other == &TokenDiscriminants::SetValue,
            Token::BitAnd => other == &TokenDiscriminants::BitAnd,
            Token::BitOr => other == &TokenDiscriminants::BitOr,
            Token::BitLeft => other == &TokenDiscriminants::BitLeft,
            Token::BitRight => other == &TokenDiscriminants::BitRight,
            Token::External => other == &TokenDiscriminants::External,
            Token::Import => other == &TokenDiscriminants::Import,
            Token::Loop => other == &TokenDiscriminants::Loop,
            Token::While => other == &TokenDiscriminants::While,
            Token::For => other == &TokenDiscriminants::For,
            Token::Continue => other == &TokenDiscriminants::Continue,
            Token::Break => other == &TokenDiscriminants::Break,
            Token::Implements => other == &TokenDiscriminants::Implements,
            Token::Trait => other == &TokenDiscriminants::Trait,
            Token::This => other == &TokenDiscriminants::This,
            Token::CompilerHintSymbol => other == &TokenDiscriminants::CompilerHintSymbol,
            Token::Export => other == &TokenDiscriminants::Export,
            Token::LeftArrow => other == &TokenDiscriminants::LeftArrow,
            Token::RightArrow => other == &TokenDiscriminants::RightArrow,
            Token::Returns => other == &TokenDiscriminants::Returns,
            Token::Namespace => other == &TokenDiscriminants::Namespace,
            Token::Reference => other == &TokenDiscriminants::Reference,
            Token::Dereference => other == &TokenDiscriminants::Dereference,
            Token::Static => other == &TokenDiscriminants::Static,
            Token::In => other == &TokenDiscriminants::In,
        }
    }
}

/// The reason this enum is rich is because some fields have inner values.
/// These does not work for the [`expr_pat!`] macro as it requires the enum to be u32 castable. A "base" discriminant is also available next to this enum for a few helper methods.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u32)]
pub enum TokenDiscriminants
{
    /// A literal is a concrete value.
    /// A literal may be created at the tokenization stage for concrete values.
    /// Numbers are not concrete as they could have different sizes or types depending on the length or accuracy of the number.
    Literal,

    /// All numbers are first tokenized as an unparsed literal, since their type is not concrete at the tokenization stage.
    UnparsedLiteral,

    Identifier,
    DocComment,

    As,

    Const,
    /// Used to flag variables as non-mutable: `const int marci = 0;`
    Variable,

    TypeDefinition,

    Ellipsis,
    Return,

    /// Flattened MathSym variant.
    MathSymAddition,
    MathSymSubtraction,
    MathSymDivision,
    MathSymMultiplication,
    MathSymModulo,
    MathSymPower,

    /// Flattened SetValueMathSym variant.
    SetValueMathSymAddition,
    SetValueMathSymSubtraction,
    SetValueMathSymDivision,
    SetValueMathSymMultiplication,
    SetValueMathSymModulo,
    SetValueMathSymPower,

    LogicalOperator,
    Not,

    If,
    Else,
    ElseIf,

    Comparison,

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

    /// Othervise known as equals.
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

    ItemVisibility,

    /// @
    CompilerHintSymbol,
    CompilerInstruction,

    /// Used to expose functions from a module into another one.
    Export,

    LeftArrow,
    RightArrow,
    /// This can be used as a substitute in function definitions in place of the `:` indicating return type.
    Returns,

    Namespace,

    Reference,
    Dereference,
    Static,
    In,
}

/// This are only the type indicating tokens, not the actual types themselves.
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

    Pointer,
    Function,
}

impl TryInto<Type> for TypeToken
{
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Type, Self::Error>
    {
        Ok(match self {
            TypeToken::I64 => Type::I64,
            TypeToken::F64 => Type::F64,
            TypeToken::U64 => Type::U64,
            TypeToken::I32 => Type::I32,
            TypeToken::F32 => Type::F32,
            TypeToken::U32 => Type::U32,
            TypeToken::I16 => Type::I16,
            TypeToken::F16 => Type::F16,
            TypeToken::U16 => Type::U16,
            TypeToken::U8 => Type::U8,
            TypeToken::String => Type::String,
            TypeToken::Boolean => Type::Boolean,
            TypeToken::Void => Type::Void,
            TypeToken::Pointer => Type::Pointer(None),
            TypeToken::Enum | TypeToken::Array | TypeToken::Struct | TypeToken::Function => {
                return Err(ParserError::InternalTypetokenNotConvertable.into());
            },
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, strum_macros::Display, Eq, Hash, EnumTryAs)]
pub enum Comparison
{
    Equal,
    NotEqual,
    Bigger,
    EqBigger,
    Smaller,
    EqSmaller,
}
