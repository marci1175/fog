use crate::app::type_system::Type;

use super::{error::ParserError, types::Token};

pub fn tokenize(raw_string: String) -> Result<Vec<Token>, ParserError> {
    let mut char_idx: usize = 0;

    let mut token_list: Vec<Token> = Vec::new();

    let char_list = raw_string.chars().collect::<Vec<char>>();

    let mut string_buffer = String::new();

    while char_idx < raw_string.len() {
        let current_char = char_list[char_idx];

        let single_char = match current_char {
            '+' => Some(Token::Addition),
            '*' => Some(Token::Multiplication),
            '/' => Some(Token::Division),
            ')' => Some(Token::CloseBracket),
            '(' => Some(Token::OpenBracket),
            '}' => Some(Token::CloseBraces),
            '{' => Some(Token::OpenBraces),
            '!' => Some(Token::Not),
            ';' => Some(Token::LineBreak),
            ',' => Some(Token::Comma),
            ':' => Some(Token::Colon),
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
        } else if current_char == '-' {
            // If the last token was a number we know that we are subtracting
            if matches!(token_list[token_list.len() - 1], Token::Literal(_))
                || matches!(token_list[token_list.len() - 1], Token::UnparsedLiteral(_))
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
                if *next_char == '=' {
                    token_list.push(Token::Equals);

                    char_idx += 2;
                    continue;
                }
            }

            token_list.push(Token::SetValue);
        } else if current_char == '&' {
            if let Some(next_char) = char_list.get(char_idx + 1) {
                if *next_char == '&' {
                    token_list.push(Token::And);

                    char_idx += 2;
                    continue;
                }
            }

            token_list.push(Token::BitAnd);
        } else if current_char == '>' {
            if let Some(next_char) = char_list.get(char_idx + 1) {
                if *next_char == '>' {
                    token_list.push(Token::BitRight);

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
        "int" => Token::TypeDefinition(crate::app::type_system::TypeDiscriminants::I32),
        "string" => Token::TypeDefinition(crate::app::type_system::TypeDiscriminants::String),
        "uint" => Token::TypeDefinition(crate::app::type_system::TypeDiscriminants::U32),
        "float" => Token::TypeDefinition(crate::app::type_system::TypeDiscriminants::F32),
        "uintsmall" => Token::TypeDefinition(crate::app::type_system::TypeDiscriminants::U8),
        "void" => Token::TypeDefinition(crate::app::type_system::TypeDiscriminants::Void),
        "bool" => Token::TypeDefinition(crate::app::type_system::TypeDiscriminants::Boolean),
        "==" => Token::Equals,
        ">=" => Token::EqBigger,
        "<=" => Token::EqSmaller,
        "&&" => Token::And,
        "||" => Token::Or,
        "if" => Token::If,
        "+=" => Token::SetValueAddition,
        "-=" => Token::SetValueSubtraction,
        "*=" => Token::SetValueMultiplication,
        "/=" => Token::SetValueDivision,
        "%=" => Token::SetValueModulo,
        "false" => Token::Literal(Type::Boolean(false)),
        "true" => Token::Literal(Type::Boolean(true)),
        "function" => Token::Function,
        "return" => Token::Return,
        "as" => Token::As,

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
