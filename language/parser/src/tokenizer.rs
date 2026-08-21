use common::{
    anyhow,
    error::{CharPosition, SpanInfo, Spanned},
    parser::function::CompilerInstructionDiscriminants,
    tokenizer::{Token, TypeToken},
    ty::Value,
};
use std::u8;

pub fn tokenize(input: &str) -> anyhow::Result<Vec<Spanned<Token>>>
{
    let mut token_list: Vec<Spanned<Token>> = Vec::new();

    let mut capture_string: Option<CaptureString> = None;

    'line_loop: for (line_idx, line) in input.lines().enumerate() {
        let line_number = line_idx + 1;
        let mut column_idx = 0;

        for raw_text in line.split_inclusive(char::is_whitespace) {
            let trimmed_text = raw_text.trim();

            let column_idx_begin = column_idx;

            // Increment column idx by the text length (This includes whitespace)
            column_idx += raw_text.len();

            // Check if we are capturing a string
            // Capture the string automatically
            if let Some(capture) = &mut capture_string {
                // Try to find the end of the string
                if let Some(quote_idx) = raw_text.find('"') {
                    // Split at the quote end
                    let (string_p, other) = raw_text.split_at(quote_idx);

                    // Store the string which is a part of the full string
                    capture.string_buffer.extend(string_p.as_bytes());

                    // Store the captured string
                    token_list.push(Spanned::new(
                        Token::Literal(Value::String(
                            String::from_utf8(capture.string_buffer.clone()).unwrap(),
                        )),
                        SpanInfo::new(
                            capture.span_start,
                            CharPosition::new(line_number, column_idx_begin + quote_idx),
                        ),
                    ));

                    // Parse the rest of the text
                    parse_single_text(
                        &mut token_list,
                        line_number,
                        other.trim(),
                        column_idx_begin,
                        &mut capture_string,
                    );

                    // Reset the capture state
                    capture_string = None;
                }
                // If the quote isnt present in the text that means that its just word in the string.
                else {
                    capture.string_buffer.extend(raw_text.as_bytes());
                }
            }
            else if trimmed_text.starts_with('#') {
                // If its a comment just skip the whole line / the rest of the line
                continue 'line_loop;
            }
            // Parse the text
            // Please note that we always pass one word (text between two whitespaces) to this function.
            // If the trimmed text could also be an empty string.
            else if !trimmed_text.is_empty() {
                parse_single_text(
                    &mut token_list,
                    line_number,
                    trimmed_text,
                    column_idx_begin,
                    &mut capture_string,
                );
            }
            else {
                continue;
            }
        }
    }

    Ok(token_list)
}

fn parse_single_text(
    token_list: &mut Vec<Spanned<Token>>,
    line_number: usize,
    text: &str,
    span_offset: usize,
    capture_string: &mut Option<CaptureString>,
)
{
    let mut buffer: Vec<u8> = Vec::new();
    let text = text.as_bytes();
    let mut idx = 0;

    while idx < text.len() {
        let iter_start_idx = idx;

        if text[idx].is_ascii_digit() {
            // Collect the integer part
            while idx < text.len() && text[idx].is_ascii_digit() {
                buffer.push(text[idx]);
                idx += 1;
            }

            // Check for a decimal point followed by more digits (or just a trailing dot)
            // e.g. "3.14" or "343."
            // We do NOT consume if it looks like "355.3.asd" — we only take the first decimal
            if idx < text.len() && text[idx] == b'.' {
                // Peek ahead — only consume the dot if what follows is a digit OR end of number
                // "343." is valid, "343.asd" is NOT a float (dot belongs to chain)
                let after_dot = idx + 1;
                let next_is_digit_or_end = after_dot >= text.len()
                    || text[after_dot].is_ascii_digit()
                    || !text[after_dot].is_ascii_alphanumeric();

                if next_is_digit_or_end {
                    buffer.push(b'.');
                    idx += 1; // consume the dot

                    // consume fractional digits
                    while idx < text.len() && text[idx].is_ascii_digit() {
                        buffer.push(text[idx]);
                        idx += 1;
                    }
                }
            }

            token_list.push(Spanned::new(
                Token::UnparsedLiteral(String::from_utf8(std::mem::take(&mut buffer)).unwrap()),
                create_span_info(line_number, span_offset, iter_start_idx, idx),
            ));
        }
        else if let Some(tkn) = try_match_token(text[idx..].trim_ascii()) {
            token_list.push(Spanned::new(
                tkn,
                create_span_info(
                    line_number,
                    span_offset,
                    idx,
                    idx + text[idx..].trim_ascii().len(),
                ),
            ));

            return;
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
                }
                else {
                    break;
                }
            }

            // Walk back to the last valid match
            while match_end > idx {
                if let Some(matched) = try_match_token(&text[idx..match_end]) {
                    token_list.push(Spanned::new(
                        matched,
                        create_span_info(line_number, span_offset, idx, match_end),
                    ));
                    idx = match_end;
                    break;
                }
                match_end -= 1;
            }
        }
        else if text[idx] == b'"' {
            let mut string_buffer = Vec::new();
            let idx_start = idx;
            let mut quote_present = false;

            // Move the cursor to the first letter of the string
            idx += 1;

            for c in &text[idx..] {
                idx += 1;

                if *c == b'"' {
                    quote_present = true;
                    break;
                }

                string_buffer.push(*c);
            }

            // If the quote was present that means that the string didnt have any spaces.
            if quote_present {
                token_list.push(Spanned::new(
                    Token::Literal(Value::String(String::from_utf8(string_buffer).unwrap())),
                    create_span_info(line_number, span_offset, idx_start, idx),
                ));
            }
            // If the quote was not present, that means that the string consists of multiple words.
            // We have to set the state of `capture_string` to capture the next words.
            else {
                *capture_string = Some(CaptureString {
                    span_start: CharPosition::new(line_number, span_offset + idx_start),
                    string_buffer,
                });
            }
        }
        // If its not a number and was not matched by the keywords this should be an identifier
        else {
            // Store the chars until we can match a char
            while (idx < text.len())
                && let None = try_match_token(&[text[idx]])
            {
                buffer.push(text[idx]);
                idx += 1;
            }

            // Empty the buffer when trying to create the identifier
            let word = String::from_utf8(std::mem::take(&mut buffer)).unwrap();

            // If the ident can be parsed as a token then do so, if that fails fall back to using it as an ident
            let token = try_match_token(word.as_bytes()).unwrap_or_else(|| Token::Identifier(word));

            // Store the identifier
            token_list.push(Spanned::new(
                token,
                create_span_info(line_number, span_offset, iter_start_idx, idx),
            ));
        }
    }
}

