use crate::app::type_system::type_system::Type;

use super::{error::ParserError, types::Token};

fn contains_non_digits(s: &str) -> bool {
    s.chars().any(|c| !c.is_ascii_digit())
}

pub fn tokenize(raw_input: &str) -> Result<Vec<Token>, ParserError> {
    let mut char_idx: usize = 0;

    let mut token_list: Vec<Token> = Vec::new();

    let char_list = raw_input.chars().collect::<Vec<char>>();

    let mut string_buffer = String::new();

    while char_idx < char_list.len() {
        let current_char = char_list[char_idx];

        let single_char = match current_char {
            '+' => Some(Token::Addition),
            '*' => Some(Token::Multiplication),
            '/' => Some(Token::Division),
            ')' => Some(Token::CloseParentheses),
            '(' => Some(Token::OpenParentheses),
            '}' => Some(Token::CloseBraces),
            '{' => Some(Token::OpenBraces),
            '[' => Some(Token::OpenSquareBrackets),
            ']' => Some(Token::CloseSquareBrackets),
            ';' => Some(Token::LineBreak),
            ',' => Some(Token::Comma),
            '%' => Some(Token::Modulo),
            _ => None,
        };

        if let Some(single_char_token) = single_char {
            if !string_buffer.trim().is_empty() {
                let token = match_multi_character_expression(string_buffer.clone());

                token_list.push(token);
            }

            token_list.push(single_char_token);

            string_buffer.clear();
        } else if current_char == '.' {
            let next_char = char_list.get(char_idx + 1);
            let next_char_2 = char_list.get(char_idx + 2);

            if Some(&'.') == next_char_2 && Some(&'.') == next_char {
                token_list.push(Token::Ellipsis);

                char_idx += 3;

                continue;
            } else if !contains_non_digits(&string_buffer) {
                string_buffer.push(current_char);
            } else {
                token_list.push(Token::Dot);
            }
        } else if current_char == '-' {
            let last_token = &token_list[token_list.len() - 1];

            // If the last token was a number we know that we are subtracting
            if (matches!(last_token, Token::Literal(_))
                || matches!(last_token, Token::UnparsedLiteral(_))
                || matches!(last_token, Token::Identifier(_))
                || matches!(last_token, Token::CloseParentheses))
            {
                token_list.push(Token::Subtraction);
            }
            // If the last token wasnt a number we know that we are defining a negative number
            else {
                string_buffer.push(current_char);
            }
        } else if current_char == '#' {
            let mut comment_buffer = String::new();

            let mut comment_idx = char_idx + 1;

            loop {
                let quote_char = char_list[comment_idx];
                if quote_char == '\n' {
                    token_list.push(Token::Comment(comment_buffer.trim().to_string()));

                    char_idx = comment_idx + 1;

                    break;
                }

                comment_buffer.push(quote_char);

                comment_idx += 1;
            }

            continue;
        } else if current_char == '"' {
            let mut quotes_buffer = String::new();

            let mut quote_idx = char_idx + 1;

            loop {
                let quote_char = char_list.get(quote_idx);

                match quote_char {
                    Some(quote_char) => {
                        if *quote_char == '\\' {
                            match char_list.get(quote_idx + 1) {
                                Some('n') => {
                                    quotes_buffer.push('\n');

                                    quote_idx += 2;

                                    continue;
                                }
                                Some('r') => {
                                    quotes_buffer.push('\r');

                                    quote_idx += 2;

                                    continue;
                                }
                                Some('t') => {
                                    quotes_buffer.push('\t');

                                    quote_idx += 2;

                                    continue;
                                }
                                Some('0') => {
                                    quotes_buffer.push('\0');

                                    quote_idx += 2;

                                    continue;
                                }
                                Some('\\') => {
                                    quotes_buffer.push('\\');
                                    quote_idx += 2;

                                    continue;
                                }
                                Some(char) => {
                                    quotes_buffer.push('\\');
                                    quotes_buffer.push(*char);

                                    quote_idx += 2;

                                    continue;
                                }

                                None => {}
                            }
                        }

                        if *quote_char == '"' {
                            token_list.push(Token::Literal(Type::String(quotes_buffer)));

                            char_idx = quote_idx + 1;

                            break;
                        }

                        quotes_buffer.push(*quote_char);

                        quote_idx += 1;
                    }
                    // If there are no more tokens left and we are still in the quote
                    None => {
                        return Err(ParserError::SyntaxError(
                            super::error::SyntaxError::OpenQuotes,
                        ));
                    }
                }
            }

            continue;
        } else if current_char == '=' {
            if let Some(next_char) = char_list.get(char_idx + 1) {
                match *next_char {
                    '=' => {
                        token_list.push(Token::Equal);

                        char_idx += 2;
                    }
                    '+' => {
                        token_list.push(Token::SetValueAddition);

                        char_idx += 2;
                    }
                    '-' => {
                        token_list.push(Token::SetValueSubtraction);

                        char_idx += 2;
                    }
                    '*' => {
                        token_list.push(Token::SetValueMultiplication);

                        char_idx += 2;
                    }
                    '/' => {
                        token_list.push(Token::SetValueDivision);

                        char_idx += 2;
                    }
                    '%' => {
                        token_list.push(Token::SetValueModulo);

                        char_idx += 2;
                    }

                    _ => {
                        token_list.push(Token::SetValue);

                        char_idx += 1;
                    }
                }

                continue;
            }
        } else if current_char == ':' {
            if !string_buffer.trim().is_empty() {
                let token = match_multi_character_expression(string_buffer.clone());

                token_list.push(token);

                string_buffer.clear();
            }

            if let Some(next_char) = char_list.get(char_idx + 1) {
                if *next_char == ':' {
                    token_list.push(Token::DoubleColon);

                    char_idx += 2;
                    continue;
                }
            }

            token_list.push(Token::Colon);
        } else if current_char == '&' {
            if let Some(next_char) = char_list.get(char_idx + 1) {
                if *next_char == '&' {
                    token_list.push(Token::And);

                    char_idx += 2;
                    continue;
                }
            }

            token_list.push(Token::BitAnd);
        } else if current_char == '!' {
            if let Some(next_char) = char_list.get(char_idx + 1) {
                if *next_char == '=' {
                    token_list.push(Token::NotEqual);

                    char_idx += 2;
                    continue;
                }
            }

            token_list.push(Token::Not);
        } else if current_char == '>' {
            if let Some(next_char) = char_list.get(char_idx + 1) {
                if *next_char == '>' {
                    token_list.push(Token::BitRight);

                    char_idx += 2;
                    continue;
                } else if *next_char == '=' {
                    token_list.push(Token::EqBigger);

                    char_idx += 2;

                    continue;
                }
            }

            token_list.push(Token::Bigger);
        } else if current_char == '<' {
            if let Some(next_char) = char_list.get(char_idx + 1) {
                if *next_char == '<' {
                    token_list.push(Token::BitLeft);

                    char_idx += 2;

                    continue;
                } else if *next_char == '=' {
                    token_list.push(Token::EqSmaller);

                    char_idx += 2;

                    continue;
                }
            }

            token_list.push(Token::Smaller);
        } else if current_char == '|' {
            if let Some(next_char) = char_list.get(char_idx + 1) {
                if *next_char == '|' {
                    token_list.push(Token::Or);

                    char_idx += 2;
                    continue;
                }
            }

            token_list.push(Token::BitOr);
        } else if current_char == ' ' && !string_buffer.trim().is_empty() {
            let token = match_multi_character_expression(string_buffer.clone());

            token_list.push(token);

            string_buffer.clear();
        } else if current_char != ' ' {
            string_buffer.push(current_char);
        }

        char_idx += 1;
    }

    Ok(token_list)
}

