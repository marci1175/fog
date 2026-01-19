use crate::{
    error::{parser::ParserError, syntax::SyntaxError},
    parser::function::CompilerHint,
    ty::{Type, Value},
};

/// The basic output type of the tokenizer.
#[derive(Debug, Clone, PartialEq, strum_macros::Display, Eq, Hash)]
pub enum Token
{
    Literal(Value),

    UnparsedLiteral(String),

    /// ref
    /// Example: ```ptr foo = ref bar;```
    Reference,
    /// deref
    /// Example: ```int foo = deref bar;```
    Dereference,

    Identifier(String),
    DocComment(String),

    As,

    Const, // Used to flag variables as non-mutable (vars a mutable by default): `const int marci = 0;`
    Struct,
    TypeDefinition(Type),

    /// Kinda like C enums but with any type
    Enum(Option<Box<Token>>),

    Extend,
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
    
    Impls,
    Trait,
    This,

    Private,       // priv
    Public,        // pub
    PublicLibrary, // publib

    CompilerHintSymbol, // @
    CompilerHint(CompilerHint),

    /// Used to expose functions from a module into another one.
    Export,
}

// impl Token {
//     pub fn return_error(error_type: ParserError, char_range: Range<usize>) -> anyhow::Error {
//         error_type.into()
//     }
// }

/// Pass in 0 for the `open_paren_count` if you're searching for the very next closing token on the same level.
pub fn find_closing_angled_bracket_char(
    paren_start_slice: &[u8],
    angled_bracket_count: usize,
) -> Result<usize, ParserError>
{
    let mut paren_layer_counter = 1;
    for (idx, token) in paren_start_slice.iter().enumerate() {
        match token {
            b'<' => paren_layer_counter += 1,
            b'>' => {
                paren_layer_counter -= 1;
                if paren_layer_counter == angled_bracket_count {
                    return Ok(idx);
                }
            },
            _ => continue,
        }
    }

    Err(ParserError::SyntaxError(
        SyntaxError::LeftOpenAngledBrackets,
    ))
}
