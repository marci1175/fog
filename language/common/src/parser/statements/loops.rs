use crate::{
    error::Spanned,
    parser::common::Streamable,
    tokenizer::Token,
};

pub fn loop_for<S: Streamable<Spanned<Token>>>(_tkns: &mut S) -> anyhow::Result<()>
{
    Ok(())
}

pub fn loop_while<S: Streamable<Spanned<Token>>>(_tkns: &mut S) -> anyhow::Result<()>
{
    Ok(())
}
