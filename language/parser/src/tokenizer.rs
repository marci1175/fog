use common::{
    anyhow,
    error::{CharPosition, SpanInfo},
    parser::function::CompilerHint,
    tokenizer::{Spanned, Token, TypeToken},
    ty::{Type, Value},
};
use std::{sync::Arc, u8};

pub fn tokenize(input: &str) -> anyhow::Result<Vec<Spanned<Token>>>
{
    let mut token_list: Vec<Spanned<Token>> = Vec::new();

    for (line_idx, line) in input.lines().enumerate() {
        let line_number = line_idx + 1;

        let mut column_idx = 0;

        for raw_text in line.split_inclusive(char::is_whitespace) {
            let trimmed_text = raw_text.trim();

            let column_idx_begin = column_idx;

            // Increment column idx by the text length (This includes whitespace)
            column_idx += raw_text.len();

            let column_idx_end = column_idx;

            // Try to match full keywords present
            if let Some(tkn) = try_match_token(trimmed_text.as_bytes()) {
                token_list.push(Spanned::new(
                    tkn,
                    SpanInfo::new(
                        CharPosition::new(line_number, column_idx_begin),
                        CharPosition::new(line_number, column_idx_end),
                    ),
                ));
            }
            // If the token was not immediately recognized it is most likely number or an Identifier
            // If the trimmed text could also be an empty string.
            else if !trimmed_text.is_empty() {
                parse_string(&mut token_list, line_number, trimmed_text, column_idx_begin);
            }
            else {
                continue;
            }
        }
    }

    Ok(token_list)
}

fn parse_string(
    token_list: &mut Vec<Spanned<Token>>,
    line_number: usize,
    text: &str,
    span_offset: usize,
)
{
    let mut buffer: Vec<u8> = Vec::new();
    let text = text.as_bytes();
    let mut idx = 0;

    while idx < text.len() {
        let iter_start_idx = idx;

        if text[idx].is_ascii_digit() {
            // Collect the characters until its not a number anymore
            while (idx < text.len()) && text[idx].is_ascii_digit() {
                buffer.push(text[idx]);
                idx += 1;
            }

            // Store the number we have parsed
            token_list.push(Spanned::new(
                // Empty the buffer when making the literal
                Token::UnparsedLiteral(String::from_utf8(std::mem::take(&mut buffer)).unwrap()),
                SpanInfo::new(
                    CharPosition::new(line_number, span_offset + iter_start_idx),
                    CharPosition::new(line_number, span_offset + idx),
                ),
            ));
        }
        /*
            NOTICE:
            THIS TYPE OF TOKEN MATCHING LIMITS THE SYNTAX OF TOKENS:
            If we want to be able to parse >>= both > and >> have to be a valid token.
            This part of the code is basically limited to parsing special expressions.
            If I were to try to tokenize `helloint` the identifier branch would parse int with hello.
            This branch is made to parse `a*f` or `foo==bar`.
        */
        else if try_match_token(&[text[idx]]).is_some() {
            // Try to greedily consume the longest matching token
            let mut match_end = idx + 1;

            while match_end < text.len() {
                if try_match_token(&text[idx..=match_end]).is_some() {
                    match_end += 1;
                } else {
                    break;
                }
            }

            // Walk back to the last valid match
            while match_end > idx {
                if let Some(matched) = try_match_token(&text[idx..match_end]) {
                    token_list.push(Spanned::new(
                        matched,
                        SpanInfo::new(
                            CharPosition::new(line_number, span_offset + idx),
                            CharPosition::new(line_number, span_offset + match_end),
                        ),
                    ));
                    idx = match_end;
                    break;
                }
                match_end -= 1;
            }
        }
        // If its not a number and was not matched by the keywords this should be an identifier
        else {
            // Store the chars until we can match a char
            while (idx < text.len()) && let None = try_match_token(&[text[idx]])
            {
                buffer.push(text[idx]);
                idx += 1;
            }

            // Store the identifier
            token_list.push(Spanned::new(
                // Empty the buffer when creating the identifier
                Token::Identifier(String::from_utf8(std::mem::take(&mut buffer)).unwrap()),
                SpanInfo::new(
                    CharPosition::new(line_number, span_offset + iter_start_idx),
                    CharPosition::new(line_number, span_offset + idx),
                ),
            ));
        }
    }
}

