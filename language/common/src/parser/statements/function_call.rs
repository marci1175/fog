use crate::{error::Spanned, parser::common::Streamable, tokenizer::Token};

pub fn function_call<S: Streamable<Spanned<Token>>>(
    _function_name: &str,
    _tkns: &mut S,
) -> anyhow::Result<()>
{
    Ok(())
}
