use crate::{
    error::Spanned,
    parser::common::Streamable,
    tokenizer::Token,
};

pub fn var_decl<S: Streamable<Spanned<Token>>>(_tkns: &mut S) -> anyhow::Result<()>
{
    Ok(())
}

pub fn mod_variable<S: Streamable<Spanned<Token>>>(_tkns: &mut S) -> anyhow::Result<()>
{
    Ok(())
}
