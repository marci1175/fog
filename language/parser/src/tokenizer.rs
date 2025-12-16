use std::{ops::Range, u8, u16, u64};

use common::{
    anyhow,
    error::{CharPosition, DebugInformation, parser::ParserError, syntax::SyntaxError},
    tokenizer::{Token, find_closing_angled_bracket_char},
    ty::{Type, TypeDiscriminant},
};

pub fn only_contains_digits(s: &[u8]) -> bool
{
    s.iter().all(|c| c.is_ascii_digit())
}

const DOUBLE_BACKSLASH_U8: u8 = b'\\';
const NEWLINE_CHAR_U8: u8 = b'\n';
const ENDLINE_CHAR_U8: u8 = b'\r';

pub fn tokenize(
    raw_input: &str,
    stop_at_token: Option<Token>,
) -> anyhow::Result<(Vec<Token>, Vec<DebugInformation>, usize)>
{
    let mut dest_num_type: Option<TypeDiscriminant> = None;
    let mut char_idx: usize = 0;

    let char_list = raw_input.as_bytes();
    let mut token_list: Vec<Token> = Vec::with_capacity(char_list.len() / 4);
    let mut token_debug_info: Vec<DebugInformation> = Vec::with_capacity(char_list.len() / 4);

    let mut string_buffer = Vec::new();

    let mut line_counter = 0;
    let mut line_char_idx = 0;

    while char_idx < char_list.len() {
        if let Some(stop_at_token) = &stop_at_token
            && token_list
                .last()
                .is_some_and(|last_token| last_token == stop_at_token)
        {
            return Ok((token_list, token_debug_info, char_idx));
        }

        let current_char = char_list[char_idx];

        let single_char = match current_char {
            b'+' => Some(Token::Addition),
            b'*' => Some(Token::Multiplication),
            b'/' => Some(Token::Division),
            b')' => Some(Token::CloseParentheses),
            b'(' => Some(Token::OpenParentheses),
            b'}' => Some(Token::CloseBraces),
            b'{' => Some(Token::OpenBraces),
            b'[' => Some(Token::OpenSquareBrackets),
            b']' => Some(Token::CloseSquareBrackets),
            b';' => Some(Token::SemiColon),
            b',' => Some(Token::Comma),
            b'%' => Some(Token::Modulo),
            b'@' => Some(Token::CompilerHintSymbol),
            b'$' => Some(Token::Pointer),
            _ => None,
        };

        let current_char_idx_in_line = char_idx - line_char_idx;

        if let Some(single_char_token) = single_char {
            if !string_buffer.is_empty() {
                let token = match_multi_character_expression(&string_buffer)?;

                token_list.push(token);
                token_debug_info.push(DebugInformation {
                    char_start: CharPosition {
                        line: line_counter,
                        column: current_char_idx_in_line - string_buffer.len(),
                    },
                    char_end: CharPosition {
                        line: line_counter,
                        column: current_char_idx_in_line,
                    },
                });
            }

            token_list.push(single_char_token);
            token_debug_info.push(DebugInformation {
                char_start: CharPosition {
                    line: line_counter,
                    column: current_char_idx_in_line,
                },
                char_end: CharPosition {
                    line: line_counter,
                    column: current_char_idx_in_line + 1,
                },
            });

            string_buffer.clear();

            char_idx += 1;

            continue;
        }
        if string_buffer == b"array" {
            if current_char == b'<' {
                char_idx += 1;

                let closing_idx = find_closing_angled_bracket_char(&char_list[char_idx..], 0)?;

                let list_type = &char_list[char_idx..closing_idx + char_idx];

                let comma_pos = list_type.len()
                    - list_type
                        .iter()
                        .rev()
                        .position(|char| *char == b',')
                        .ok_or(ParserError::SyntaxError(
                            SyntaxError::MissingCommaAtArrayDef,
                        ))?;

                let array_len = String::from_utf8_lossy(&list_type[comma_pos..]);
                let list_type_def = String::from_utf8_lossy(&list_type[..comma_pos - 1]);

                let (inner_token, _, _) = tokenize(&list_type_def, None)?;

                if inner_token.len() > 1 {
                    return Err(ParserError::InvalidArrayTypeDefinition(inner_token).into());
                }

                token_list.push(Token::TypeDefinition(TypeDiscriminant::Array((
                    Box::new(inner_token[0].clone()),
                    array_len.trim().parse::<usize>().map_err(|_| {
                        ParserError::SyntaxError(SyntaxError::UnparsableExpression(
                            array_len.clone().to_string(),
                        ))
                    })?,
                ))));
                token_debug_info.push(DebugInformation {
                    char_start: CharPosition::new(line_counter, current_char_idx_in_line),
                    char_end: CharPosition::new(
                        line_counter,
                        current_char_idx_in_line + closing_idx,
                    ),
                });

                string_buffer.clear();

                char_idx += closing_idx + 1;
                continue;
            }
        }
        if (current_char == b' ' || current_char as u8 == NEWLINE_CHAR_U8)
            && !string_buffer.is_empty()
        {
            let token = match_multi_character_expression(&string_buffer)?;

            token_list.push(token);
            token_debug_info.push(DebugInformation {
                char_start: CharPosition {
                    line: line_counter,
                    column: current_char_idx_in_line - string_buffer.len(),
                },
                char_end: CharPosition {
                    line: line_counter,
                    column: current_char_idx_in_line,
                },
            });

            string_buffer.clear();
        }
        else if string_buffer.len() + 1 == char_list.len() {
            string_buffer.push(char_list[char_list.len() - 1]);

            let token = match_multi_character_expression(&string_buffer)?;

            token_list.push(token);
            token_debug_info.push(DebugInformation {
                char_start: CharPosition {
                    line: line_counter,
                    column: current_char_idx_in_line - (string_buffer.len() - 1),
                },
                char_end: CharPosition {
                    line: line_counter,
                    column: current_char_idx_in_line,
                },
            });

            string_buffer.clear();
        }

        match current_char {
            b'.' => {
                let next_char = char_list.get(char_idx + 1);
                let next_char_2 = char_list.get(char_idx + 2);

                let dot = '.' as u8;
                if Some(&dot) == next_char_2 && Some(&dot) == next_char {
                    token_list.push(Token::Ellipsis);
                    token_debug_info.push(DebugInformation {
                        char_start: CharPosition {
                            line: line_counter,
                            column: current_char_idx_in_line,
                        },
                        char_end: CharPosition {
                            line: line_counter,
                            column: current_char_idx_in_line + 3,
                        },
                    });

                    char_idx += 3;

                    continue;
                }
                else if only_contains_digits(&string_buffer) && !string_buffer.is_empty()
                /* This might break ellpisis parsing */
                {
                    string_buffer.push(current_char);
                }
                else {
                    if !string_buffer.is_empty() {
                        // Push the chars we have collected as an ident
                        let token = match_multi_character_expression(&string_buffer)?;

                        token_list.push(token);
                        token_debug_info.push(DebugInformation {
                            char_start: CharPosition {
                                line: line_counter,
                                column: current_char_idx_in_line - string_buffer.len(),
                            },
                            char_end: CharPosition {
                                line: line_counter,
                                column: current_char_idx_in_line,
                            },
                        });

                        string_buffer.clear();
                    }

                    token_list.push(Token::Dot);
                    token_debug_info.push(DebugInformation {
                        char_start: CharPosition {
                            line: line_counter,
                            column: current_char_idx_in_line,
                        },
                        char_end: CharPosition {
                            line: line_counter,
                            column: current_char_idx_in_line + 1,
                        },
                    });
                }
            },
            b'-' => {
                let last_token = &token_list[token_list.len() - 1];

                // If the last token was a number we know that we are subtracting
                if (matches!(last_token, Token::Literal(_))
                    || matches!(last_token, Token::UnparsedLiteral(_))
                    || matches!(last_token, Token::Identifier(_))
                    || matches!(last_token, Token::CloseParentheses))
                {
                    token_list.push(Token::Subtraction);
                    token_debug_info.push(DebugInformation {
                        char_start: CharPosition {
                            line: line_counter,
                            column: current_char_idx_in_line,
                        },
                        char_end: CharPosition {
                            line: line_counter,
                            column: current_char_idx_in_line + 1,
                        },
                    });
                }
                // If the last token wasnt a number we know that we are defining a negative number
                else {
                    string_buffer.push(current_char);
                }
            },
            b'#' => {
                if let Some(char) = char_list.get(char_idx + 1) {
                    if *char == b'-' && char_list.get(char_idx + 2) == Some(&('>' as u8)) {
                        char_idx += 3;

                        let slice = raw_input[char_idx..].as_bytes();
                        let sig_bitmask = u32::from_le_bytes(*b"#->\0");

                        let mut i = 0;

                        while i + 2 < slice.len() {
                            let window_bitmask =
                                u32::from_le_bytes([slice[i], slice[i + 1], slice[i + 2], 0]);

                            if sig_bitmask == window_bitmask {
                                break;
                            }
                            else {
                                i += 1;
                            }
                        }

                        char_idx += i + 1;

                        continue;
                    }

                    let original_char_idx = char_idx;

                    let hastag = '#' as u8;
                    if *char == hastag {
                        if char_list.get(char_idx + 2) == Some(&hastag) {
                            char_idx += 3;

                            loop {
                                let quote_char = char_list[char_idx + 1];

                                if quote_char == NEWLINE_CHAR_U8 {
                                    token_list.push(Token::DocComment(
                                        String::from_utf8_lossy(
                                            &char_list[original_char_idx..char_idx],
                                        )
                                        .to_string(),
                                    ));

                                    token_debug_info.push(DebugInformation {
                                        char_start: CharPosition {
                                            line: original_char_idx,
                                            column: original_char_idx,
                                        },
                                        char_end: CharPosition {
                                            line: line_counter,
                                            column: current_char_idx_in_line,
                                        },
                                    });

                                    char_idx += 2;

                                    break;
                                }

                                char_idx += 1;
                            }

                            continue;
                        }
                    }
                    else {
                        // Parse until nextline char, because then we know that the comment has ended.
                        // We wont store the comment
                        loop {
                            let quote_char = char_list[char_idx + 1];

                            if quote_char == ENDLINE_CHAR_U8 {
                                char_idx += 2;
                                break;
                            }

                            char_idx += 1;
                        }

                        continue;
                    }
                }
            },
            b'"' => {
                let mut quotes_buffer: Vec<u8> = Vec::new();

                let mut quote_idx = char_idx + 1;

                loop {
                    let quote_char = char_list.get(quote_idx);

                    match quote_char {
                        Some(quote_char) => {
                            if *quote_char == DOUBLE_BACKSLASH_U8 {
                                match char_list.get(quote_idx + 1) {
                                    Some(b'n') => {
                                        quotes_buffer.push(NEWLINE_CHAR_U8);

                                        quote_idx += 2;

                                        continue;
                                    },
                                    Some(b'r') => {
                                        quotes_buffer.push(ENDLINE_CHAR_U8);

                                        quote_idx += 2;

                                        continue;
                                    },
                                    Some(b't') => {
                                        quotes_buffer.push(b'\t');

                                        quote_idx += 2;

                                        continue;
                                    },
                                    Some(b'0') => {
                                        quotes_buffer.push(b'\0');

                                        quote_idx += 2;

                                        continue;
                                    },
                                    Some(&DOUBLE_BACKSLASH_U8) => {
                                        quotes_buffer.push(DOUBLE_BACKSLASH_U8);
                                        quote_idx += 2;

                                        continue;
                                    },
                                    Some(char) => {
                                        quotes_buffer.push(DOUBLE_BACKSLASH_U8 as u8);
                                        quotes_buffer.push(*char);

                                        quote_idx += 2;

                                        continue;
                                    },

                                    None => {},
                                }
                            }

                            if *quote_char == b'"' {
                                token_list.push(Token::Literal(Type::String(
                                    String::from_utf8(quotes_buffer)
                                        .map_err(|_| ParserError::InvalidUtf8Literal)?,
                                )));
                                token_debug_info.push(DebugInformation {
                                    char_start: CharPosition {
                                        line: line_counter,
                                        column: char_idx - line_char_idx,
                                    },
                                    char_end: CharPosition {
                                        line: line_counter,
                                        column: quote_idx + 1 - line_char_idx,
                                    },
                                });

                                char_idx = quote_idx + 1;

                                break;
                            }

                            quotes_buffer.push(*quote_char);

                            quote_idx += 1;
                        },
                        // If there are no more tokens left and we are still in the quote
                        None => {
                            return Err(ParserError::SyntaxError(SyntaxError::OpenQuotes).into());
                        },
                    }
                }

                continue;
            },
            b'=' => {
                if let Some(next_char) = char_list.get(char_idx + 1) {
                    match *next_char {
                        b'=' => {
                            token_list.push(Token::Equal);
                            token_debug_info.push(DebugInformation {
                                char_start: CharPosition {
                                    line: line_counter,
                                    column: current_char_idx_in_line,
                                },
                                char_end: CharPosition {
                                    line: line_counter,
                                    column: current_char_idx_in_line + 2,
                                },
                            });

                            char_idx += 2;
                        },
                        b'+' => {
                            token_list.push(Token::SetValueAddition);
                            token_debug_info.push(DebugInformation {
                                char_start: CharPosition {
                                    line: line_counter,
                                    column: current_char_idx_in_line,
                                },
                                char_end: CharPosition {
                                    line: line_counter,
                                    column: current_char_idx_in_line + 2,
                                },
                            });

                            char_idx += 2;
                        },
                        b'-' => {
                            token_list.push(Token::SetValueSubtraction);
                            token_debug_info.push(DebugInformation {
                                char_start: CharPosition {
                                    line: line_counter,
                                    column: current_char_idx_in_line,
                                },
                                char_end: CharPosition {
                                    line: line_counter,
                                    column: current_char_idx_in_line + 2,
                                },
                            });

                            char_idx += 2;
                        },
                        b'*' => {
                            token_list.push(Token::SetValueMultiplication);
                            token_debug_info.push(DebugInformation {
                                char_start: CharPosition {
                                    line: line_counter,
                                    column: current_char_idx_in_line,
                                },
                                char_end: CharPosition {
                                    line: line_counter,
                                    column: current_char_idx_in_line + 2,
                                },
                            });

                            char_idx += 2;
                        },
                        b'/' => {
                            token_list.push(Token::SetValueDivision);
                            token_debug_info.push(DebugInformation {
                                char_start: CharPosition {
                                    line: line_counter,
                                    column: current_char_idx_in_line,
                                },
                                char_end: CharPosition {
                                    line: line_counter,
                                    column: current_char_idx_in_line + 2,
                                },
                            });

                            char_idx += 2;
                        },
                        b'%' => {
                            token_list.push(Token::SetValueModulo);
                            token_debug_info.push(DebugInformation {
                                char_start: CharPosition {
                                    line: line_counter,
                                    column: current_char_idx_in_line,
                                },
                                char_end: CharPosition {
                                    line: line_counter,
                                    column: current_char_idx_in_line + 2,
                                },
                            });

                            char_idx += 2;
                        },

                        _ => {
                            token_list.push(Token::SetValue);
                            token_debug_info.push(DebugInformation {
                                char_start: CharPosition {
                                    line: line_counter,
                                    column: current_char_idx_in_line,
                                },
                                char_end: CharPosition {
                                    line: line_counter,
                                    column: current_char_idx_in_line + 2,
                                },
                            });

                            char_idx += 1;
                        },
                    }

                    continue;
                }
            },
            b':' => {
                if !string_buffer.is_empty() {
                    let token = match_multi_character_expression(&string_buffer)?;

                    token_list.push(token);
                    token_debug_info.push(DebugInformation {
                        char_start: CharPosition {
                            line: line_counter,
                            column: current_char_idx_in_line - string_buffer.len(),
                        },
                        char_end: CharPosition {
                            line: line_counter,
                            column: current_char_idx_in_line,
                        },
                    });

                    string_buffer.clear();
                }

                if let Some(next_char) = char_list.get(char_idx + 1)
                    && *next_char == b':'
                {
                    token_list.push(Token::DoubleColon);
                    token_debug_info.push(DebugInformation {
                        char_start: CharPosition {
                            line: line_counter,
                            column: current_char_idx_in_line,
                        },
                        char_end: CharPosition {
                            line: line_counter,
                            column: current_char_idx_in_line + 2,
                        },
                    });

                    char_idx += 2;
                    continue;
                }

                token_list.push(Token::Colon);
                token_debug_info.push(DebugInformation {
                    char_start: CharPosition {
                        line: line_counter,
                        column: current_char_idx_in_line,
                    },
                    char_end: CharPosition {
                        line: line_counter,
                        column: current_char_idx_in_line + 1,
                    },
                });
            },
            b'&' => {
                if let Some(next_char) = char_list.get(char_idx + 1)
                    && *next_char == b'&'
                {
                    token_list.push(Token::And);
                    token_debug_info.push(DebugInformation {
                        char_start: CharPosition {
                            line: line_counter,
                            column: current_char_idx_in_line,
                        },
                        char_end: CharPosition {
                            line: line_counter,
                            column: current_char_idx_in_line + 2,
                        },
                    });

                    char_idx += 2;
                    continue;
                }

                token_list.push(Token::BitAnd);
                token_debug_info.push(DebugInformation {
                    char_start: CharPosition {
                        line: line_counter,
                        column: current_char_idx_in_line,
                    },
                    char_end: CharPosition {
                        line: line_counter,
                        column: current_char_idx_in_line + 1,
                    },
                });
            },
            b'!' => {
                if let Some(next_char) = char_list.get(char_idx + 1)
                    && *next_char == b'='
                {
                    token_list.push(Token::NotEqual);
                    token_debug_info.push(DebugInformation {
                        char_start: CharPosition {
                            line: line_counter,
                            column: current_char_idx_in_line,
                        },
                        char_end: CharPosition {
                            line: line_counter,
                            column: current_char_idx_in_line + 2,
                        },
                    });

                    char_idx += 2;
                    continue;
                }

                token_list.push(Token::Not);
                token_debug_info.push(DebugInformation {
                    char_start: CharPosition {
                        line: line_counter,
                        column: current_char_idx_in_line,
                    },
                    char_end: CharPosition {
                        line: line_counter,
                        column: current_char_idx_in_line + 1,
                    },
                });
            },
            b'>' => {
                if let Some(next_char) = char_list.get(char_idx + 1) {
                    if *next_char == b'>' {
                        token_list.push(Token::BitRight);
                        token_debug_info.push(DebugInformation {
                            char_start: CharPosition {
                                line: line_counter,
                                column: current_char_idx_in_line,
                            },
                            char_end: CharPosition {
                                line: line_counter,
                                column: current_char_idx_in_line + 2,
                            },
                        });

                        char_idx += 2;
                        continue;
                    }
                    else if *next_char == b'=' {
                        token_list.push(Token::EqBigger);
                        token_debug_info.push(DebugInformation {
                            char_start: CharPosition {
                                line: line_counter,
                                column: current_char_idx_in_line,
                            },
                            char_end: CharPosition {
                                line: line_counter,
                                column: current_char_idx_in_line + 2,
                            },
                        });

                        char_idx += 2;

                        continue;
                    }
                }

                token_list.push(Token::Bigger);
                token_debug_info.push(DebugInformation {
                    char_start: CharPosition {
                        line: line_counter,
                        column: current_char_idx_in_line,
                    },
                    char_end: CharPosition {
                        line: line_counter,
                        column: current_char_idx_in_line + 1,
                    },
                });
            },
            b'<' => {
                if let Some(next_char) = char_list.get(char_idx + 1) {
                    if *next_char == b'<' {
                        token_list.push(Token::BitLeft);
                        token_debug_info.push(DebugInformation {
                            char_start: CharPosition {
                                line: line_counter,
                                column: current_char_idx_in_line,
                            },
                            char_end: CharPosition {
                                line: line_counter,
                                column: current_char_idx_in_line + 2,
                            },
                        });

                        char_idx += 2;

                        continue;
                    }
                    else if *next_char == b'=' {
                        token_list.push(Token::EqSmaller);
                        token_debug_info.push(DebugInformation {
                            char_start: CharPosition {
                                line: line_counter,
                                column: current_char_idx_in_line,
                            },
                            char_end: CharPosition {
                                line: line_counter,
                                column: current_char_idx_in_line + 2,
                            },
                        });

                        char_idx += 2;

                        continue;
                    }
                }

                token_list.push(Token::Smaller);
                token_debug_info.push(DebugInformation {
                    char_start: CharPosition {
                        line: line_counter,
                        column: current_char_idx_in_line,
                    },
                    char_end: CharPosition {
                        line: line_counter,
                        column: current_char_idx_in_line + 1,
                    },
                });
            },
            b'|' => {
                if let Some(next_char) = char_list.get(char_idx + 1)
                    && *next_char == b'|'
                {
                    token_list.push(Token::Or);
                    token_debug_info.push(DebugInformation {
                        char_start: CharPosition {
                            line: line_counter,
                            column: current_char_idx_in_line,
                        },
                        char_end: CharPosition {
                            line: line_counter,
                            column: current_char_idx_in_line + 2,
                        },
                    });

                    char_idx += 2;
                    continue;
                }

                token_list.push(Token::BitOr);
                token_debug_info.push(DebugInformation {
                    char_start: CharPosition {
                        line: line_counter,
                        column: current_char_idx_in_line,
                    },
                    char_end: CharPosition {
                        line: line_counter,
                        column: current_char_idx_in_line + 1,
                    },
                });
            },
            _ => {
                if current_char != b' '
                    && current_char != NEWLINE_CHAR_U8
                    && current_char != ENDLINE_CHAR_U8
                {
                    string_buffer.push(current_char);
                }
            },
        }

        if current_char == NEWLINE_CHAR_U8 {
            line_counter += 1;
            line_char_idx = char_idx + 1;
        }

        char_idx += 1;
    }

    Ok((token_list, token_debug_info, char_idx))
}

