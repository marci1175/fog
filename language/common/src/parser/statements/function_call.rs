use crate::{
    error::Spanned,
    parser::common::{StreamChild, Streamable},
    tokenizer::Token,
};

pub fn function_call<S: Streamable<Spanned<Token>>>(
    function_name: &str,
    tkns: &mut S,
) -> anyhow::Result<()>
{
    Ok(())
}
