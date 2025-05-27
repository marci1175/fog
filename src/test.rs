const TOKENISATION_INPUT_SAMPLE: &str = r#"
struct test {
    inner: int,
}

function main(): int {
    test var1 = test { inner: 0, };
}
"#;

#[cfg(test)]
mod parsing_tests {
    use fog::app::parser::{tokenizer::tokenize, types::Token};

    use crate::TOKENISATION_INPUT_SAMPLE;

    #[test]
    pub fn tokenization() {
        let tokens = tokenize(TOKENISATION_INPUT_SAMPLE).unwrap();

        let correct_tokens = vec![
            Token::Struct,
            Token::Identifier("test".to_string()),
            Token::OpenBraces,
            Token::Identifier("inner".to_string()),
            Token::Colon,
            Token::TypeDefinition(fog::app::type_system::type_system::TypeDiscriminants::I32),
            Token::Comma,
            Token::CloseBraces,
            Token::Function,
            Token::Identifier("main".to_string()),
            Token::OpenParentheses,
            Token::CloseParentheses,
            Token::Colon,
            Token::TypeDefinition(fog::app::type_system::type_system::TypeDiscriminants::I32),
            Token::OpenBraces,
            // The type is not known for custom types
            Token::Identifier("test".to_string()),
            Token::Identifier("var1".to_string()),
            Token::SetValue,
            Token::Identifier("test".to_string()),
            Token::OpenBraces,
            Token::Identifier("inner".to_string()),
            Token::Colon,
            Token::UnparsedLiteral("0".to_string()),
            Token::Comma,
            Token::CloseBraces,
            Token::LineBreak,
            Token::Return,
            Token::Identifier("var1".to_string()),
            Token::Dot,
            Token::Identifier("inner".to_string()),
            Token::CloseBraces,
        ];

        assert_eq!(tokens, correct_tokens);
    }
}
