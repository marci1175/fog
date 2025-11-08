use fog_common::{
    error::{parser::ParserError, syntax::SyntaxError},
    tokenizer::{Token, find_closing_angled_bracket_char},
    ty::{Type, TypeDiscriminant},
};

fn only_contains_digits(s: &str) -> bool
{
    s.chars().all(|c| c.is_ascii_digit())
}

pub fn tokenize(
    raw_input: &str,
    stop_at_token: Option<Token>,
) -> Result<(Vec<Token>, usize), ParserError>
{
    let mut char_idx: usize = 0;

    let mut token_list: Vec<Token> = Vec::new();

    let char_list = raw_input.chars().collect::<Vec<char>>();

    let mut string_buffer = String::new();

    while char_idx < char_list.len() {
        if let Some(stop_at_token) = &stop_at_token {
            if token_list
                .last()
                .is_some_and(|last_token| last_token == stop_at_token)
            {
                return Ok((token_list, char_idx));
            }
        }

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
            ';' => Some(Token::SemiColon),
            ',' => Some(Token::Comma),
            '%' => Some(Token::Modulo),
            '@' => Some(Token::CompilerHintSymbol),
            _ => None,
        };

        if let Some(single_char_token) = single_char {
            if !string_buffer.trim().is_empty() {
                let token = match_multi_character_expression(string_buffer.clone());

                token_list.push(token);
            }

            token_list.push(single_char_token);

            string_buffer.clear();
        }
        else if current_char == '.' {
            let next_char = char_list.get(char_idx + 1);
            let next_char_2 = char_list.get(char_idx + 2);

            if Some(&'.') == next_char_2 && Some(&'.') == next_char {
                token_list.push(Token::Ellipsis);

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
                    let token = match_multi_character_expression(string_buffer.clone());

                    token_list.push(token);

                    string_buffer.clear();
                }

                token_list.push(Token::Dot);
            }
        }
        else if current_char == '-' {
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
        }
        else if current_char == '#' {
            let mut comment_buffer = String::new();

            if let Some(char) = char_list.get(char_idx + 1) {
                if *char == '-' {
                    if char_list.get(char_idx + 2) == Some(&'>') {
                        char_idx += 3;

                        if stop_at_token.is_some() {
                            token_list.push(Token::MultilineComment);

                            continue;
                        }

                        // Capture everything until the finishing #->
                        // The reason im passing this into a tokenizer function is because the MultilineComment token could be in quotes and that would be invalid to capture.
                        // We can ignore the captured tokens, we increment the char_idx by the chars parsed.
                        // The captured output does not contain the contents of the multiline comment.
                        let (_, idx) =
                            tokenize(&raw_input[char_idx..], Some(Token::MultilineComment))?;

                        char_idx += idx;

                        // Continue looping through the tokens
                        // We continue here because we dont want to increment the char_idx one more time.
                        continue;
                    }
                }
                if *char == '#' {
                    if char_list.get(char_idx + 2) == Some(&'#') {
                        char_idx += 3;

                        loop {
                            let quote_char = char_list[char_idx + 1];

                            if quote_char == '\n' {
                                token_list
                                    .push(Token::DocComment(comment_buffer.trim().to_string()));

                                char_idx += 2;

                                break;
                            }

                            comment_buffer.push(quote_char);

                            char_idx += 1;
                        }

                        continue;
                    }
                }
                else {
                    loop {
                        let quote_char = char_list[char_idx + 1];

                        if quote_char == '\r' {
                            token_list.push(Token::Comment(comment_buffer.trim().to_string()));

                            char_idx += 2;

                            break;
                        }

                        comment_buffer.push(quote_char);

                        char_idx += 1;
                    }

                    continue;
                }
            }
        }
        else if current_char == '"' {
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
                                },
                                Some('r') => {
                                    quotes_buffer.push('\r');

                                    quote_idx += 2;

                                    continue;
                                },
                                Some('t') => {
                                    quotes_buffer.push('\t');

                                    quote_idx += 2;

                                    continue;
                                },
                                Some('0') => {
                                    quotes_buffer.push('\0');

                                    quote_idx += 2;

                                    continue;
                                },
                                Some('\\') => {
                                    quotes_buffer.push('\\');
                                    quote_idx += 2;

                                    continue;
                                },
                                Some(char) => {
                                    quotes_buffer.push('\\');
                                    quotes_buffer.push(*char);

                                    quote_idx += 2;

                                    continue;
                                },

                                None => {},
                            }
                        }

                        if *quote_char == '"' {
                            token_list.push(Token::Literal(Type::String(quotes_buffer)));

                            char_idx = quote_idx + 1;

                            break;
                        }

                        quotes_buffer.push(*quote_char);

                        quote_idx += 1;
                    },
                    // If there are no more tokens left and we are still in the quote
                    None => {
                        return Err(ParserError::SyntaxError(SyntaxError::OpenQuotes));
                    },
                }
            }

            continue;
        }
        else if current_char == '=' {
            if let Some(next_char) = char_list.get(char_idx + 1) {
                match *next_char {
                    '=' => {
                        token_list.push(Token::Equal);

                        char_idx += 2;
                    },
                    '+' => {
                        token_list.push(Token::SetValueAddition);

                        char_idx += 2;
                    },
                    '-' => {
                        token_list.push(Token::SetValueSubtraction);

                        char_idx += 2;
                    },
                    '*' => {
                        token_list.push(Token::SetValueMultiplication);

                        char_idx += 2;
                    },
                    '/' => {
                        token_list.push(Token::SetValueDivision);

                        char_idx += 2;
                    },
                    '%' => {
                        token_list.push(Token::SetValueModulo);

                        char_idx += 2;
                    },

                    _ => {
                        token_list.push(Token::SetValue);

                        char_idx += 1;
                    },
                }

                continue;
            }
        }
        else if current_char == ':' {
            if !string_buffer.trim().is_empty() {
                let token = match_multi_character_expression(string_buffer.clone());

                token_list.push(token);

                string_buffer.clear();
            }

            if let Some(next_char) = char_list.get(char_idx + 1)
                && *next_char == ':'
            {
                token_list.push(Token::DoubleColon);

                char_idx += 2;
                continue;
            }

            token_list.push(Token::Colon);
        }
        else if current_char == '&' {
            if let Some(next_char) = char_list.get(char_idx + 1)
                && *next_char == '&'
            {
                token_list.push(Token::And);

                char_idx += 2;
                continue;
            }

            token_list.push(Token::BitAnd);
        }
        else if current_char == '!' {
            if let Some(next_char) = char_list.get(char_idx + 1)
                && *next_char == '='
            {
                token_list.push(Token::NotEqual);

                char_idx += 2;
                continue;
            }

            token_list.push(Token::Not);
        }
        else if current_char == '>' {
            if let Some(next_char) = char_list.get(char_idx + 1) {
                if *next_char == '>' {
                    token_list.push(Token::BitRight);

                    char_idx += 2;
                    continue;
                }
                else if *next_char == '=' {
                    token_list.push(Token::EqBigger);

                    char_idx += 2;

                    continue;
                }
            }

            token_list.push(Token::Bigger);
        }
        else if string_buffer.trim() == "array" {
            if current_char == '<' {
                char_idx += 1;

                let closing_idx = find_closing_angled_bracket_char(&char_list[char_idx..], 0)?;

                let list_type = &char_list[char_idx..closing_idx + char_idx];

                let comma_pos = list_type.len()
                    - list_type.iter().rev().position(|char| *char == ',').ok_or(
                        ParserError::SyntaxError(SyntaxError::MissingCommaAtArrayDef),
                    )?;

                let array_len = list_type[comma_pos..]
                    .iter()
                    .collect::<String>()
                    .trim()
                    .to_string();

                let list_type_def = list_type[..comma_pos - 1]
                    .iter()
                    .collect::<String>()
                    .trim()
                    .to_string();

                let (inner_token, _) = tokenize(&list_type_def, None)?;

                if inner_token.len() > 1 {
                    return Err(ParserError::InvalidArrayTypeDefinition(inner_token));
                }

                token_list.push(Token::TypeDefinition(TypeDiscriminant::Array((
                    Box::new(inner_token[0].clone()),
                    array_len.parse::<usize>().map_err(|_| {
                        ParserError::SyntaxError(SyntaxError::UnparsableExpression(
                            array_len.clone(),
                        ))
                    })?,
                ))));

                string_buffer.clear();

                char_idx += closing_idx;
            }
        }
        else if current_char == '<' {
            if let Some(next_char) = char_list.get(char_idx + 1) {
                if *next_char == '<' {
                    token_list.push(Token::BitLeft);

                    char_idx += 2;

                    continue;
                }
                else if *next_char == '=' {
                    token_list.push(Token::EqSmaller);

                    char_idx += 2;

                    continue;
                }
            }

            token_list.push(Token::Smaller);
        }
        else if current_char == '|' {
            if let Some(next_char) = char_list.get(char_idx + 1)
                && *next_char == '|'
            {
                token_list.push(Token::Or);

                char_idx += 2;
                continue;
            }

            token_list.push(Token::BitOr);
        }
        else if (current_char == ' ' || current_char == '\n') && !string_buffer.trim().is_empty()
        {
            let token = match_multi_character_expression(string_buffer.clone());

            token_list.push(token);

            string_buffer.clear();
        }
        else if string_buffer.len() + 1 == char_list.len() {
            string_buffer.push(char_list[char_list.len() - 1]);

            let token = match_multi_character_expression(string_buffer.clone());

            token_list.push(token);

            string_buffer.clear();
        }
        else if current_char != ' ' && current_char != '\n' && current_char != '\r' {
            string_buffer.push(current_char);
        }

        char_idx += 1;
    }

    Ok((token_list, char_idx))
}