fn match_multi_character_expression(string_buffer: String) -> Token {
    let trimmed_string = string_buffer.trim();

    match trimmed_string {
        "int" => Token::TypeDefinition(crate::app::type_system::type_system::TypeDiscriminant::I32),
        "uint" => {
            Token::TypeDefinition(crate::app::type_system::type_system::TypeDiscriminant::U32)
        }
        "float" => {
            Token::TypeDefinition(crate::app::type_system::type_system::TypeDiscriminant::F32)
        }

        "inthalf" => {
            Token::TypeDefinition(crate::app::type_system::type_system::TypeDiscriminant::I16)
        }
        "uinthalf" => {
            Token::TypeDefinition(crate::app::type_system::type_system::TypeDiscriminant::U16)
        }
        "floathalf" => {
            Token::TypeDefinition(crate::app::type_system::type_system::TypeDiscriminant::F16)
        }

        "intlong" => {
            Token::TypeDefinition(crate::app::type_system::type_system::TypeDiscriminant::I64)
        }
        "uintlong" => {
            Token::TypeDefinition(crate::app::type_system::type_system::TypeDiscriminant::U64)
        }
        "floatlong" => {
            Token::TypeDefinition(crate::app::type_system::type_system::TypeDiscriminant::F64)
        }

        "uintsmall" => {
            Token::TypeDefinition(crate::app::type_system::type_system::TypeDiscriminant::U8)
        }

        "bool" => {
            Token::TypeDefinition(crate::app::type_system::type_system::TypeDiscriminant::Boolean)
        }
        "void" => {
            Token::TypeDefinition(crate::app::type_system::type_system::TypeDiscriminant::Void)
        }
        "string" => {
            Token::TypeDefinition(crate::app::type_system::type_system::TypeDiscriminant::String)
        }

        "==" => Token::Equal,
        "&&" => Token::And,
        "||" => Token::Or,
        "if" => Token::If,
        "=+" => Token::SetValueAddition,
        "=-" => Token::SetValueSubtraction,
        "=*" => Token::SetValueMultiplication,
        "=/" => Token::SetValueDivision,
        "%=" => Token::SetValueModulo,
        "false" => Token::Literal(Type::Boolean(false)),
        "true" => Token::Literal(Type::Boolean(true)),
        "import" => Token::Import,
        "function" => Token::Function,
        "return" => Token::Return,
        "as" => Token::As,
        "extend" => Token::Extend,
        "struct" => Token::Struct,
        "else" => Token::Else,
        "loop" => Token::Loop,
        "for" => Token::For,
        "break" => Token::Break,
        "continue" => Token::Continue,

        _ => eval_constant_definition(trimmed_string.to_string()),
    }
}

// I guess this works too lol
pub fn eval_constant_definition(raw_string: String) -> Token {
    if raw_string.parse::<u8>().is_ok()
        || raw_string.parse::<u32>().is_ok()
        || raw_string.parse::<f32>().is_ok()
        || raw_string.parse::<i32>().is_ok()
    {
        Token::UnparsedLiteral(raw_string)
    } else {
        Token::Identifier(raw_string)
    }
}
