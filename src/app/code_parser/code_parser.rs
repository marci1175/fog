use super::Tokens;

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
            '/' => Some(Tokens::Division),
            '>' => Some(Tokens::Bigger),
            '<' => Some(Tokens::Smaller),
            ';' => Some(Tokens::LineBreak),
            '=' => Some(Tokens::SetValue),
            ')' => Some(Tokens::CloseBracket),
            '(' => Some(Tokens::OpenBracket),

            _ => None,
        };

        if let Some(single_char_token) = single_char {
            if !string_buffer.is_empty() {
                let token = match_multi_character_expression(string_buffer.clone());

                token_list.push(token);
            }

            token_list.push(single_char_token);

            string_buffer.clear();
        }
        else if current_char == ' ' && !string_buffer.is_empty() {
            let token = match_multi_character_expression(string_buffer.clone());

            token_list.push(token);

            string_buffer.clear();
        }
        else if current_char != ' ' {
            string_buffer.push(current_char);
        }

        char_idx += 1;
    }

    token_list
}

fn match_multi_character_expression(string_buffer: String) -> Tokens {
    let trimmed_string = string_buffer.trim();

    let token = match trimmed_string {
        "int" => Tokens::TypeDefinition(crate::app::type_system::TypesDiscriminants::I32),
        "uint" => Tokens::TypeDefinition(crate::app::type_system::TypesDiscriminants::U32),
        "float" => Tokens::TypeDefinition(crate::app::type_system::TypesDiscriminants::F32),
        "uintsmall" => Tokens::TypeDefinition(crate::app::type_system::TypesDiscriminants::U8),
        "void" => Tokens::TypeDefinition(crate::app::type_system::TypesDiscriminants::Void),
        "bool" => Tokens::TypeDefinition(crate::app::type_system::TypesDiscriminants::Boolean),
        "==" => Tokens::Equals,
        ">=" => Tokens::EqBigger,
        "<=" => Tokens::EqSmaller,
        "&&" => Tokens::And,
        "||" => Tokens::Or,

        _ => Tokens::Identifier(trimmed_string.to_string()),
    };

    token
}
