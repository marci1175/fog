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
    let mut in_multiline_comment = false;
    let mut capture_string: Option<CaptureString> = None;

    'line_loop: for (line_idx, mut line) in input.lines().enumerate() {
        let line_number = line_idx + 1;
        let mut column_idx = 0;

        if in_multiline_comment {
            let closing_tkn = "<-#";

            // Find the indicator for closing the multiline comment
            if let Some(multiline_comm_end) = line.find(closing_tkn) {
                // Restore the line we are working with.
                let line_start = multiline_comm_end + closing_tkn.len();
                line = &line[line_start..];

                // Set the column index to the correct position
                column_idx = line_start;

                // Set the multiline comment to false
                in_multiline_comment = false;
            }
        }

        // Skip the line if we are in a multiline comment
        if in_multiline_comment {
            continue 'line_loop;
        }

        // We only check if there is a multiline opening token after the actual line skipping check, so that the remainder of this line will still get parsed
        if let Some(multiline_comm_start) = line.find("#->") {
            // Restore the the line before the multiline comment's start
            line = &line[..multiline_comm_start];

            // Set the state to true
            in_multiline_comment = true;
        }

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
                    let string_p = &raw_text[..quote_idx];
                    let other = &raw_text[quote_idx + 1..];

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
    raw_text: &str,
    span_offset: usize,
    capture_string: &mut Option<CaptureString>,
)
{
    let mut buffer: Vec<u8> = Vec::new();
    let text = raw_text.as_bytes();
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
            // The guard above confirms text[idx] alone is a valid 1-byte token,
            // so this is always guaranteed to match at least length 1.
            // match_longest_token finds the longest one instead (e.g. preferring
            // "..." over ".").
            let (matched, match_len) = match_longest_token(&text[idx..])
                .expect("guarded above: text[idx] alone is always a valid 1-byte token");

            token_list.push(Spanned::new(
                matched,
                create_span_info(line_number, span_offset, idx, idx + match_len),
            ));

            idx += match_len;
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

/// Longest possible length (in bytes) of any multi-character symbolic token
const MAX_SYMBOL_TOKEN_LEN: usize = 3;

fn match_longest_token(bytes: &[u8]) -> Option<(Token, usize)>
{
    let max_len = bytes.len().min(MAX_SYMBOL_TOKEN_LEN);

    (1..=max_len)
        .rev()
        .find_map(|len| try_match_token(&bytes[..len]).map(|tok| (tok, len)))
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
        b"..." => Token::Ellipsis,
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

        b"==" => Token::Comparison(common::codegen::Order::Equal),
        b"!=" => Token::Comparison(common::codegen::Order::NotEqual),
        b">=" => Token::Comparison(common::codegen::Order::EqBigger),
        b"<=" => Token::Comparison(common::codegen::Order::EqSmaller),
        // b">" => Token::Comparison(common::codegen::Order::Bigger),
        // b"<" => Token::Comparison(common::codegen::Order::Smaller),

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
        b"b>>" => Token::BitRight,
        b"b<<" => Token::BitLeft,
        b"|" => Token::BitOr,
        b"&" => Token::BitAnd,
        b"@" => Token::CompilerHintSymbol,
        b";" => Token::SemiColon,

        b"const" => Token::Const,
        b"static" => Token::Static,

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
