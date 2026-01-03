use crate::{
    error::{parser::ParserError, syntax::SyntaxError},
    tokenizer::Token,
};
use anyhow::Result;

/// Make sure to pass in a slice of the tokens in which the first token is an `Token::Identifier`.
pub fn parse_import_path(tokens: &[Token]) -> Result<(Vec<String>, usize)>
{
    let mut import_path = vec![];
    let mut idx = 0;

    while idx < tokens.len() {
        // Check if the module definition path contains the correct tokens
        if let Some(Token::Identifier(module_name)) = tokens.get(idx) {
            import_path.push(module_name.clone());
        }
        else {
            return Err(
                ParserError::InvalidModulePathDefinition(tokens.get(idx).unwrap().clone()).into(),
            );
        }

        // Check if there is another double colon, that means that the module path is not fully definied yet.
        if let Some(Token::DoubleColon) = tokens.get(idx + 1) {
            idx += 2;
        }
        // If there are no more double colons after the identifier, that is the last item in the path list.
        // That will be the item's name at the specified module path.
        else if let Some(Token::SemiColon) = tokens.get(idx + 1) {
            break;
        }
        // Return a missing semi colon error
        else {
            return Err(SyntaxError::MissingSemiColon.into());
        }
    }

    Ok((import_path, idx))
}