fn try_match_token(string_to_match: &[u8]) -> Option<Token>
{
    Some(match string_to_match {
        b"+" => Token::Addition,
        b"-" => Token::Subtraction,
        b"*" => Token::Multiplication,
        b"/" => Token::Division,
        b"%" => Token::Modulo,

        b"}" => Token::CloseBraces,
        b">" => Token::CloseAngledBrackets,
        b")" => Token::CloseParentheses,
        b"]" => Token::CloseSquareBrackets,

        b"{" => Token::OpenBraces,
        b"<" => Token::OpenAngledBrackets,
        b"(" => Token::OpenParentheses,
        b"[" => Token::OpenSquareBrackets,

        b"int" => Token::TypeDefinition(TypeToken::I32),
        b"uint" => Token::TypeDefinition(TypeToken::U32),
        b"float" => Token::TypeDefinition(TypeToken::F32),
        b"inthalf" => Token::TypeDefinition(TypeToken::I16),
        b"uinthalf" => Token::TypeDefinition(TypeToken::U16),
        b"floathalf" => Token::TypeDefinition(TypeToken::F16),
        b"intlong" => Token::TypeDefinition(TypeToken::I64),
        b"uintlong" => Token::TypeDefinition(TypeToken::U64),
        b"floatlong" => Token::TypeDefinition(TypeToken::F64),
        b"uintsmall" => Token::TypeDefinition(TypeToken::U8),
        b"bool" => Token::TypeDefinition(TypeToken::Boolean),
        b"void" => Token::TypeDefinition(TypeToken::Void),
        b"string" => Token::TypeDefinition(TypeToken::String),
        b"array" => Token::TypeDefinition(TypeToken::Array),
        b"enum" => Token::TypeDefinition(TypeToken::Enum),
        b"ref" => Token::TypeDefinition(TypeToken::Reference),
        b"deref" => Token::TypeDefinition(TypeToken::Dereference),
        b"struct" => Token::TypeDefinition(TypeToken::Struct),

        b"==" => Token::Equal,
        b"&&" => Token::And,
        b"||" => Token::Or,
        b"=+" => Token::SetValueAddition,
        b"=-" => Token::SetValueSubtraction,
        b"=*" => Token::SetValueMultiplication,
        b"=/" => Token::SetValueDivision,
        b"%=" => Token::SetValueModulo,
        b"false" => Token::Literal(Value::Boolean(false)),
        b"true" => Token::Literal(Value::Boolean(true)),
        b"external" => Token::External,
        b"import" => Token::Import,
        b"function" => Token::Function,
        b"return" => Token::Return,
        b"as" => Token::As,
        b"if" => Token::If,
        b"else" => Token::Else,
        b"elseif" => Token::ElseIf,
        b"loop" => Token::Loop,
        b"for" => Token::For,
        b"break" => Token::Break,
        b"continue" => Token::Continue,
        b"priv" => Token::Private,
        b"pub" => Token::Public,
        b"publib" => Token::PublicLibrary,
        b"exp" => Token::Export,
        b"cold" => Token::CompilerHint(CompilerHint::Cold),
        b"nofree" => Token::CompilerHint(CompilerHint::NoFree),
        b"nounwind" => Token::CompilerHint(CompilerHint::NoUnWind),
        b"inline" => Token::CompilerHint(CompilerHint::Inline),
        b"feature" => Token::CompilerHint(CompilerHint::Feature),
        b"." => Token::Dot,
        b":" => Token::Colon,
        b"::" => Token::DoubleColon,
        b"<-" => Token::LeftArrow,
        b"->" => Token::RightArrow,
        b"=" => Token::SetValue,
        b">>" => Token::BitRight,
        b"<<" => Token::BitLeft,
        b"|" => Token::BitOr,
        b"&" => Token::BitAnd,
        b"@" => Token::CompilerHintSymbol,
        b";" => Token::SemiColon,
        b"const" => Token::Const,
        b"var" => Token::Variable,
        _ => return None,
    })
}
