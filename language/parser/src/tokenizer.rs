use std::{ops::Range, u8};

use common::{
    anyhow,
    error::{CharPosition, DbgInfo, parser::ParserError, syntax::SyntaxError},
    parser::function::CompilerHint,
    tokenizer::{Token, find_closing_angled_bracket_char},
    ty::{Type, Value},
};

pub fn only_contains_digits(s: &[u8]) -> bool
{
    s.iter().all(|c| c.is_ascii_digit())
}

const DOUBLE_BACKSLASH: u8 = b'\\';
const NEWLINE_CHAR: u8 = b'\n';
const ENDLINE_CHAR_U8: u8 = b'\r';

// TODO: Recode this function as its too crowded, i should also move parsing stuff out of this functions (such as ptr, enum, array etc, its doing smth that is not its job)
pub fn tokenize(
    raw_input: &str,
    stop_at_token: Option<Token>,
) -> anyhow::Result<(Vec<Token>, Vec<DbgInfo>, usize)>
{
    todo!()
}

fn match_multi_character_expression(string_to_match: &[u8]) -> anyhow::Result<Token>
{
    todo!()
}

pub fn eval_constant_definition(raw_string: &[u8]) -> Token
{
    let string = String::from_utf8_lossy(raw_string);

    let mut negative_flag = false;
    let mut float_flag = false;

    let is_number = raw_string.iter().enumerate().all(|(idx, byte)| {
        if *byte == b'.' && (float_flag || idx == 0) {
            return false;
        }
        if *byte == b'-' && (negative_flag || idx != 0) {
            return false;
        }

        if *byte == b'.' {
            float_flag = true;
            return true;
        }
        if *byte == b'-' {
            negative_flag = true;
            return true;
        }

        byte.is_ascii_digit()
    });

    if is_number {
        Token::UnparsedLiteral(string.to_string())
    }
    else {
        Token::Identifier(string.to_string())
    }
}