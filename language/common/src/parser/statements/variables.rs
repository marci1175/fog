use crate::{error::Spanned, parser::common::StreamChild, tokenizer::Token};

pub fn var_decl(tkns: &mut StreamChild<'_, Spanned<Token>>) -> anyhow::Result<()>
{
    Ok(())
}

pub fn mod_variable(tkns: &mut StreamChild<'_, Spanned<Token>>) -> anyhow::Result<()>
{
    Ok(())
}
