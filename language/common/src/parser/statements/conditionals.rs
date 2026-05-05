use crate::{
    error::Spanned,
    parser::common::{StreamChild, Streamable},
    tokenizer::Token,
};

pub fn conditional_if<S: Streamable<Spanned<Token>>>(tkns: &mut S) -> anyhow::Result<()>
{
    Ok(())
}

pub fn conditional_elseif<S: Streamable<Spanned<Token>>>(tkns: &mut S) -> anyhow::Result<()>
{
    Ok(())
}

pub fn conditional_else<S: Streamable<Spanned<Token>>>(tkns: &mut S) -> anyhow::Result<()>
{
    Ok(())
}
