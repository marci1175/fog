use crate::{
    error::Spanned,
    parser::common::{StatementVariant, Streamable},
    tokenizer::Token,
};

pub fn conditional_if<S: Streamable<Spanned<Token>>>(
    _tkns: &mut S,
) -> anyhow::Result<Spanned<StatementVariant>>
{
    Ok(todo!())
}

pub fn conditional_elseif<S: Streamable<Spanned<Token>>>(
    _tkns: &mut S,
) -> anyhow::Result<Spanned<StatementVariant>>
{
    Ok(todo!())
}

pub fn conditional_else<S: Streamable<Spanned<Token>>>(
    _tkns: &mut S,
) -> anyhow::Result<Spanned<StatementVariant>>
{
    Ok(todo!())
}