fn match_multi_character_expression(string_buffer: String) -> Token
{
    let trimmed_string = string_buffer.trim();

    match trimmed_string {
        "int" => Token::TypeDefinition(TypeDiscriminant::I32),
        "uint" => Token::TypeDefinition(TypeDiscriminant::U32),
        "float" => Token::TypeDefinition(TypeDiscriminant::F32),

        "inthalf" => Token::TypeDefinition(TypeDiscriminant::I16),
        "uinthalf" => Token::TypeDefinition(TypeDiscriminant::U16),
        "floathalf" => Token::TypeDefinition(TypeDiscriminant::F16),

        "intlong" => Token::TypeDefinition(TypeDiscriminant::I64),
        "uintlong" => Token::TypeDefinition(TypeDiscriminant::U64),
        "floatlong" => Token::TypeDefinition(TypeDiscriminant::F64),

        "uintsmall" => Token::TypeDefinition(TypeDiscriminant::U8),

        "bool" => Token::TypeDefinition(TypeDiscriminant::Boolean),
        "void" => Token::TypeDefinition(TypeDiscriminant::Void),
        "string" => Token::TypeDefinition(TypeDiscriminant::String),

        "==" => Token::Equal,
        "&&" => Token::And,
        "||" => Token::Or,
        "=+" => Token::SetValueAddition,
        "=-" => Token::SetValueSubtraction,
        "=*" => Token::SetValueMultiplication,
        "=/" => Token::SetValueDivision,
        "%=" => Token::SetValueModulo,
        "false" => Token::Literal(Type::Boolean(false)),
        "true" => Token::Literal(Type::Boolean(true)),
        "external" => Token::External,
        "import" => Token::Import,
        "function" => Token::Function,
        "return" => Token::Return,
        "as" => Token::As,

        // Unused
        "extend" => Token::Extend,

        "struct" => Token::Struct,

        "if" => Token::If,
        "else" => Token::Else,
        "elseif" => Token::ElseIf,

        "loop" => Token::Loop,
        "for" => Token::For,
        "break" => Token::Break,
        "continue" => Token::Continue,

        "priv" => Token::Private,
        "pub" => Token::Public,
        "libpub" => Token::PublicLibrary,
        "exp" => Token::Export,

        "#->" => Token::MultilineComment,

        "cold" => Token::CompilerHint(fog_common::parser::CompilerHint::Cold),
        "nofree" => Token::CompilerHint(fog_common::parser::CompilerHint::NoFree),
        "nounwind" => Token::CompilerHint(fog_common::parser::CompilerHint::NoUnWind),
        "inline" => Token::CompilerHint(fog_common::parser::CompilerHint::Inline),
        "feature" => Token::CompilerHint(fog_common::parser::CompilerHint::Feature),

        _ => eval_constant_definition(trimmed_string.to_string()),
    }
}

// I guess this works too lol
pub fn eval_constant_definition(raw_string: String) -> Token
{
    if raw_string.parse::<u8>().is_ok()
        || raw_string.parse::<u32>().is_ok()
        || raw_string.parse::<f32>().is_ok()
        || raw_string.parse::<i32>().is_ok()
    {
        Token::UnparsedLiteral(raw_string)
    }
    else {
        Token::Identifier(raw_string)
    }
}
