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

            let token_span_info = SpanInfo::new(
                CharPosition::new(line_number, column_idx_begin),
                CharPosition::new(line_number, column_idx_end),
            );

            if let Some(tkn) = try_match_token(trimmed_text.as_bytes()) {
                token_list.push(Spanned::new(tkn, token_span_info));
            }
            // If the token was not immediately recognized it is most likely number or an Identifier
            // If the trimmed text could also be an empty string.
            else if !trimmed_text.is_empty() {
                // `trimmed_text` cannot be empty here, so we can actually check the first charater
                // if the trimmed text is a number, then this is supposed to be some sort of a number
                // We will only check the validness of the number later on in the resolver process.
                if trimmed_text.starts_with(char::is_numeric) {
                    // Lets try to find the end of the number (We store every digit and . and split with the rest)
                    let num_end_pos = trimmed_text.find(|c: char| !(c.is_numeric() || c == '.'));

                    if let Some(pos) = num_end_pos {
                        // Split the string at the desired position
                        let (number, expressions) = trimmed_text.split_at(pos);

                        // Store the number
                        token_list.push(Spanned::new(
                            Token::UnparsedLiteral(number.to_string()),
                            SpanInfo::new(
                                CharPosition::new(line_number, column_idx_begin),
                                CharPosition::new(line_number, column_idx_begin + pos),
                            ),
                        ));

                        /*
                            What I was thinking of doing is basically parsing the characters until there is no match to a token (/ keyword)
                            So in this case: ==*
                            I'd parse the first character, (this will parse as a setvalue sign) after seeing that this succeeded parse the second with it (this will return an equal sign)
                            When parsing the 3rd char with the last two it will not return any valid token. Store the last successful token parsing result, and continue this process from the 3rd char.
                        */

                        // Variables to keep track of stuff
                        let mut char_buf = String::new();
                        let mut last_token: Option<Token> = None;
                        let mut last_stored_token_idx = 0;
                        let mut last_idx = 0;

                        // Parse the tokens after the number
                        for c in expressions.char_indices() {
                            try_match_char(
                                &mut token_list,
                                &mut char_buf,
                                &mut last_token,
                                &mut last_stored_token_idx,
                                c,
                                line_number,
                            );

                            last_idx = c.0;
                        }

                        // If there were any tokens we didnt store (for example when the last character matched too) we need to store that too.
                        if let Some(last_matched) = last_token {
                            token_list.push(Spanned::new(last_matched, SpanInfo::new(CharPosition::new(line_number, last_stored_token_idx), CharPosition::new(line_number, last_idx))));
                        }
                    }
                    // This means that only a number was present in the `trimmed_text`
                    else {
                        token_list.push(Spanned::new(
                            Token::UnparsedLiteral(trimmed_text.to_string()),
                            token_span_info,
                        ));
                    }
                }
            }
            else {
                continue;
            }
        }
    }

    Ok(token_list)
}

fn try_match_char(
    token_list: &mut Vec<Spanned<Token>>,
    char_buf: &mut String,
    last_matched_token: &mut Option<Token>,
    last_stored_token_idx: &mut usize,
    // The current char indice
    (idx, c): (usize, char),
    line_number: usize,
)
{
    // Store the character into a buffer
    char_buf.push(c);

    // Try to match the current contents of the char buffer
    // If it succeeds then store the next character until it doesnt.
    if let Some(matched) = try_match_token(char_buf.as_bytes()) {
        *last_matched_token = Some(matched);

        // Try to store the next character on the next iter and try to match again
        return;
    }
    // If it didnt match with anything, store the last match and continue parsing from the last character
    else if let Some(last_matched) = &last_matched_token {
        // Reset the buffer
        *char_buf = String::new();

        // Store the last match
        token_list.push(Spanned::new(
            last_matched.clone(),
            SpanInfo::new(
                CharPosition::new(line_number, *last_stored_token_idx),
                CharPosition::new(line_number, idx - 1),
            ),
        ));

        // Update the last matched idx
        *last_stored_token_idx = idx;

        // Restart the process
        try_match_char(
            token_list,
            char_buf,
            last_matched_token,
            last_stored_token_idx,
            (idx, c),
            line_number,
        );
    }
    else {
        panic!("Return an error here")
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
