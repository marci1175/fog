use crate::{
    error::Spanned,
    parser::common::{StatementVariant, Streamable},
    tokenizer::Token,
};

pub fn var_decl<S: Streamable<Spanned<Token>>>(
    _tkns: &mut S,
) -> anyhow::Result<Spanned<StatementVariant>>
{
    Ok(todo!())
}

pub fn mod_variable<S: Streamable<Spanned<Token>>>(
    tkns: &mut S,
) -> anyhow::Result<Spanned<StatementVariant>>
{
    Ok(todo!())
}
