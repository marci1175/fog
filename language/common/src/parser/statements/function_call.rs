use crate::{
    error::Spanned,
    parser::common::{StreamChild, Streamable},
    tokenizer::Token,
};

pub fn function_call<S: Streamable<Spanned<Token>>>(tkns: &mut S) -> anyhow::Result<()>
{
    Ok(())
}
