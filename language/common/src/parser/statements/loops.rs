use crate::{
    error::Spanned,
    parser::common::{StatementVariant, Streamable},
    tokenizer::Token,
};

pub fn loop_for<S: Streamable<Spanned<Token>>>(
    _tkns: &mut S,
) -> anyhow::Result<Spanned<StatementVariant>>
{
    Ok(todo!())
}

pub fn loop_while<S: Streamable<Spanned<Token>>>(
    _tkns: &mut S,
) -> anyhow::Result<Spanned<StatementVariant>>
{
    Ok(todo!())
}
