use crate::{error::Spanned, parser::common::StreamChild, tokenizer::Token};

pub fn conditional_if(tkns: &mut StreamChild<'_, Spanned<Token>>) -> anyhow::Result<()>
{
    Ok(())
}

pub fn conditional_elseif(tkns: &mut StreamChild<'_, Spanned<Token>>) -> anyhow::Result<()>
{
    Ok(())
}

pub fn conditional_else(tkns: &mut StreamChild<'_, Spanned<Token>>) -> anyhow::Result<()>
{
    Ok(())
}
