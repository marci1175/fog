use crate::{error::Spanned, parser::common::StreamChild, tokenizer::Token};

pub fn function_call(tkns: &mut StreamChild<'_, Spanned<Token>>) -> anyhow::Result<()>
{
    Ok(())
}