/// This assumes that the Span we are trying to create is in one line.
fn create_span_info(line: usize, offset: usize, start: usize, end: usize) -> SpanInfo
{
    SpanInfo {
        char_start: CharPosition {
            line,
            column: offset + start,
        },
        char_end: CharPosition {
            line,
            column: offset + end,
        },
    }
}

fn try_match_token(string_to_match: &[u8]) -> Option<Token>
{
    Some(match string_to_match {
        b"+" => Token::MathSym(common::parser::numeric_value::MathematicalSymbol::Addition),
        b"-" => Token::MathSym(common::parser::numeric_value::MathematicalSymbol::Subtraction),
        b"*" => Token::MathSym(common::parser::numeric_value::MathematicalSymbol::Multiplication),
        b"/" => Token::MathSym(common::parser::numeric_value::MathematicalSymbol::Division),
        b"%" => Token::MathSym(common::parser::numeric_value::MathematicalSymbol::Modulo),

        b"}" => Token::CloseBraces,
        b">" => Token::CloseAngledBrackets,
        b")" => Token::CloseParentheses,
        b"]" => Token::CloseSquareBrackets,

        b"{" => Token::OpenBraces,
        b"<" => Token::OpenAngledBrackets,
        b"(" => Token::OpenParentheses,
        b"[" => Token::OpenSquareBrackets,

        b"," => Token::Comma,
        b"." => Token::Dot,
        b":" => Token::Colon,

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
        b"struct" => Token::TypeDefinition(TypeToken::Struct),
        b"enum" => Token::TypeDefinition(TypeToken::Enum),
        
        b"ptr" => Token::TypeDefinition(TypeToken::Pointer),
        b"ref" => Token::Reference,
        b"deref" => Token::Dereference,


        b"==" => Token::Equal,
        b"&&" => Token::And,
        b"||" => Token::Or,

        b"=+" => {
            Token::SetValueMathSym(common::parser::numeric_value::MathematicalSymbol::Addition)
        },
        b"=-" => {
            Token::SetValueMathSym(common::parser::numeric_value::MathematicalSymbol::Subtraction)
        },
        b"=*" => {
            Token::SetValueMathSym(
                common::parser::numeric_value::MathematicalSymbol::Multiplication,
            )
        },
        b"=/" => {
            Token::SetValueMathSym(common::parser::numeric_value::MathematicalSymbol::Division)
        },
        b"%=" => Token::SetValueMathSym(common::parser::numeric_value::MathematicalSymbol::Modulo),

        b"false" => Token::Literal(Value::Boolean(false)),
        b"true" => Token::Literal(Value::Boolean(true)),
        b"external" => Token::External,
        b"import" => Token::Import,
        b"function" => Token::TypeDefinition(TypeToken::Function),
        b"return" => Token::Return,
        b"as" => Token::As,
        b"if" => Token::If,
        b"else" => Token::Else,
        b"elseif" => Token::ElseIf,
        b"loop" => Token::Loop,
        b"for" => Token::For,
        b"while" => Token::While,
        b"break" => Token::Break,
        b"continue" => Token::Continue,
        b"priv" => Token::ItemVisibility(common::parser::common::ItemVisibility::Private),
        b"pub" => Token::ItemVisibility(common::parser::common::ItemVisibility::Public),
        b"publib" => Token::ItemVisibility(common::parser::common::ItemVisibility::PublicLibrary),
        b"exp" => Token::Export,
        b"cold" => Token::CompilerInstruction(CompilerInstructionDiscriminants::Cold),
        b"nofree" => Token::CompilerInstruction(CompilerInstructionDiscriminants::NoFree),
        b"nounwind" => Token::CompilerInstruction(CompilerInstructionDiscriminants::NoUnWind),
        b"inline" => Token::CompilerInstruction(CompilerInstructionDiscriminants::Inline),
        b"feature" => Token::CompilerInstruction(CompilerInstructionDiscriminants::Feature),

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
        b"namespace" => Token::Namespace,
        _ => return None,
    })
}

struct CaptureString
{
    span_start: CharPosition,
    string_buffer: Vec<u8>,
}