fn match_multi_character_expression(string_to_match: &[u8]) -> anyhow::Result<Token>
{
    Ok(match string_to_match {
        b"ptr" => Token::TypeDefinition(TypeDiscriminant::Pointer),
        b"int" => Token::TypeDefinition(TypeDiscriminant::I32),
        b"uint" => Token::TypeDefinition(TypeDiscriminant::U32),
        b"float" => Token::TypeDefinition(TypeDiscriminant::F32),
        b"inthalf" => Token::TypeDefinition(TypeDiscriminant::I16),
        b"uinthalf" => Token::TypeDefinition(TypeDiscriminant::U16),
        b"floathalf" => Token::TypeDefinition(TypeDiscriminant::F16),
        b"intlong" => Token::TypeDefinition(TypeDiscriminant::I64),
        b"uintlong" => Token::TypeDefinition(TypeDiscriminant::U64),
        b"floatlong" => Token::TypeDefinition(TypeDiscriminant::F64),
        b"uintsmall" => Token::TypeDefinition(TypeDiscriminant::U8),
        b"bool" => Token::TypeDefinition(TypeDiscriminant::Boolean),
        b"void" => Token::TypeDefinition(TypeDiscriminant::Void),
        b"string" => Token::TypeDefinition(TypeDiscriminant::String),
        b"==" => Token::Equal,
        b"&&" => Token::And,
        b"||" => Token::Or,
        b"=+" => Token::SetValueAddition,
        b"=-" => Token::SetValueSubtraction,
        b"=*" => Token::SetValueMultiplication,
        b"=/" => Token::SetValueDivision,
        b"%=" => Token::SetValueModulo,
        b"false" => Token::Literal(Type::Boolean(false)),
        b"true" => Token::Literal(Type::Boolean(true)),
        b"external" => Token::External,
        b"import" => Token::Import,
        b"function" => Token::Function,
        b"return" => Token::Return,
        b"as" => Token::As,
        // Unused
        b"extend" => Token::Extend,
        b"struct" => Token::Struct,
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
        b"cold" => Token::CompilerHint(common::parser::CompilerHint::Cold),
        b"nofree" => Token::CompilerHint(common::parser::CompilerHint::NoFree),
        b"nounwind" => Token::CompilerHint(common::parser::CompilerHint::NoUnWind),
        b"inline" => Token::CompilerHint(common::parser::CompilerHint::Inline),
        b"feature" => Token::CompilerHint(common::parser::CompilerHint::Feature),

        _ => eval_constant_definition(string_to_match),
    })
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

pub fn strip_range_from_token_list(tokens: &Vec<(Token, Range<usize>)>) -> Vec<Token>
{
    let mut buf = Vec::new();

    for (token, _) in tokens {
        buf.push(token.clone());
    }

    buf
}
