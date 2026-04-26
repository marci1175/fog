use crate::{error::Spanned, parser::common::StreamChild, tokenizer::Token};

pub fn loop_for(tkns: &mut StreamChild<'_, Spanned<Token>>) -> anyhow::Result<()>
{
    Ok(())
}

pub fn loop_while(tkns: &mut StreamChild<'_, Spanned<Token>>) -> anyhow::Result<()>
{
    Ok(())
}
