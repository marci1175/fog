use crate::{
    error::{Spanned, parser::ParserError},
    parser::{common::ItemVisibility, function::CompilerInstructionDiscriminants},
    ty::{Type, Value},
};
use strum::{EnumDiscriminants, EnumTryAs};

/// The basic output type of the tokenizer.
#[derive(
    Debug, Clone, PartialEq, strum_macros::Display, Eq, Hash, EnumTryAs, EnumDiscriminants,
)]
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

    ItemVisibility(ItemVisibility),

    CompilerHintSymbol, // @
    CompilerInstruction(CompilerInstructionDiscriminants),

    /// Used to expose functions from a module into another one.
    Export,

    LeftArrow,
    RightArrow,
    /// This can be used as a substitute in function definitions in place of the `:` indicating return type.
    Returns,

    Namespace,
    Use,

    Reference,
    Dereference,
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
            Token::Multiplication => other == &TokenDiscriminants::Multiplication,
            Token::Division => other == &TokenDiscriminants::Division,
            Token::Addition => other == &TokenDiscriminants::Addition,
            Token::Subtraction => other == &TokenDiscriminants::Subtraction,
            Token::Modulo => other == &TokenDiscriminants::Modulo,
            Token::SetValueMultiplication => other == &TokenDiscriminants::SetValueMultiplication,
            Token::SetValueDivision => other == &TokenDiscriminants::SetValueDivision,
            Token::SetValueAddition => other == &TokenDiscriminants::SetValueAddition,
            Token::SetValueSubtraction => other == &TokenDiscriminants::SetValueSubtraction,
            Token::SetValueModulo => other == &TokenDiscriminants::SetValueModulo,
            Token::And => other == &TokenDiscriminants::And,
            Token::Or => other == &TokenDiscriminants::Or,
            Token::Not => other == &TokenDiscriminants::Not,
            Token::If => other == &TokenDiscriminants::If,
            Token::Else => other == &TokenDiscriminants::Else,
            Token::ElseIf => other == &TokenDiscriminants::ElseIf,
            Token::Equal => other == &TokenDiscriminants::Equal,
            Token::NotEqual => other == &TokenDiscriminants::NotEqual,
            Token::Bigger => other == &TokenDiscriminants::Bigger,
            Token::EqBigger => other == &TokenDiscriminants::EqBigger,
            Token::Smaller => other == &TokenDiscriminants::Smaller,
            Token::EqSmaller => other == &TokenDiscriminants::EqSmaller,
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
            Token::Use => other == &TokenDiscriminants::Use,
        }
    }
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
