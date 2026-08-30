use crate::{
    error::{Spanned, parser::ParserError, syntax::SyntaxError},
    parser::{
        common::{StatementVariant, Streamable, find_closing_paren},
        statement::Expr,
    },
    tokenizer::{Token, TokenDiscriminants},
};

pub fn conditional_branch<S: Streamable<Spanned<Token>>>(
    expr: Expr,
    tkns: &mut S,
) -> anyhow::Result<Spanned<StatementVariant>>
{
    // The first token must be an `if` token
    tkns.try_consume_match(
        ParserError::InternalFastpathMatchingError(expr),
        &TokenDiscriminants::If,
    )?;

    // The condition needs to be created with an open parentheses
    tkns.try_consume_match(
        ParserError::InternalFastpathMatchingError(expr),
        &TokenDiscriminants::OpenParentheses,
    )?;

    // Extract the tokens until the closing parentheses
    let condition_tokens = tkns
        .child_iterator_bulk(
            find_closing_paren(tkns)
                .ok_or(ParserError::SyntaxError(SyntaxError::LeftOpenBraces))?,
        )
        .ok_or(ParserError::EOF)?;

    Ok(todo!())
}
