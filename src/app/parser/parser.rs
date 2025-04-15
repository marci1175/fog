use std::collections::HashMap;

use crate::app::parser::FunctionDefinition;

use super::{ParsedTokens, Tokens};

pub fn parse_code(raw_string: String) -> Vec<Tokens> {
    let mut char_idx: usize = 0;

    let mut token_list: Vec<Tokens> = Vec::new();

    let char_list = raw_string.chars().collect::<Vec<char>>();

    let mut string_buffer = String::new();

    while char_idx < raw_string.len() {
        let current_char = char_list[char_idx];

        let single_char = match current_char {
            '+' => Some(Tokens::Addition),
            '-' => Some(Tokens::Subtraction),
            '*' => Some(Tokens::Multiplication),
            '/' => Some(Tokens::Division),
            ')' => Some(Tokens::CloseBracket),
            '(' => Some(Tokens::OpenBracket),
            '}' => Some(Tokens::CloseBraces),
            '{' => Some(Tokens::OpenBraces),
            '!' => Some(Tokens::Not),
            ';' => Some(Tokens::LineBreak),
            ',' => Some(Tokens::Comma),
            ':' => Some(Tokens::Colon),

            _ => None,
        };

        if let Some(single_char_token) = single_char {
            if !string_buffer.trim().is_empty() {
                let token = match_multi_character_expression(string_buffer.clone());

                token_list.push(token);
            }

            token_list.push(single_char_token);

            string_buffer.clear();
        } else if current_char == '#' {
            let mut comment_buffer = String::new();

            let mut comment_idx = char_idx + 1;

            loop {
                let quote_char = char_list[comment_idx];
                if quote_char == '\n' {
                    token_list.push(Tokens::Comment(comment_buffer.trim().to_string()));

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
                let quote_char = char_list[quote_idx];
                if quote_char == '"' {
                    token_list.push(Tokens::Quote(quotes_buffer));

                    char_idx = quote_idx + 1;

                    break;
                }

                quotes_buffer.push(quote_char);

                quote_idx += 1;
            }

            continue;
        } else if current_char == '=' {
            if let Some(next_char) = char_list.get(char_idx + 1) {
                if *next_char == '=' {
                    token_list.push(Tokens::Equals);

                    char_idx += 1;
                    continue;
                }
            }

            token_list.push(Tokens::SetValue);
        } else if current_char == '&' {
            if let Some(next_char) = char_list.get(char_idx + 1) {
                if *next_char == '&' {
                    token_list.push(Tokens::And);

                    char_idx += 1;
                    continue;
                }
            }

            token_list.push(Tokens::BitAnd);
        } else if current_char == '>' {
            if let Some(next_char) = char_list.get(char_idx + 1) {
                if *next_char == '>' {
                    token_list.push(Tokens::BitRight);

                    char_idx += 1;
                    continue;
                }
            }

            token_list.push(Tokens::Bigger);
        } else if current_char == '<' {
            if let Some(next_char) = char_list.get(char_idx + 1) {
                if *next_char == '<' {
                    token_list.push(Tokens::BitLeft);

                    char_idx += 1;

                    continue;
                }
            }

            token_list.push(Tokens::Smaller);
        } else if current_char == '|' {
            if let Some(next_char) = char_list.get(char_idx + 1) {
                if *next_char == '|' {
                    token_list.push(Tokens::Or);

                    char_idx += 1;
                    continue;
                }
            }

            token_list.push(Tokens::BitOr);
        } else if current_char == ' ' && !string_buffer.trim().is_empty() {
            let token = match_multi_character_expression(string_buffer.clone());

            token_list.push(token);

            string_buffer.clear();
        } else if current_char != ' ' {
            string_buffer.push(current_char);
        }

        char_idx += 1;
    }

    token_list
}

fn match_multi_character_expression(string_buffer: String) -> Tokens {
    let trimmed_string = string_buffer.trim();

    let token = match trimmed_string {
        "int" => Tokens::TypeDefinition(crate::app::type_system::TypeDiscriminants::I32),
        "string" => Tokens::TypeDefinition(crate::app::type_system::TypeDiscriminants::String),
        "uint" => Tokens::TypeDefinition(crate::app::type_system::TypeDiscriminants::U32),
        "float" => Tokens::TypeDefinition(crate::app::type_system::TypeDiscriminants::F32),
        "uintsmall" => Tokens::TypeDefinition(crate::app::type_system::TypeDiscriminants::U8),
        "void" => Tokens::TypeDefinition(crate::app::type_system::TypeDiscriminants::Void),
        "bool" => Tokens::TypeDefinition(crate::app::type_system::TypeDiscriminants::Boolean),
        "==" => Tokens::Equals,
        ">=" => Tokens::EqBigger,
        "<=" => Tokens::EqSmaller,
        "&&" => Tokens::And,
        "||" => Tokens::Or,
        "if" => Tokens::If,
        "function" => Tokens::Function,

        _ => Tokens::Identifier(trimmed_string.to_string()),
    };

    token
}

pub fn parse_token(tokens: Vec<Tokens>) -> Vec<ParsedTokens> {
    let mut parsed_tokens: Vec<ParsedTokens> = vec![];

    let mut function_list: HashMap<String, FunctionDefinition> = HashMap::new();

    for token in tokens {
        
    }

    vec![]
}