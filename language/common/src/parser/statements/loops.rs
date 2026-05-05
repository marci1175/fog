use crate::{
    error::Spanned,
    parser::common::{StreamChild, Streamable},
    tokenizer::Token,
};

pub fn loop_for<S: Streamable<Spanned<Token>>>(tkns: &mut S) -> anyhow::Result<()>
{
    Ok(())
}

pub fn loop_while<S: Streamable<Spanned<Token>>>(tkns: &mut S) -> anyhow::Result<()>
{
    Ok(())
}
