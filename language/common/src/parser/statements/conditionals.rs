use crate::{
    error::Spanned,
    parser::common::Streamable,
    tokenizer::Token,
};

pub fn conditional_if<S: Streamable<Spanned<Token>>>(_tkns: &mut S) -> anyhow::Result<()>
{
    Ok(())
}

pub fn conditional_elseif<S: Streamable<Spanned<Token>>>(_tkns: &mut S) -> anyhow::Result<()>
{
    Ok(())
}

pub fn conditional_else<S: Streamable<Spanned<Token>>>(_tkns: &mut S) -> anyhow::Result<()>
{
    Ok(())
}
